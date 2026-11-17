//! Benchmarks de avance del núcleo (`cargo bench -p heliarca-sim`).
//!
//! # Qué mide
//!
//! El costo de **avanzar** una partida con el escenario por defecto (vacío: sin sistemas, sin
//! comandos ni eventos) con la API pública: `Sim::advance` repetido hasta cubrir 1 o 40 años de
//! 365 días. Es la **línea base del núcleo** (planificador, tiempo, RNG, registro de eventos),
//! no el costo del juego real: los sistemas que se vayan sumando sumarán costo, y el
//! presupuesto del doc 21 §1/§2 («1 año de juego en < 0,75 s», «40 años en < 30 s») aplica al
//! juego completo. Un benchmark que se acerque a ese presupuesto es una señal de alarma.
//!
//! La construcción del [`Sim`] queda **fuera** de la medición (`iter_batched` con
//! `BatchSize::SmallInput`: la preparación no se cronometra). Cada iteración mide sólo los
//! `advance`; el `Sim` resultante se devuelve a criterion para que se destruya fuera del tiempo
//! medido y el optimizador no pueda descartar el trabajo.
//!
//! # Nombres estables
//!
//! `advance_1_year_d1` (365 × `D1`), `advance_1_year_d5` (73 × `D5`), `advance_40_years_d1`
//! (14 600 × `D1`) y `advance_40_years_d5` (2 920 × `D5`). `D30` no se incluye: 365 no es
//! múltiplo de 30 y mezclar incrementos esconde qué se mide. No renombrar: la comparación contra
//! una línea base (README, «Benchmarks») usa estos nombres.
//!
//! # Tiempo
//!
//! Este archivo no usa el reloj del sistema: sólo criterion cronometra (el `clippy.toml` de
//! heliarca-sim prohíbe `std::time::SystemTime` e `Instant`, y sus `now`, en nuestro código;
//! `std::time::Duration`, que sólo es un valor, sí está permitido).

use std::hint::black_box;
use std::time::Duration;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use heliarca_sim::{Increment, Scenario, Sim};

/// Semilla fija: el avance no depende de ella en el escenario vacío, pero el bench es
/// reproducible igual.
const SEED: u64 = 1;

/// Días de un año del calendario (ver el issue #439 sobre `days_per_year`).
const DAYS_PER_YEAR: u64 = 365;

/// Registra un benchmark que avanza `steps` veces con `increment`.
fn bench_advance(c: &mut Criterion, name: &str, increment: Increment, steps: u64) {
    c.bench_function(name, |b| {
        b.iter_batched(
            || Sim::new(Scenario::default(), black_box(SEED)),
            |mut sim| {
                for _ in 0..steps {
                    let result = sim.advance(black_box(increment));
                    // Un error o una interrupción invalidaría la medición: se aborta a gritos.
                    assert!(
                        result.is_ok_and(|r| r.completed()),
                        "el avance no se completó"
                    );
                }
                sim
            },
            BatchSize::SmallInput,
        );
    });
}

fn advance(c: &mut Criterion) {
    bench_advance(c, "advance_1_year_d1", Increment::D1, DAYS_PER_YEAR);
    bench_advance(c, "advance_1_year_d5", Increment::D5, DAYS_PER_YEAR / 5);
    bench_advance(c, "advance_40_years_d1", Increment::D1, 40 * DAYS_PER_YEAR);
    bench_advance(
        c,
        "advance_40_years_d5",
        Increment::D5,
        40 * DAYS_PER_YEAR / 5,
    );
}

criterion_group! {
    name = benches;
    // Pocos segundos por benchmark: el avance en escenario vacío son microsegundos a
    // milisegundos, así que 1 s de medición y 20 muestras alcanzan para una estimación estable.
    config = Criterion::default()
        .sample_size(20)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(1));
    targets = advance
}
criterion_main!(benches);
