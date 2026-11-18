//! Propiedad de ida y vuelta del guardado «en cualquier punto» (issue #53).
//!
//! **Propiedad:** guardar, cargar y continuar es indistinguible de continuar sin guardar. Se
//! corren dos `Sim` con la misma secuencia aleatoria de operaciones: la *referencia* nunca se
//! guarda; el *sujeto* se reemplaza por su copia guardada y cargada cada vez que la secuencia lo
//! pide (varias veces en cadena: guardar -> cargar -> avanzar -> guardar -> cargar...). Tras
//! **cada** operación coinciden hash de estado, `steps`, tiempo, fecha, eventos (incluido lo que
//! devuelve `poll_events`), cola y historial de comandos y hasta el texto del guardado.
//!
//! # Qué se cubre
//!
//! - Operaciones: avances `S5`/`H1`/`D1`/`D5`/`D30`, comandos `debug.mark` enviados en pasos
//!   aleatorios (quedan pendientes hasta el siguiente avance), sondeos de eventos, y guardado
//!   por JSON (`to_save_json`/`from_save_json`) o por archivo `.sav.zst`.
//! - Escenarios **no estándar**: `Params` (5/20/200, 7/10/365...) y calendario con otra fecha de
//!   inicio, y semillas arbitrarias (incluidas `0` y `u64::MAX`). Sin esto, cargar los valores
//!   por defecto en vez de los guardados pasaría inadvertido.
//! - RNG **realmente consumido** antes de guardar: la operación `Roll` saca un `u64` del RNG raíz
//!   y lo deja en un evento. Si el guardado perdiera el estado del RNG (p. ej. reconstruirlo desde
//!   la semilla), el siguiente `Roll` del sujeto daría otro valor y todo divergiría.
//!
//! # Limitación (decisión de diseño)
//!
//! Un `Sim` con sistemas registrados no se puede guardar (`SystemsNotSerializable`: los sistemas
//! son código, no datos), así que acá **no hay sistemas**: el consumo del RNG y la emisión de
//! eventos los hace `Roll` por el acceso interno `pub(crate)`, sin ensanchar la API pública. Los
//! sistemas periódicos con subflujos del RNG están cubiertos en `partition_tests.rs`.
//!
//! # Detección comprobada a propósito
//!
//! Los tests `un_guardado_sin_el_estado_del_rng_*`, `..._con_params_por_defecto_*` y
//! `..._con_calendario_por_defecto_*` toman un guardado válido y le reemplazan ese campo por el
//! valor por defecto (lo que haría un cargador defectuoso): la carga debe fallar con
//! `HashMismatch`. Además, los mutantes del cargador se probaron a mano (ver el PR).

use std::sync::atomic::{AtomicU64, Ordering};

use proptest::prelude::*;
use serde_json::Value as Json;

use crate::SaveError;
use crate::command_registry::CommandRegistry;
use crate::events::{EventKind, Interrupt, Payload, Value};
use crate::params::Params;
use crate::proptest_config;
use crate::rng::Rng;
use crate::scenario::Scenario;
use crate::sim::Sim;
use crate::time::{Calendar, CalendarDate, Increment};

/// Una operación de la secuencia aleatoria.
#[derive(Debug, Clone)]
enum Op {
    Advance(Increment),
    /// Envía `debug.mark` (queda pendiente hasta el próximo avance).
    Mark(u8),
    /// Consume el RNG raíz y emite un evento con el valor sorteado.
    Roll,
    Poll,
    /// Sólo el sujeto: guardar por JSON y reemplazarlo por la copia cargada.
    SaveJson,
    /// Sólo el sujeto: guardar a un archivo `.sav.zst` y reemplazarlo por la copia cargada.
    SaveFile,
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => prop::sample::select(vec![
            Increment::S5, Increment::H1, Increment::D1, Increment::D5, Increment::D30,
        ]).prop_map(Op::Advance),
        2 => any::<u8>().prop_map(Op::Mark),
        3 => Just(Op::Roll),
        2 => Just(Op::Poll),
        3 => Just(Op::SaveJson),
        1 => Just(Op::SaveFile),
    ]
}

fn scenario() -> impl Strategy<Value = Scenario> {
    let params = prop_oneof![
        Just((7, 30, 365)),
        Just((5, 20, 200)),
        Just((7, 10, 365)),
        Just((3, 9, 27)),
        Just((1, 1, 1)),
    ];
    let start = (1u64..=1_000, 1u16..=365, 0u8..24, 0u8..60, 0u8..60);
    (params, start).prop_map(|((w, m, y), (year, day, hour, minute, second))| {
        Scenario::new()
            .with_params(Params::new(w, m, y).unwrap())
            .with_calendar(
                Calendar::new(CalendarDate {
                    year,
                    day,
                    hour,
                    minute,
                    second,
                })
                .unwrap(),
            )
    })
}

fn seed() -> impl Strategy<Value = u64> {
    prop_oneof![any::<u64>(), Just(0), Just(u64::MAX), Just(1)]
}

fn mark(sim: &mut Sim, n: u8) {
    let args = Payload::new().with("label", Value::Text(format!("marca {n}")));
    let command = CommandRegistry::builtin()
        .build("debug.mark", &args)
        .unwrap();
    sim.submit_command(command).unwrap();
}

/// Consume el RNG raíz y deja el valor en un evento (ver el módulo).
fn roll(sim: &mut Sim) {
    let value = sim.rng_mut().next_u64();
    let time = sim.time();
    sim.events_mut()
        .emit(
            time,
            EventKind::DebugMark,
            None,
            Interrupt::None,
            Payload::new().with("roll", Value::UInt(value)),
        )
        .unwrap();
}

/// Ruta de archivo temporal única por proceso y llamada.
fn temp_path() -> std::path::PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "heliarca-roundtrip-{}-{}.sav.zst",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ))
}

fn load_json(sim: &Sim) -> Sim {
    Sim::from_save_json(&sim.to_save_json(None).unwrap(), None).unwrap()
}

fn load_file(sim: &Sim) -> Sim {
    let path = temp_path();
    sim.save_to_file(&path, None).unwrap();
    let loaded = Sim::load_from_file(&path, None);
    std::fs::remove_file(&path).unwrap();
    loaded.unwrap()
}

/// Compara todo lo observable de dos `Sim`.
fn assert_same(a: &Sim, b: &Sim, what: &str) -> Result<(), TestCaseError> {
    prop_assert_eq!(a.state_hash(), b.state_hash(), "hash ({})", what);
    prop_assert_eq!(a.steps(), b.steps(), "steps ({})", what);
    prop_assert_eq!(a.time(), b.time(), "tiempo ({})", what);
    prop_assert_eq!(a.date(), b.date(), "fecha ({})", what);
    prop_assert_eq!(a.seed(), b.seed(), "semilla ({})", what);
    prop_assert_eq!(a.events().len(), b.events().len(), "eventos ({})", what);
    prop_assert_eq!(
        a.events().pending(),
        b.events().pending(),
        "cursor ({})",
        what
    );
    prop_assert_eq!(a.pending_command_count(), b.pending_command_count());
    prop_assert_eq!(
        a.command_history(),
        b.command_history(),
        "historial ({})",
        what
    );
    // El texto guardado resume el resto (RNG, calendario, parámetros, cola...).
    prop_assert_eq!(
        a.to_save_json(None).unwrap(),
        b.to_save_json(None).unwrap(),
        "texto del guardado ({})",
        what
    );
    Ok(())
}

/// Configuración de ESTA propiedad: determinista como el resto del repo, pero con al menos 256
/// casos aunque el valor por defecto del repo sea menor (criterio del issue).
fn config() -> ProptestConfig {
    let mut config = proptest_config::config();
    config.cases = config.cases.max(256);
    config
}

proptest! {
    #![proptest_config(config())]

    /// Criterio del issue: guardar -> cargar -> continuar == continuar sin guardar, en cualquier
    /// punto y encadenado, con escenarios y semillas arbitrarios.
    #[test]
    fn guardar_y_cargar_en_cualquier_punto_no_cambia_nada(
        scenario in scenario(),
        seed in seed(),
        ops in prop::collection::vec(op(), 1..40),
    ) {
        let mut reference = Sim::new(scenario, seed);
        let mut subject = Sim::new(scenario, seed);
        for (i, op) in ops.iter().enumerate() {
            match op {
                Op::Advance(inc) => {
                    let (r, s) = (reference.advance(*inc).unwrap(), subject.advance(*inc).unwrap());
                    prop_assert_eq!(r, s, "resultado del avance");
                }
                Op::Mark(n) => {
                    mark(&mut reference, *n);
                    mark(&mut subject, *n);
                }
                Op::Roll => {
                    roll(&mut reference);
                    roll(&mut subject);
                }
                Op::Poll => {
                    let r = reference.poll_events().to_vec();
                    let s = subject.poll_events().to_vec();
                    prop_assert_eq!(r, s, "eventos sondeados en la operación {}", i);
                }
                Op::SaveJson => subject = load_json(&subject),
                Op::SaveFile => subject = load_file(&subject),
            }
            assert_same(&reference, &subject, &format!("operación {i}: {op:?}"))?;
        }
        // Al final se sondea todo lo que quede: tienen que ser los mismos eventos.
        let r = reference.poll_events().to_vec();
        let s = subject.poll_events().to_vec();
        prop_assert_eq!(r, s, "eventos finales");
        prop_assert!(subject.check_invariants().is_ok());
    }
}

/// `Sim` de un escenario no estándar con el RNG ya consumido, eventos, cola pendiente y cursor.
fn rich_sim() -> Sim {
    let scenario = Scenario::new()
        .with_params(Params::new(5, 20, 200).unwrap())
        .with_calendar(Calendar::new(CalendarDate::at_day_start(9, 77)).unwrap());
    let mut sim = Sim::new(scenario, 12345);
    for n in 0..5 {
        roll(&mut sim);
        mark(&mut sim, n);
        sim.advance(Increment::D5).unwrap();
        let _ = sim.poll_events();
    }
    roll(&mut sim);
    mark(&mut sim, 99);
    sim
}

/// Guardado de `rich_sim` con el campo `field` reemplazado por `value`.
fn tampered(field: &str, value: Json) -> Result<Sim, SaveError> {
    let mut json: Json = serde_json::from_str(&rich_sim().to_save_json(None).unwrap()).unwrap();
    assert!(json.get(field).is_some(), "el guardado tiene `{field}`");
    json[field] = value;
    Sim::from_save_json(&json.to_string(), None)
}

#[test]
fn el_guardado_valido_de_referencia_carga() {
    let sim = rich_sim();
    assert_eq!(load_json(&sim).state_hash(), sim.state_hash());
}

/// Un cargador que reconstruyera el RNG desde la semilla (en vez de leer el estado guardado) se
/// delataría: el RNG ya avanzó, así que su estado difiere del de una semilla recién creada.
#[test]
fn un_guardado_sin_el_estado_del_rng_se_detecta() {
    let fresh = Rng::from_seed(12345);
    assert_ne!(fresh.state(), rich_sim().rng().state(), "el RNG sí avanzó");
    let error = tampered("rng_state", Json::String(format!("{:032x}", fresh.state())));
    assert!(
        matches!(error, Err(SaveError::HashMismatch { .. })),
        "{error:?}"
    );
}

#[test]
fn un_guardado_con_params_por_defecto_se_detecta() {
    let default = Params::DEFAULT;
    let value = serde_json::json!({
        "days_per_week": default.days_per_week(),
        "days_per_month": default.days_per_month(),
        "days_per_year": default.days_per_year(),
    });
    let error = tampered("params", value);
    assert!(
        matches!(error, Err(SaveError::HashMismatch { .. })),
        "{error:?}"
    );
}

#[test]
fn un_guardado_con_calendario_por_defecto_se_detecta() {
    let value = serde_json::json!({"year": 1, "day": 1, "hour": 0, "minute": 0, "second": 0});
    let error = tampered("calendar", value);
    assert!(
        matches!(error, Err(SaveError::HashMismatch { .. })),
        "{error:?}"
    );
}
