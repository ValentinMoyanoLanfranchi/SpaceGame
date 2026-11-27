//! Propiedades de equivalencia de particiones **con sistemas** (issue #45).
//!
//! Complementa a `tests/properties.rs` (API pública, escenario vacío): acá, desde dentro del
//! crate, se registran sistemas periódicos de prueba (diario, semanal, mensual y anual) que
//! emiten eventos y consumen un subflujo del RNG, y se comprueba que **cómo se parta el avance
//! no cambia lo simulado**: misma ejecución de sistemas (instante y orden), mismos eventos y
//! mismo hash de estado.
//!
//! # Alcance y supuestos
//!
//! - **Sin comandos:** `CommandRecord::step` es parte del historial de entradas y entra al hash.
//! - **Sin `EveryIncrement`:** corre una vez por llamada, depende de la partición por diseño.
//! - **`steps` no entra al hash** (contabilidad de llamadas, ver `state_hash`): los tests
//!   comprueban que difiere entre las particiones.
//! - **Con interrupciones** (`Interrupt::Halt`/`SlowDown`) un avance se detiene en el instante
//!   del evento, así que 1 × `D30` no equivale a 30 × `D1` *llamada a llamada*. Lo que sí vale (y
//!   se prueba) es que, volviendo a avanzar hasta cubrir el mismo total de días, el estado final
//!   converge: el avance interrumpido deja el planificador en el instante exacto de la frontera y
//!   continúa desde ahí sin repetir ni saltear corridas. Esto requiere que las interrupciones
//!   caigan en fronteras de día (así es: los sistemas son periódicos).

use std::cell::RefCell;
use std::rc::Rc;

use proptest::prelude::*;

use crate::events::{EventId, EventKind, Interrupt, Payload, Value};
use crate::ids::EntityId;
use crate::proptest_config;
use crate::rng::Rng;
use crate::scenario::Scenario;
use crate::schedule::{Cadence, Priority, StepContext, System, SystemId};
use crate::sim::Sim;
use crate::state_hash::StateWriter;
use crate::time::{Increment, SECONDS_PER_DAY};

type Runs = Rc<RefCell<Vec<(&'static str, u64)>>>;

/// Sistema periódico de prueba: en cada corrida anota `(id, instante)`, consume su subflujo del
/// RNG y emite un evento encadenado al anterior propio. Si `interrupt_one_in` es `Some(n)`,
/// interrumpe (`Halt`) cuando el valor sorteado es múltiplo de `n`.
#[derive(Debug)]
struct Periodic {
    id: &'static str,
    priority: u16,
    cadence: Cadence,
    rng: Rng,
    last: Option<EventId>,
    interrupt_one_in: Option<u64>,
    runs: Runs,
}

impl System for Periodic {
    fn id(&self) -> SystemId {
        SystemId(self.id)
    }
    fn priority(&self) -> Priority {
        Priority(self.priority)
    }
    fn cadence(&self) -> Cadence {
        self.cadence
    }
    fn step(&mut self, ctx: &mut StepContext<'_>) {
        let now = ctx.time().as_seconds();
        self.runs.borrow_mut().push((self.id, now));
        let roll = self.rng.next_u64();
        let interrupt = match self.interrupt_one_in {
            Some(n) if roll.is_multiple_of(n) => Interrupt::Halt,
            _ => Interrupt::None,
        };
        let payload = Payload::new()
            .with("roll", Value::UInt(roll % 1_000))
            .with("sys", Value::Text(self.id.into()));
        self.last = ctx
            .emit(EventKind::JumpPointCrossed, self.last, interrupt, payload)
            .ok();
    }
    fn hash_state(&self, w: &mut StateWriter) {
        w.write_u128(self.rng.state());
        w.write_option(self.last, |w, id| w.write_u32(id.raw()));
    }
}

/// `Sim` con un sistema por cadencia periódica (el diario es el único que puede interrumpir,
/// con probabilidad `1/n` por corrida si `interrupt_one_in` es `Some(n)`).
fn sim_with_systems(seed: u64, interrupt_one_in: Option<u64>) -> (Sim, Runs) {
    let mut sim = Sim::new(Scenario::default(), seed).unwrap();
    let runs = Runs::default();
    for (id, priority, cadence) in [
        ("test.diario", 1, Cadence::Daily),
        ("test.semanal", 2, Cadence::Weekly),
        ("test.mensual", 3, Cadence::Monthly),
        ("test.anual", 4, Cadence::Yearly),
    ] {
        let rng = sim.rng().derive(id);
        sim.scheduler_mut()
            .register(Box::new(Periodic {
                id,
                priority,
                cadence,
                rng,
                last: None,
                interrupt_one_in: interrupt_one_in.filter(|_| cadence == Cadence::Daily),
                runs: Rc::clone(&runs),
            }))
            .unwrap();
    }
    (sim, runs)
}

/// Incremento de días enteros que cabe en `remaining` días, elegido por `pick`.
fn pick_increment(pick: u8, remaining: u64) -> (Increment, u64) {
    let options: Vec<(Increment, u64)> =
        [(Increment::D30, 30), (Increment::D5, 5), (Increment::D1, 1)]
            .into_iter()
            .filter(|&(_, days)| days <= remaining)
            .collect();
    options[usize::from(pick) % options.len()]
}

/// Avanza `total` días eligiendo incrementos con `picks` (se reciclan; `D1` si se acaban).
/// Si hay interrupciones, vuelve a avanzar hasta cubrir el total. Devuelve las llamadas hechas.
fn advance_partitioned(sim: &mut Sim, total: u64, picks: &[u8]) -> u64 {
    let (mut remaining, mut calls) = (total, 0u64);
    while remaining > 0 {
        let pick = picks
            .get(usize::try_from(calls).unwrap())
            .copied()
            .unwrap_or(2);
        let (increment, _) = pick_increment(pick, remaining);
        let result = sim.advance(increment).unwrap();
        assert!(result.advanced > 0, "el avance debe progresar");
        assert_eq!(result.advanced % SECONDS_PER_DAY, 0, "días enteros");
        remaining -= result.advanced / SECONDS_PER_DAY;
        calls += 1;
    }
    calls
}

/// Referencia: un `D1` por vez hasta cubrir `total` días.
fn advance_daily(sim: &mut Sim, total: u64) {
    advance_partitioned(sim, total, &[]);
}

/// Aserta que dos corridas simularon lo mismo: tiempo, ejecuciones, eventos y hash.
fn assert_same_simulation(
    (a, runs_a): &(Sim, Runs),
    (b, runs_b): &(Sim, Runs),
) -> Result<(), TestCaseError> {
    prop_assert_eq!(a.time(), b.time());
    prop_assert_eq!(
        &*runs_a.borrow(),
        &*runs_b.borrow(),
        "ejecución de sistemas"
    );
    let events = |s: &Sim| s.events().iter().cloned().collect::<Vec<_>>();
    prop_assert_eq!(events(a), events(b), "eventos");
    prop_assert_eq!(a.state_hash(), b.state_hash(), "hash de estado");
    Ok(())
}

proptest! {
    #![proptest_config(proptest_config::config())]

    /// Criterio del issue con sistemas: 30 × `D1` ≡ 1 × `D30`.
    #[test]
    fn treinta_d1_equivalen_a_un_d30_con_sistemas_periodicos(seed in any::<u64>()) {
        let mut a = sim_with_systems(seed, None);
        for _ in 0..30 {
            a.0.advance(Increment::D1).unwrap();
        }
        let mut b = sim_with_systems(seed, None);
        b.0.advance(Increment::D30).unwrap();
        prop_assert_eq!((a.0.steps(), b.0.steps()), (30, 1));
        // 30 diarias + 4 semanales + 1 mensual: no es un caso vacío.
        prop_assert_eq!(a.1.borrow().len(), 35);
        assert_same_simulation(&a, &b)?;
    }

    /// Particiones aleatorias de 1 a 400 días (cruza el año: corre el sistema anual).
    #[test]
    fn cualquier_particion_de_1_a_400_dias_da_lo_mismo(
        seed in any::<u64>(),
        total in 1u64..=400,
        picks in prop::collection::vec(any::<u8>(), 0..64),
    ) {
        let mut a = sim_with_systems(seed, None);
        advance_partitioned(&mut a.0, total, &picks);
        let mut b = sim_with_systems(seed, None);
        advance_daily(&mut b.0, total);
        prop_assert_eq!(a.0.time().as_seconds(), total * SECONDS_PER_DAY);
        assert_same_simulation(&a, &b)?;
    }

    /// Con interrupciones, volver a avanzar hasta cubrir el total converge al mismo estado.
    #[test]
    fn con_interrupciones_las_particiones_convergen_al_cubrir_el_total(
        seed in any::<u64>(),
        total in 1u64..=400,
        picks in prop::collection::vec(any::<u8>(), 0..64),
        one_in in 2u64..=9,
    ) {
        let mut a = sim_with_systems(seed, Some(one_in));
        advance_partitioned(&mut a.0, total, &picks);
        let mut b = sim_with_systems(seed, Some(one_in));
        advance_daily(&mut b.0, total);
        assert_same_simulation(&a, &b)?;
    }

    /// Semillas distintas divergen aunque la partición sea la misma.
    #[test]
    fn semillas_distintas_divergen_con_sistemas(
        (seed_a, seed_b) in (any::<u64>(), any::<u64>()).prop_filter("distintas", |(a, b)| a != b),
        total in 1u64..=60,
    ) {
        let (mut a, mut b) = (sim_with_systems(seed_a, None), sim_with_systems(seed_b, None));
        advance_daily(&mut a.0, total);
        advance_daily(&mut b.0, total);
        prop_assert_ne!(a.0.state_hash(), b.0.state_hash());
    }
}

/// Las propiedades con interrupciones no son vacuas: con una semilla fija hay interrupciones,
/// la partición cambia la cantidad de llamadas y el estado final coincide igual.
#[test]
fn las_interrupciones_ocurren_y_la_particion_cambia_los_pasos_no_el_hash() {
    let (mut a, runs_a) = sim_with_systems(7, Some(3));
    let calls = advance_partitioned(&mut a, 400, &[0; 64]);
    let (mut b, runs_b) = sim_with_systems(7, Some(3));
    advance_daily(&mut b, 400);
    assert!(a.events().iter().any(|e| e.interrupt != Interrupt::None));
    assert_eq!(a.steps(), calls);
    assert_ne!(a.steps(), b.steps(), "particiones distintas");
    assert_eq!(*runs_a.borrow(), *runs_b.borrow());
    assert_eq!(a.state_hash(), b.state_hash());
}
