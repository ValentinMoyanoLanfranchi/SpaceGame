//! Propiedades del hash de estado sobre la API **pública** de `heliarca-sim` (issue #45).
//!
//! La propiedad central: **avanzar 30 × `D1` equivale a avanzar 1 × `D30`**, comparando
//! [`Sim::state_hash`]. Se generaliza a cualquier partición del tiempo en `D30`/`D5`/`D1` y en
//! días armados con `H1`/`S5`.
//!
//! # Alcance y supuestos
//!
//! - **Escenario vacío.** Desde la API pública no se pueden registrar sistemas, así que acá se
//!   prueban tiempo, calendario, RNG y eventos vacíos. La misma propiedad con sistemas periódicos
//!   que emiten eventos y consumen el RNG está en los tests unitarios de `sim.rs`.
//! - **Sin comandos.** `CommandRecord::step` (la llamada en que se aplicó un comando) es parte
//!   del historial de entradas y **sí** entra al hash; por eso la equivalencia de particiones se
//!   enuncia sin comandos.
//! - **Sin sistemas `EveryIncrement`.** Corren una vez por llamada: dependen de la partición por
//!   diseño. La propiedad vale para las cadencias periódicas (diaria, semanal, mensual, anual).
//! - **`steps` no entra al hash** (es contabilidad de llamadas, no estado simulado): ver el
//!   módulo `state_hash`. Estas propiedades comprueban a propósito que `steps` difiere.
//!
//! # Configuración y reproducción
//!
//! Semilla fija y pocos casos (ver `tests/common/mod.rs`). Para explorar más o reproducir:
//! `PROPTEST_RNG_SEED=<u64> PROPTEST_CASES=<n> cargo test -p heliarca-sim --test properties`.

mod common;

use heliarca_sim::{Increment, Scenario, Sim};
use proptest::prelude::*;

/// Un tramo de avance que suma una cantidad entera de días.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Chunk {
    /// 1 × `D30` (30 días).
    D30,
    /// 1 × `D5` (5 días).
    D5,
    /// 1 × `D1` (1 día).
    D1,
    /// 24 × `H1` (1 día).
    Hours,
    /// 23 × `H1` + 720 × `S5` (1 día, con el último tramo en segundos).
    HoursAndSeconds,
}

impl Chunk {
    const fn days(self) -> u64 {
        match self {
            Self::D30 => 30,
            Self::D5 => 5,
            Self::D1 | Self::Hours | Self::HoursAndSeconds => 1,
        }
    }

    fn apply(self, sim: &mut Sim) {
        let mut advance = |inc: Increment, times: u32| {
            for _ in 0..times {
                sim.advance(inc).unwrap();
            }
        };
        match self {
            Self::D30 => advance(Increment::D30, 1),
            Self::D5 => advance(Increment::D5, 1),
            Self::D1 => advance(Increment::D1, 1),
            Self::Hours => advance(Increment::H1, 24),
            Self::HoursAndSeconds => {
                advance(Increment::H1, 23);
                advance(Increment::S5, 720);
            }
        }
    }
}

fn chunk() -> impl Strategy<Value = Chunk> {
    prop_oneof![
        Just(Chunk::D30),
        Just(Chunk::D5),
        Just(Chunk::D1),
        Just(Chunk::Hours),
        Just(Chunk::HoursAndSeconds),
    ]
}

/// Partición aleatoria (0 a 40 tramos, hasta 1 200 días).
fn partition() -> impl Strategy<Value = Vec<Chunk>> {
    prop::collection::vec(chunk(), 0..40)
}

fn run(seed: u64, chunks: &[Chunk]) -> Sim {
    let mut sim = Sim::new(Scenario::default(), seed).unwrap();
    for &chunk in chunks {
        chunk.apply(&mut sim);
    }
    sim
}

/// Referencia: el mismo total de días, un `D1` por vez.
fn run_daily(seed: u64, days: u64) -> Sim {
    let mut sim = Sim::new(Scenario::default(), seed).unwrap();
    for _ in 0..days {
        sim.advance(Increment::D1).unwrap();
    }
    sim
}

fn total_days(chunks: &[Chunk]) -> u64 {
    chunks.iter().map(|c| c.days()).sum()
}

proptest! {
    #![proptest_config(common::config())]

    /// Criterio del issue: 30 × `D1` ≡ 1 × `D30`, comparando hashes de estado.
    #[test]
    fn treinta_d1_equivalen_a_un_d30(seed in any::<u64>()) {
        let mut a = Sim::new(Scenario::default(), seed).unwrap();
        for _ in 0..30 {
            a.advance(Increment::D1).unwrap();
        }
        let mut b = Sim::new(Scenario::default(), seed).unwrap();
        b.advance(Increment::D30).unwrap();

        // La partición es distinta (30 llamadas contra 1)...
        prop_assert_eq!((a.steps(), b.steps()), (30, 1));
        // ...pero el estado simulado es el mismo.
        prop_assert_eq!(a.time(), b.time());
        prop_assert_eq!(a.state_hash(), b.state_hash());
    }

    /// Cualquier partición de un total de días da el mismo hash final que avanzar de a un día.
    #[test]
    fn cualquier_particion_del_mismo_total_da_el_mismo_hash(
        seed in any::<u64>(),
        chunks in partition(),
    ) {
        let days = total_days(&chunks);
        let partitioned = run(seed, &chunks);
        let daily = run_daily(seed, days);
        prop_assert_eq!(partitioned.time(), daily.time());
        prop_assert_eq!(partitioned.state_hash(), daily.state_hash());
    }

    /// Dos particiones distintas del mismo total coinciden entre sí.
    #[test]
    fn dos_particiones_del_mismo_total_coinciden(
        seed in any::<u64>(),
        a in partition(),
        b in partition(),
    ) {
        // Se completa la más corta con `D1` hasta igualar los días totales.
        let (days_a, days_b) = (total_days(&a), total_days(&b));
        let (mut a, mut b) = (a, b);
        a.extend(std::iter::repeat_n(Chunk::D1, usize::try_from(days_a.max(days_b) - days_a).unwrap()));
        b.extend(std::iter::repeat_n(Chunk::D1, usize::try_from(days_a.max(days_b) - days_b).unwrap()));
        prop_assert_eq!(run(seed, &a).state_hash(), run(seed, &b).state_hash());
    }

    /// Misma semilla y misma secuencia: mismo hash, y en cada paso.
    #[test]
    fn misma_secuencia_mismo_hash_en_cada_paso(seed in any::<u64>(), chunks in partition()) {
        let (mut a, mut b) = (Sim::new(Scenario::default(), seed).unwrap(), Sim::new(Scenario::default(), seed).unwrap());
        prop_assert_eq!(a.state_hash(), b.state_hash());
        for &chunk in &chunks {
            chunk.apply(&mut a);
            chunk.apply(&mut b);
            prop_assert_eq!(a.state_hash(), b.state_hash());
        }
    }

    /// Semillas distintas dan hashes distintos, desde el inicio y tras cualquier avance.
    #[test]
    fn semillas_distintas_dan_hashes_distintos(
        (seed_a, seed_b) in (any::<u64>(), any::<u64>()).prop_filter("semillas distintas", |(a, b)| a != b),
        chunks in partition(),
    ) {
        prop_assert_ne!(
            Sim::new(Scenario::default(), seed_a).unwrap().state_hash(),
            Sim::new(Scenario::default(), seed_b).unwrap().state_hash()
        );
        prop_assert_ne!(run(seed_a, &chunks).state_hash(), run(seed_b, &chunks).state_hash());
    }

    /// El hash sí distingue el tiempo simulado: un día más cambia el estado.
    #[test]
    fn avanzar_mas_tiempo_cambia_el_hash(seed in any::<u64>(), chunks in partition()) {
        let sim = run(seed, &chunks);
        let mut later = run(seed, &chunks);
        later.advance(Increment::D1).unwrap();
        prop_assert_ne!(sim.state_hash(), later.state_hash());
    }
}
