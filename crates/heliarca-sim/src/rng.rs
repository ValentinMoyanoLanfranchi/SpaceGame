//! RNG determinista con subflujos estables (doc 21 §2 «Determinista»).
//!
//! # Garantía de estabilidad
//!
//! Dada una semilla `u64`, la secuencia de un [`Rng`] y la de todos sus subflujos es **la misma
//! en cualquier plataforma, compilador y versión de este crate**. Cambiar cualquiera de los
//! algoritmos de abajo (o las constantes) cambia las partidas guardadas y las repeticiones, por
//! lo que es un cambio incompatible: los tests «golden» de este módulo existen para detectarlo.
//!
//! # Algoritmos (todos con aritmética `u64` explícita y bytes en orden little-endian)
//!
//! - **Generador:** `Pcg64Mcg` de `rand_pcg` (PCG-XSL-RR 128/64 con congruencial multiplicativo).
//!   Se eligió por tener 128 bits de estado, buena calidad estadística, ser muy rápido y no
//!   necesitar un parámetro de flujo (los flujos independientes se obtienen derivando semillas,
//!   no con constantes de incremento). Su algoritmo está fijado por la versión exacta de la
//!   dependencia (`Cargo.lock`).
//! - **Expansión de la semilla:** el estado inicial de 128 bits se arma con las dos primeras
//!   salidas de [`splitmix64`] sobre la semilla (la primera es la mitad baja, la segunda la
//!   alta), serializadas little-endian como el `Seed` de `Pcg64Mcg`. Así no se depende del
//!   `seed_from_u64` de `rand_core`, cuyo algoritmo no es una garantía de la API.
//! - **Hash de etiqueta:** [`fnv1a_64`] sobre los bytes UTF-8 de la etiqueta (offset
//!   `0xcbf29ce484222325`, primo `0x100000001b3`).
//! - **Derivación:** `derive(label)` crea un [`Rng`] nuevo con la semilla
//!   `splitmix_step(splitmix_step(seed) ^ fnv1a_64(label))`, donde `splitmix_step` es la
//!   función de mezcla de `SplitMix64` (incluye la suma de la constante áurea previa).
//!
//! # Propiedades de la derivación
//!
//! - Es una **función pura de `(semilla, etiqueta)`**: no consume ni altera el flujo del padre,
//!   y no depende de cuánto se haya consumido de él ni del orden en que se derive.
//! - **Compone:** el hijo es un [`Rng`] completo con su propia semilla, por lo que
//!   `derive("a").derive("b")` es estable y distinto de `derive("b")` (y de
//!   `derive("b").derive("a")`).
//! - Etiquetas distintas dan semillas distintas salvo colisiones del hash de 64 bits
//!   (probabilidad ~2^-64 por par); no hay detección de colisiones.

use rand_core::{Rng as _, SeedableRng};
use rand_pcg::Pcg64Mcg;

/// Constante de incremento de `SplitMix64` (la parte fraccionaria de la razón áurea en 64 bits).
const SPLITMIX_GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;
/// Offset basis de FNV-1a de 64 bits.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// Primo de FNV de 64 bits.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Hash FNV-1a de 64 bits sobre `bytes` (especificación pública de Fowler–Noll–Vo).
///
/// Es una función fija: no depende de la plataforma, del proceso ni de la versión de Rust.
#[must_use]
pub fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Paso de `SplitMix64`: suma la constante áurea a `x` y mezcla el resultado (Steele et al.).
///
/// Es una biyección sobre `u64` y no tiene el punto fijo en 0 de la mezcla sin suma.
#[must_use]
pub fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(SPLITMIX_GAMMA);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Generador de números pseudoaleatorios determinista, con semilla y subflujos.
///
/// No implementa `Clone` a propósito: duplicar un flujo es una fuente típica de correlaciones
/// ocultas; para un flujo independiente usar [`Rng::derive`].
#[derive(Debug)]
pub struct Rng {
    seed: u64,
    inner: Pcg64Mcg,
}

impl Rng {
    /// Crea un generador a partir de una semilla. La misma semilla da siempre la misma secuencia.
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        // Dos salidas consecutivas de SplitMix64 (estado `seed`, `seed + gamma`) forman el
        // estado de 128 bits, little-endian.
        let low = splitmix64(seed);
        let high = splitmix64(seed.wrapping_add(SPLITMIX_GAMMA));
        let mut bytes = [0u8; 16];
        bytes[..8].copy_from_slice(&low.to_le_bytes());
        bytes[8..].copy_from_slice(&high.to_le_bytes());
        Self {
            seed,
            inner: Pcg64Mcg::from_seed(bytes),
        }
    }

    /// Semilla con la que se creó este generador (para un derivado, la semilla derivada).
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Deriva un subflujo estable e independiente identificado por `label`.
    ///
    /// Función pura de `(semilla, etiqueta)`: no consume ni altera este generador. El resultado
    /// es un [`Rng`] completo, así que se puede volver a derivar (ver la documentación del
    /// módulo para el algoritmo exacto).
    #[must_use]
    pub fn derive(&self, label: &str) -> Rng {
        let mixed = splitmix64(self.seed) ^ fnv1a_64(label.as_bytes());
        Rng::from_seed(splitmix64(mixed))
    }

    /// Estado interno de 128 bits del generador, para el hash de estado (`state_hash`). No se
    /// expone fuera del crate ni se hace `Clone` el generador.
    pub(crate) fn state(&self) -> u128 {
        self.inner.state()
    }

    /// `(semilla, estado interno)` para el hash. Desestructura **sin `..`**: un campo nuevo obliga
    /// a decidir si se hashea.
    pub(crate) fn hash_parts(&self) -> (u64, u128) {
        let Self { seed, inner } = self;
        (*seed, inner.state())
    }

    /// Reconstruye un generador desde su semilla y su estado interno (camino inverso de
    /// [`Rng::hash_parts`], para cargar un guardado). `None` si `state` es par: el estado de
    /// `Pcg64Mcg` es siempre impar, así que un valor par sólo puede venir de datos corruptos.
    /// Construye con un literal **sin `..`**: un campo nuevo obliga a decidir cómo se restaura.
    pub(crate) fn from_parts(seed: u64, state: u128) -> Option<Self> {
        if state & 1 == 0 {
            return None;
        }
        Some(Self {
            seed,
            inner: Pcg64Mcg::new(state),
        })
    }

    /// Siguiente valor de 64 bits.
    pub fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }

    /// Siguiente valor de 32 bits (los 32 bits bajos del siguiente `u64` de `Pcg64Mcg`).
    pub fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn take(rng: &mut Rng, n: usize) -> Vec<u64> {
        (0..n).map(|_| rng.next_u64()).collect()
    }

    // Vectores de referencia publicados de FNV-1a de 64 bits.
    #[test]
    fn fnv1a_coincide_con_vectores_publicados() {
        let casos: [(&str, u64); 3] = [
            ("", 0xcbf2_9ce4_8422_2325),
            ("a", 0xaf63_dc4c_8601_ec8c),
            ("foobar", 0x8594_4171_f739_67e8),
        ];
        for (entrada, esperado) in casos {
            assert_eq!(
                fnv1a_64(entrada.as_bytes()),
                esperado,
                "entrada {entrada:?}"
            );
        }
    }

    // Vectores de referencia de SplitMix64 con semilla 0 (el generador original produce
    // estos valores en sus primeras salidas; cada salida es `splitmix64(estado)`).
    #[test]
    fn splitmix64_coincide_con_vectores_conocidos() {
        assert_eq!(splitmix64(0), 0xe220_a839_7b1d_cdaf);
        assert_eq!(splitmix64(SPLITMIX_GAMMA), 0x6e78_9e6a_a1b9_65f4);
    }

    // VALORES GOLDEN. Fijan la secuencia para siempre: si un cambio los rompe, rompió el
    // determinismo (partidas guardadas y repeticiones dejan de valer) y NO hay que «arreglar»
    // el test sino revertir el cambio, o decidir una migración explícita de formato.
    // Para regenerarlos (sólo en una migración deliberada): imprimir `take(&mut rng, 4)` en el
    // caso correspondiente con `cargo test -- --nocapture` y pegar los literales.
    const GOLDEN_SEED_42: [u64; 4] = [
        0x423e_b8fc_be48_4751,
        0x4c4a_eb93_5620_c4a8,
        0xc7dc_cc22_2f9a_c63e,
        0x5dd5_b18e_c30f_be13,
    ];
    const GOLDEN_SEED_42_DERIVE_PLANETA: [u64; 4] = [
        0x1fb8_dbd1_9846_cfd5,
        0x38de_20db_7247_3d4e,
        0x9bcc_753e_f40a_d622,
        0x6ca3_75ab_a76f_94b6,
    ];
    const GOLDEN_SEED_42_DERIVE_A_B: [u64; 4] = [
        0x033b_2e9a_2a31_77b6,
        0x48c1_824c_756b_3e43,
        0x9d2a_5c12_66cd_478b,
        0xb346_305d_1df6_dd32,
    ];

    #[test]
    fn golden_semilla_42() {
        let mut rng = Rng::from_seed(42);
        let v = take(&mut rng, 4);
        assert_eq!(v, GOLDEN_SEED_42);
    }

    #[test]
    fn golden_subflujo_planeta() {
        let mut rng = Rng::from_seed(42).derive("planeta");
        let v = take(&mut rng, 4);
        assert_eq!(v, GOLDEN_SEED_42_DERIVE_PLANETA);
    }

    #[test]
    fn golden_subflujo_compuesto() {
        let mut rng = Rng::from_seed(42).derive("a").derive("b");
        let v = take(&mut rng, 4);
        assert_eq!(v, GOLDEN_SEED_42_DERIVE_A_B);
    }

    // La semilla derivada también queda fija (cubre la mezcla, incluida una etiqueta no ASCII,
    // cuyos bytes UTF-8 dependen del orden de bytes y no de la plataforma).
    #[test]
    fn golden_semillas_derivadas() {
        let raiz = Rng::from_seed(42);
        assert_eq!(raiz.derive("planeta").seed(), 0x328b_4c37_70e5_b55e);
        assert_eq!(raiz.derive("órbita").seed(), 0x3274_3711_dc8a_b633);
    }

    #[test]
    fn misma_semilla_misma_secuencia() {
        let a = take(&mut Rng::from_seed(7), 64);
        let b = take(&mut Rng::from_seed(7), 64);
        assert_eq!(a, b);
    }

    #[test]
    fn misma_semilla_y_etiqueta_misma_secuencia() {
        let a = take(&mut Rng::from_seed(7).derive("x"), 64);
        let b = take(&mut Rng::from_seed(7).derive("x"), 64);
        assert_eq!(a, b);
    }

    #[test]
    fn etiquetas_distintas_secuencias_distintas() {
        let raiz = Rng::from_seed(7);
        let etiquetas = ["", "a", "b", "A", "ab", "ba", "a ", "sistema", "sistemas"];
        let secuencias: Vec<Vec<u64>> = etiquetas
            .iter()
            .map(|e| take(&mut raiz.derive(e), 8))
            .collect();
        for i in 0..secuencias.len() {
            for j in (i + 1)..secuencias.len() {
                assert_ne!(
                    secuencias[i], secuencias[j],
                    "{:?} vs {:?}",
                    etiquetas[i], etiquetas[j]
                );
            }
        }
    }

    #[test]
    fn semillas_distintas_secuencias_distintas() {
        let semillas = [0u64, 1, 2, 42, u64::MAX];
        let secuencias: Vec<Vec<u64>> = semillas
            .iter()
            .map(|&s| take(&mut Rng::from_seed(s), 8))
            .collect();
        for i in 0..secuencias.len() {
            for j in (i + 1)..secuencias.len() {
                assert_ne!(
                    secuencias[i], secuencias[j],
                    "{} vs {}",
                    semillas[i], semillas[j]
                );
            }
        }
        // También con la misma etiqueta.
        for (&a, &b) in semillas.iter().zip(semillas.iter().skip(1)) {
            assert_ne!(
                take(&mut Rng::from_seed(a).derive("x"), 8),
                take(&mut Rng::from_seed(b).derive("x"), 8)
            );
        }
    }

    #[test]
    fn derivar_no_altera_al_padre() {
        let mut con_derivacion = Rng::from_seed(9);
        let mut sin_derivacion = Rng::from_seed(9);
        let antes = take(&mut con_derivacion, 3);
        let _ = con_derivacion.derive("x");
        let _ = con_derivacion.derive("y").derive("z");
        let despues = take(&mut con_derivacion, 3);
        let esperado = take(&mut sin_derivacion, 6);
        assert_eq!([antes, despues].concat(), esperado);
    }

    #[test]
    fn derivar_no_depende_de_lo_consumido_del_padre() {
        let mut padre = Rng::from_seed(9);
        let antes = take(&mut padre.derive("x"), 8);
        let _ = take(&mut padre, 100);
        let despues = take(&mut padre.derive("x"), 8);
        assert_eq!(antes, despues);
    }

    #[test]
    fn derivar_compone_y_el_orden_importa() {
        let raiz = Rng::from_seed(3);
        let ab = take(&mut raiz.derive("a").derive("b"), 8);
        assert_eq!(ab, take(&mut raiz.derive("a").derive("b"), 8));
        assert_ne!(ab, take(&mut raiz.derive("b"), 8));
        assert_ne!(ab, take(&mut raiz.derive("a"), 8));
        assert_ne!(ab, take(&mut raiz.derive("b").derive("a"), 8));
    }

    #[test]
    fn derivar_con_etiqueta_vacia_no_devuelve_al_padre() {
        let raiz = Rng::from_seed(3);
        assert_ne!(raiz.derive("").seed(), raiz.seed());
        assert_ne!(
            take(&mut raiz.derive(""), 4),
            take(&mut Rng::from_seed(3), 4)
        );
    }

    #[test]
    fn next_u32_son_los_bits_bajos_del_u64() {
        let mut a = Rng::from_seed(5);
        let mut b = Rng::from_seed(5);
        for _ in 0..16 {
            let esperado = u32::try_from(a.next_u64() & 0xffff_ffff).unwrap_or(0);
            assert_eq!(b.next_u32(), esperado);
        }
    }

    #[test]
    fn la_semilla_se_conserva() {
        assert_eq!(Rng::from_seed(123).seed(), 123);
    }
}
