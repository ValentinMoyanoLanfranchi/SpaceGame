//! Marco de invariantes (doc 21 §2 y §4): la red de seguridad del simulador.
//!
//! Un **invariante** es una propiedad que el estado de [`Sim`] cumple siempre, en cualquier
//! instante entre dos pasos (p. ej. «el tiempo nunca retrocede», «los ids de evento son
//! secuenciales»). No valida entradas del jugador (eso lo hace [`Command::validate`](crate::Command::validate))
//! ni reemplaza a los tests: detecta **errores de programación** que dejarían el estado
//! corrupto, lo antes posible y señalando qué se rompió y dónde. Cuando haya recursos y recetas
//! (M0.3+), «no se crean ni destruyen recursos sin receta» entra como un invariante más.
//!
//! # Piezas
//!
//! - [`Violation`] (pública): qué invariante falló (nombre estable), sobre qué entidad
//!   (`event#7`, `system.sys.prod`, `command#3`...) y el detalle en español.
//! - [`Invariant`] (interno): `name()` + `check(&Sim)`. Es `pub(crate)` a propósito: desde afuera
//!   sólo se necesita [`Sim::check_invariants`] y [`Violation`]; el catálogo de invariantes es un
//!   detalle de implementación que crece con cada módulo y no debe quedar fijado en la API.
//! - **Cada módulo dueño de estado registra los suyos** en una constante `INVARIANTS`
//!   (`&[FnInvariant]`), y este módulo las reúne en un registro único de orden fijo y
//!   determinista (módulos en el orden de [`all`], y dentro de cada uno el de su constante).
//!
//! # Cuándo corren
//!
//! - [`Sim::check_invariants`] es pública y se puede llamar cuando se quiera (devuelve la
//!   **primera** violación en el orden del registro; si se necesitan todas, se corrige y se
//!   vuelve a llamar). No modifica el estado ni el hash.
//! - **Automáticamente**, sólo con `cfg(any(test, debug_assertions))` (tests y compilaciones de
//!   desarrollo): [`Sim::advance`] corre todos los invariantes **antes** de aplicar el paso
//!   (detecta corrupción ocurrida entre pasos, sin tocar nada) y **después** (detecta un paso que
//!   dejó el estado mal), y devuelve [`AdvanceError::InvariantViolated`](crate::AdvanceError::InvariantViolated).
//!   En release **no** corren (costo cero).
//! - **No** corren en [`Sim::new`] (el estado inicial es válido por construcción; hay un test que
//!   lo comprueba) ni en [`Sim::submit_command`] (sólo agrega a la cola, que el chequeo previo del
//!   siguiente `advance` cubre).
//!
//! # Costo
//!
//! Cada chequeo recorre todo el estado: **O(eventos + comandos + sistemas)**, y `advance` lo hace
//! dos veces en tests/debug. Es una red de seguridad para desarrollo, no algo para cada cuadro:
//! por eso no se ejecuta en release.
//!
//! # Mojón de tiempo y hash de estado
//!
//! «El tiempo no retrocede» necesita recordar el último instante chequeado. Ese mojón
//! ([`Mark`]) vive en el `Sim`, se actualiza al terminar cada paso y **no** entra al hash de
//! estado: es contabilidad derivada (se reconstruye como el tiempo al final del último paso), no
//! estado simulado; que el chequeo esté activo o no no puede cambiar [`Sim::state_hash`].
//!
//! # Agregar un invariante
//!
//! 1. En el módulo dueño del estado, una función `fn check_xxx(sim: &Sim) -> Result<(), Violation>`
//!    que devuelva `Violation::new(NOMBRE, Some(entidad), detalle)` con el **mismo** nombre con el
//!    que se registra.
//! 2. Agregarla a la constante `INVARIANTS` del módulo con [`FnInvariant::new`]`("modulo.nombre", f)`.
//!    El nombre es estable (`dominio.regla`): otros lo citan en tests y reportes, no se renombra.
//! 3. Un test `cfg(test)` que corrompa el estado a propósito y compruebe nombre y entidad.
//!
//! Debe ser barato (lineal), determinista y sin efectos; sin `HashMap`, sin `panic!`/`unwrap`.

use std::fmt;

use crate::sim::Sim;
use crate::time::SimTime;

/// Un invariante roto: qué regla, sobre qué entidad y por qué.
///
/// Es el error de [`Sim::check_invariants`] y de
/// [`AdvanceError::InvariantViolated`](crate::AdvanceError::InvariantViolated). Su `Display` lo
/// resume en una línea en español, p. ej.
/// `invariante «events.causes_precede» violado en event#7: la causa event#9 no es anterior`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    invariant: &'static str,
    entity: Option<String>,
    detail: String,
}

impl Violation {
    /// Arma una violación (uso interno: lo hacen los invariantes de cada módulo).
    pub(crate) fn new(
        invariant: &'static str,
        entity: Option<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            invariant,
            entity,
            detail: detail.into(),
        }
    }

    /// Nombre estable del invariante roto (`dominio.regla`, p. ej. `time.monotonic`).
    #[must_use]
    pub const fn invariant(&self) -> &'static str {
        self.invariant
    }

    /// Entidad afectada (`event#7`, `command#3`, `system.<id>`, `sim.time`...), si hay una
    /// concreta.
    #[must_use]
    pub fn entity(&self) -> Option<&str> {
        self.entity.as_deref()
    }

    /// Explicación en español de qué se esperaba y qué se encontró.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invariante «{}» violado", self.invariant)?;
        if let Some(entity) = &self.entity {
            write!(f, " en {entity}")?;
        }
        write!(f, ": {}", self.detail)
    }
}

impl std::error::Error for Violation {}

/// Nombre de entidad de un evento en las violaciones (`event#7`).
pub(crate) fn entity_event(raw: u32) -> String {
    format!("event#{raw}")
}

/// Nombre de entidad de un comando en las violaciones (`command#3`).
pub(crate) fn entity_command(raw: u32) -> String {
    format!("command#{raw}")
}

/// Una regla que el estado de [`Sim`] debe cumplir siempre entre dos pasos.
pub(crate) trait Invariant {
    /// Nombre estable `dominio.regla`. Hoy sólo lo leen los tests (que comprueban que sea único
    /// y que coincida con el de la violación); es parte del contrato del invariante.
    #[allow(dead_code)]
    fn name(&self) -> &'static str;

    /// Comprueba la regla sobre el estado completo, sin modificarlo.
    ///
    /// # Errors
    ///
    /// La [`Violation`] (con `invariant() == name()`) si no se cumple.
    fn check(&self, sim: &Sim) -> Result<(), Violation>;
}

/// Invariante definido por una función: la forma habitual de registrarlos en `INVARIANTS`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FnInvariant {
    #[allow(dead_code)] // sólo lo lee `Invariant::name` (ver ahí)
    name: &'static str,
    check: fn(&Sim) -> Result<(), Violation>,
}

impl FnInvariant {
    pub(crate) const fn new(name: &'static str, check: fn(&Sim) -> Result<(), Violation>) -> Self {
        Self { name, check }
    }
}

impl Invariant for FnInvariant {
    fn name(&self) -> &'static str {
        self.name
    }

    fn check(&self, sim: &Sim) -> Result<(), Violation> {
        (self.check)(sim)
    }
}

/// Mojón del último instante visto al terminar un paso (ver «Mojón de tiempo y hash de estado»).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Mark {
    time: SimTime,
}

impl Mark {
    /// Último instante registrado.
    pub(crate) const fn time(self) -> SimTime {
        self.time
    }

    /// Registra el instante al terminar un paso.
    pub(crate) fn record(&mut self, time: SimTime) {
        self.time = time;
    }
}

/// Los invariantes registrados por los módulos, en orden fijo: `sim`, planificador, eventos,
/// comandos.
pub(crate) fn all() -> impl Iterator<Item = &'static dyn Invariant> {
    let groups: [&'static [FnInvariant]; 4] = [
        crate::sim::INVARIANTS,
        crate::schedule::INVARIANTS,
        crate::events::INVARIANTS,
        crate::command::INVARIANTS,
    ];
    groups
        .into_iter()
        .flatten()
        .map(|invariant| -> &'static dyn Invariant { invariant })
}

/// Corre todos los invariantes y devuelve la primera violación (orden del registro).
pub(crate) fn check_all(sim: &Sim) -> Result<(), Violation> {
    all().try_for_each(|invariant| invariant.check(sim))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use proptest::prelude::*;

    use super::*;
    use crate::command::{Command, CommandError, CommandId};
    use crate::events::{Event, EventId, EventKind, Interrupt, Payload};
    use crate::ids::EntityId;
    use crate::scenario::Scenario;
    use crate::schedule::{Cadence, Priority, StepContext, System, SystemId};
    use crate::sim::AdvanceError;
    use crate::time::Increment;

    fn sim() -> Sim {
        Sim::new(Scenario::default(), 7).unwrap()
    }

    #[derive(Debug)]
    struct Mark1;

    impl Command for Mark1 {
        fn kind(&self) -> &'static str {
            "test.mark"
        }
        fn args(&self) -> Payload {
            Payload::new()
        }
        fn validate(&self, _: &Sim) -> Result<(), CommandError> {
            Ok(())
        }
        fn apply(&self, sim: &mut Sim) -> Result<(), CommandError> {
            let time = sim.time();
            sim.events_mut().emit(
                time,
                EventKind::ResearchCompleted,
                None,
                Interrupt::None,
                Payload::new(),
            )?;
            Ok(())
        }
    }

    #[derive(Debug)]
    struct Sys {
        id: &'static str,
        priority: u16,
        cadence: Cadence,
    }

    impl System for Sys {
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
            let _ = ctx.emit(
                EventKind::SurveyCompleted,
                None,
                Interrupt::None,
                Payload::new(),
            );
        }
    }

    fn with_systems(sim: &mut Sim) {
        for (id, priority, cadence) in [
            ("sys.dia", 2, Cadence::Daily),
            ("sys.sem", 1, Cadence::Weekly),
            ("sys.inc", 0, Cadence::EveryIncrement),
        ] {
            sim.scheduler_mut()
                .register(Box::new(Sys {
                    id,
                    priority,
                    cadence,
                }))
                .unwrap();
        }
    }

    /// Una simulación con historia: sistemas, comandos y 365 días de avance.
    fn lived_in() -> Sim {
        let mut sim = sim();
        with_systems(&mut sim);
        for day in 0..365 {
            if day % 10 == 0 {
                sim.submit_command(Box::new(Mark1)).unwrap();
            }
            sim.advance(Increment::D1).unwrap();
        }
        sim
    }

    fn expect(sim: &Sim, invariant: &str, entity: &str) -> Violation {
        let violation = sim.check_invariants().unwrap_err();
        assert_eq!(violation.invariant(), invariant, "{violation}");
        assert_eq!(violation.entity(), Some(entity), "{violation}");
        assert_ne!(violation.detail(), "");
        violation
    }

    #[test]
    fn el_registro_tiene_nombres_unicos_y_con_formato_dominio_regla() {
        let names: Vec<&str> = all().map(Invariant::name).collect();
        assert!(names.len() >= 10);
        let unique: BTreeSet<_> = names.iter().collect();
        assert_eq!(unique.len(), names.len(), "nombres repetidos: {names:?}");
        for name in &names {
            let (domain, rule) = name.split_once('.').unwrap();
            assert!(!domain.is_empty() && !rule.is_empty(), "{name}");
            assert!(
                name.chars()
                    .all(|c| c.is_ascii_lowercase() || c == '.' || c == '_'),
                "{name}"
            );
        }
        for required in [
            "time.monotonic",
            "events.sequential_ids",
            "events.causes_precede",
            "commands.ids_increasing",
        ] {
            assert!(names.contains(&required), "falta {required}");
        }
    }

    #[test]
    fn un_sim_recien_creado_cumple_los_invariantes() {
        assert_eq!(sim().check_invariants(), Ok(()));
    }

    #[test]
    fn tras_365_dias_con_comandos_y_sistemas_cumple_los_invariantes() {
        let sim = lived_in();
        assert!(!sim.events().is_empty() && !sim.command_history().is_empty());
        assert_eq!(sim.check_invariants(), Ok(()));
    }

    #[test]
    fn check_invariants_es_idempotente_y_no_cambia_el_hash() {
        let sim = lived_in();
        let before = sim.state_hash();
        assert_eq!(sim.check_invariants(), Ok(()));
        assert_eq!(sim.check_invariants(), Ok(()));
        assert_eq!(sim.state_hash(), before);
    }

    #[test]
    fn el_chequeo_automatico_no_cambia_el_hash_ni_el_mojon_entra_en_el_hash() {
        // Mismo estado simulado, distinto mojón: mismo hash.
        let a = lived_in();
        let mut b = lived_in();
        b.invariant_mark_mut().record(SimTime::ZERO);
        assert_ne!(a.invariant_mark().time(), b.invariant_mark().time());
        assert_eq!(a.state_hash(), b.state_hash());
    }

    #[test]
    fn el_mojon_sigue_el_tiempo_al_terminar_cada_paso() {
        let mut sim = sim();
        assert_eq!(sim.invariant_mark().time(), SimTime::ZERO);
        sim.advance(Increment::D5).unwrap();
        assert_eq!(sim.invariant_mark().time(), sim.time());
    }

    #[test]
    fn el_chequeo_cubre_el_pasado_tambien_tras_un_panico_de_apply() {
        #[derive(Debug)]
        struct Boom;
        impl Command for Boom {
            fn kind(&self) -> &'static str {
                "test.boom"
            }
            fn args(&self) -> Payload {
                Payload::new()
            }
            fn validate(&self, _: &Sim) -> Result<(), CommandError> {
                Ok(())
            }
            fn apply(&self, _: &mut Sim) -> Result<(), CommandError> {
                panic!("de prueba");
            }
        }
        let mut sim = sim();
        sim.submit_command(Box::new(Mark1)).unwrap();
        sim.submit_command(Box::new(Boom)).unwrap();
        let caught =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sim.advance(Increment::D1)));
        assert!(caught.is_err());
        // El estado «a medias» sigue cumpliendo los invariantes y el siguiente paso funciona.
        assert_eq!(sim.check_invariants(), Ok(()));
        sim.advance(Increment::D1).unwrap();
    }

    // ---- Violaciones forzadas: nombre exacto, entidad y mensaje ----

    #[test]
    fn tiempo_que_retrocede_viola_time_monotonic() {
        let mut sim = sim();
        sim.advance(Increment::D5).unwrap();
        sim.scheduler_mut()
            .set_time_for_test(SimTime::from_seconds(10));
        let v = expect(&sim, "time.monotonic", "sim.time");
        assert!(v.detail().contains("10"), "{v}");
        assert_eq!(
            v.to_string(),
            format!(
                "invariante «time.monotonic» violado en sim.time: {}",
                v.detail()
            )
        );
    }

    #[test]
    fn advance_devuelve_invariant_violated_si_se_corrompe_entre_pasos() {
        let mut sim = sim();
        sim.advance(Increment::D5).unwrap();
        // Retrocede apenas: tras un D1 el tiempo igual superaría al mojón; lo detecta el chequeo previo.
        sim.scheduler_mut()
            .set_time_for_test(SimTime::from_seconds(10));
        let before = sim.state_hash();
        let steps = sim.steps();
        let Err(AdvanceError::InvariantViolated(v)) = sim.advance(Increment::D30) else {
            panic!("debía fallar con InvariantViolated");
        };
        assert_eq!(v.invariant(), "time.monotonic");
        assert_eq!(v.entity(), Some("sim.time"));
        // El chequeo previo no toca nada.
        assert_eq!(sim.state_hash(), before);
        assert_eq!(sim.steps(), steps);
    }

    #[test]
    fn un_paso_que_deja_el_estado_mal_se_detecta_al_terminar() {
        #[derive(Debug)]
        struct Corrupt;
        impl Command for Corrupt {
            fn kind(&self) -> &'static str {
                "test.corrupt"
            }
            fn args(&self) -> Payload {
                Payload::new()
            }
            fn validate(&self, _: &Sim) -> Result<(), CommandError> {
                Ok(())
            }
            fn apply(&self, sim: &mut Sim) -> Result<(), CommandError> {
                // Un evento con una causa futura, metido a mano.
                let log = sim.events_mut();
                let time = SimTime::ZERO;
                let a = log
                    .emit(
                        time,
                        EventKind::LaunchFailed,
                        None,
                        Interrupt::None,
                        Payload::new(),
                    )
                    .unwrap();
                log.events_for_test()[0].cause = Some(EventId::from_raw(a.raw() + 5));
                Ok(())
            }
        }
        let mut sim = sim();
        sim.submit_command(Box::new(Corrupt)).unwrap();
        let Err(AdvanceError::InvariantViolated(v)) = sim.advance(Increment::D1) else {
            panic!("debía fallar con InvariantViolated");
        };
        assert_eq!(v.invariant(), "events.causes_precede");
        assert_eq!(v.entity(), Some("event#0"));
        assert!(v.to_string().contains("event#5"), "{v}");
        // El mojón no avanzó: el paso no se dio por bueno.
        assert_eq!(sim.invariant_mark().time(), SimTime::ZERO);
    }

    #[test]
    fn ids_de_evento_no_secuenciales_violan_events_sequential_ids() {
        let mut sim = lived_in();
        sim.events_mut().events_for_test()[3].id = EventId::from_raw(99);
        expect(&sim, "events.sequential_ids", "event#99");
    }

    #[test]
    fn causa_futura_en_id_viola_events_causes_precede() {
        let mut sim = lived_in();
        // El evento 3 cita como causa al 4 (un id mayor).
        sim.events_mut().events_for_test()[3].cause = Some(EventId::from_raw(4));
        let v = expect(&sim, "events.causes_precede", "event#3");
        assert!(v.detail().contains("event#4"), "{v}");
    }

    #[test]
    fn causa_posterior_en_el_tiempo_viola_events_causes_precede() {
        let mut sim = sim();
        let time = |s| SimTime::from_seconds(s);
        let log = sim.events_mut();
        let root = log
            .emit(
                time(0),
                EventKind::LaunchSucceeded,
                None,
                Interrupt::None,
                Payload::new(),
            )
            .unwrap();
        log.emit(
            time(0),
            EventKind::LaunchFailed,
            Some(root),
            Interrupt::None,
            Payload::new(),
        )
        .unwrap();
        sim.advance(Increment::S5).unwrap();
        assert_eq!(sim.check_invariants(), Ok(()));
        let events: &mut Vec<Event> = sim.events_mut().events_for_test();
        events[0].time = SimTime::from_seconds(3);
        events[1].time = SimTime::from_seconds(2);
        let v = expect(&sim, "events.causes_precede", "event#1");
        assert!(v.detail().contains("event#0"), "{v}");
    }

    #[test]
    fn evento_en_el_futuro_viola_events_not_in_future() {
        let mut sim = lived_in();
        let now = sim.time().as_seconds();
        sim.events_mut().events_for_test()[0].time = SimTime::from_seconds(now + 1);
        expect(&sim, "events.not_in_future", "event#0");
    }

    #[test]
    fn cursor_fuera_de_rango_viola_events_cursor_in_range() {
        let mut sim = lived_in();
        *sim.events_mut().cursor_for_test() = 1_000_000;
        expect(&sim, "events.cursor_in_range", "events.cursor");
    }

    #[test]
    fn ids_de_comando_desordenados_violan_commands_ids_increasing() {
        let mut sim = lived_in();
        let (_, history, _) = sim.commands_for_test();
        history[2].id = CommandId::from_raw(0);
        expect(&sim, "commands.ids_increasing", "command#0");
    }

    #[test]
    fn next_id_atrasado_viola_commands_next_id_above() {
        let mut sim = lived_in();
        let (_, _, next_id) = sim.commands_for_test();
        *next_id = 1;
        let v = expect(&sim, "commands.next_id_above", "commands.next_id");
        assert!(v.detail().contains('1'), "{v}");
    }

    #[test]
    fn id_pendiente_repetido_viola_commands_ids_increasing() {
        let mut sim = sim();
        sim.submit_command(Box::new(Mark1)).unwrap();
        sim.submit_command(Box::new(Mark1)).unwrap();
        let (pending, _, _) = sim.commands_for_test();
        pending[1].id = pending[0].id;
        expect(&sim, "commands.ids_increasing", "command#0");
    }

    #[test]
    fn historial_fuera_de_orden_viola_commands_history_ordered() {
        let mut sim = lived_in();
        let (_, history, _) = sim.commands_for_test();
        history[5].step = 0;
        expect(&sim, "commands.history_ordered", "command#5");
    }

    #[test]
    fn aplicado_antes_de_enviado_viola_commands_history_ordered() {
        let mut sim = lived_in();
        let (_, history, _) = sim.commands_for_test();
        history[1].submitted_at = SimTime::from_seconds(history[1].applied_at.as_seconds() + 1);
        expect(&sim, "commands.history_ordered", "command#1");
    }

    #[test]
    fn pendiente_enviado_en_el_futuro_viola_commands_pending_ordered() {
        let mut sim = sim();
        sim.submit_command(Box::new(Mark1)).unwrap();
        let (pending, _, _) = sim.commands_for_test();
        pending[0].submitted_at = SimTime::from_seconds(60);
        expect(&sim, "commands.pending_ordered", "command#0");
    }

    #[test]
    fn historial_con_pasos_de_mas_viola_sim_steps_cover_history() {
        let mut sim = lived_in();
        let steps = sim.steps();
        sim.set_steps_for_test(steps - 10);
        let v = expect(&sim, "sim.steps_cover_history", "sim.steps");
        assert!(v.detail().contains(&(steps - 10).to_string()), "{v}");
    }

    #[test]
    fn historial_aplicado_en_el_futuro_viola_sim_history_not_in_future() {
        let mut sim = lived_in();
        sim.scheduler_mut().set_time_for_test(SimTime::ZERO);
        sim.invariant_mark_mut().record(SimTime::ZERO);
        // Con el tiempo en 0, el comando aplicado el día 10 queda «en el futuro».
        expect(&sim, "sim.history_not_in_future", "command#1");
    }

    #[test]
    fn sistemas_con_el_mismo_id_violan_schedule_unique_ids() {
        let mut sim = sim();
        with_systems(&mut sim);
        // Mismo id con otra prioridad: pasa `register` (que sólo mira el id) sólo si se fuerza.
        sim.scheduler_mut().systems_for_test().push(Box::new(Sys {
            id: "sys.dia",
            priority: 9,
            cadence: Cadence::Daily,
        }));
        expect(&sim, "schedule.unique_ids", "system.sys.dia");
    }

    #[test]
    fn sistemas_desordenados_violan_schedule_order() {
        let mut sim = sim();
        with_systems(&mut sim);
        sim.scheduler_mut().systems_for_test().reverse();
        // Orden correcto: sys.inc (0), sys.sem (1), sys.dia (2); invertido, falla en el 2.º.
        expect(&sim, "schedule.order", "system.sys.sem");
    }

    #[test]
    fn params_del_planificador_distintos_a_los_del_sim_violan_schedule_params_match() {
        let mut sim = sim();
        let other = crate::params::Params::new(2, 3, 4).unwrap();
        sim.scheduler_mut().set_params_for_test(other);
        expect(&sim, "schedule.params_match_sim", "scheduler.params");
    }

    #[test]
    fn ni_new_ni_submit_command_chequean_pero_el_siguiente_advance_si() {
        let mut sim = sim();
        sim.advance(Increment::D1).unwrap();
        sim.scheduler_mut().set_time_for_test(SimTime::ZERO);
        // `submit_command` no corre invariantes: acepta aunque el estado esté corrupto...
        sim.submit_command(Box::new(Mark1)).unwrap();
        // ...y el chequeo previo del siguiente avance lo detecta sin aplicar el comando.
        assert!(matches!(
            sim.advance(Increment::S5),
            Err(AdvanceError::InvariantViolated(_))
        ));
        assert_eq!(sim.pending_command_count(), 1);
    }

    #[test]
    fn violacion_persistente_hace_fallar_los_avances_siguientes() {
        let mut sim = lived_in();
        sim.events_mut().events_for_test()[0].cause = Some(EventId::from_raw(0));
        for _ in 0..2 {
            assert!(matches!(
                sim.advance(Increment::S5),
                Err(AdvanceError::InvariantViolated(v)) if v.invariant() == "events.causes_precede"
            ));
        }
    }

    /// Una operación válida sobre el `Sim`.
    #[derive(Debug, Clone, Copy)]
    enum Op {
        Submit,
        Advance(Increment),
        Poll,
    }

    fn op() -> impl Strategy<Value = Op> {
        prop_oneof![
            2 => Just(Op::Submit),
            4 => proptest::sample::select(Increment::ALL.to_vec()).prop_map(Op::Advance),
            1 => Just(Op::Poll),
        ]
    }

    proptest! {
        #![proptest_config(crate::proptest_config::config())]

        /// Secuencias aleatorias de operaciones válidas nunca violan invariantes (y `advance`,
        /// que los chequea solo en tests, nunca devuelve `InvariantViolated`).
        #[test]
        fn operaciones_validas_nunca_violan_invariantes(
            seed in any::<u64>(),
            ops in proptest::collection::vec(op(), 0..40),
        ) {
            let mut sim = Sim::new(Scenario::default(), seed).unwrap();
            with_systems(&mut sim);
            for op in ops {
                match op {
                    Op::Submit => {
                        sim.submit_command(Box::new(Mark1)).unwrap();
                    }
                    Op::Advance(increment) => {
                        prop_assert!(sim.advance(increment).is_ok());
                    }
                    Op::Poll => {
                        let _ = sim.poll_events();
                    }
                }
                prop_assert_eq!(sim.check_invariants(), Ok(()));
            }
        }
    }

    #[test]
    fn violation_se_puede_mostrar_y_usar_como_error() {
        let v = Violation::new("a.b", None, "detalle");
        assert_eq!(v.to_string(), "invariante «a.b» violado: detalle");
        let boxed: Box<dyn std::error::Error> = Box::new(v);
        assert!(boxed.to_string().contains("a.b"));
    }
}
