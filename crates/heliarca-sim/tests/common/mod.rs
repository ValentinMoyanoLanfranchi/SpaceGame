//! Configuración compartida de `proptest` para los tests de propiedades de `heliarca-sim`.
//!
//! Se usa desde `tests/properties.rs` (`mod common;`) y desde los tests unitarios del crate
//! (`#[path]` en `lib.rs`), así que hay una sola política.
//!
//! # Política
//!
//! - **Determinista por defecto:** semilla fija ([`DEFAULT_SEED`]) y algoritmo `ChaCha` (el de proptest por defecto). El CI
//!   (Linux y Windows) ejecuta siempre los mismos casos: no hay fallos intermitentes ni «anduvo
//!   en mi máquina». Para explorar más: `PROPTEST_RNG_SEED=<u64>` y/o `PROPTEST_CASES=<n>`.
//! - **Pocos casos:** [`DEFAULT_CASES`] (64) por propiedad, para no encarecer el CI. Subir con
//!   `PROPTEST_CASES`.
//! - **Sin persistencia de fallos:** no se escribe `proptest-regressions/` (ensuciaría el
//!   repositorio). Un fallo se reproduce con la semilla y el caso mínimo que imprime proptest
//!   (ver CONTRIBUTING.md); si el caso merece quedar para siempre, se lo copia a un test de
//!   ejemplo normal.
//!
//! El resto de las variables `PROPTEST_*` (p. ej. `PROPTEST_MAX_SHRINK_ITERS`) las lee
//! [`ProptestConfig::default`] como siempre.

use proptest::test_runner::{Config as ProptestConfig, RngSeed};

/// Semilla por defecto de proptest en este repositorio.
pub const DEFAULT_SEED: u64 = 0x4845_4C49_4152_4341; // «HELIARCA»

/// Casos por propiedad si no se fija `PROPTEST_CASES`.
pub const DEFAULT_CASES: u32 = 64;

/// Configuración de proptest del repositorio (ver el módulo).
#[must_use]
pub fn config() -> ProptestConfig {
    // `default()` ya incorpora las variables `PROPTEST_*` del entorno.
    let mut config = ProptestConfig::default();
    if std::env::var_os("PROPTEST_CASES").is_none() {
        config.cases = DEFAULT_CASES;
    }
    if matches!(config.rng_seed, RngSeed::Random) {
        config.rng_seed = RngSeed::Fixed(DEFAULT_SEED);
    }
    config.failure_persistence = None;
    config
}
