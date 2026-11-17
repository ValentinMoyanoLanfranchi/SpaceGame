//! Estado raíz [`Sim`] y su API pública (doc 21 §2).
//!
//! `Sim` es **el** punto de entrada del núcleo: la CLI y la extensión de Godot sólo usan esta
//! API (y los tipos que reexporta `lib.rs`); lo demás es interno al crate. Reúne lo que ya
//! existe (semilla y RNG, planificador/tiempo, calendario, bus de eventos, cola e historial de
//! comandos) y **no** contiene estado del mundo: cuerpos, naves y demás llegan con la carga de
//! datos de M0.3+ y se agregan aquí sin cambiar la forma de la API.
//!
//! # Flujo
//!
//! 1. [`Sim::new`] con un [`Scenario`] y una semilla.
//! 2. [`Sim::submit_command`]: valida y encola; **no** aplica.
//! 3. [`Sim::advance`]: aplica los comandos pendientes (FIFO) y avanza el tiempo un
//!    [`Increment`].
//! 4. [`Sim::poll_events`]: eventos nuevos desde la última lectura; más los accesores de
//!    lectura ([`Sim::time`], [`Sim::date`], [`Sim::command_history`]...).
//!
//! # Provisional: [`Sim::advance`]
//!
//! La firma y la semántica de `advance` son **provisionales**. La tarea #27 (`M0.2.4.1`) la
//! cambia a `advance(Increment) -> AdvanceResult`: subdividirá el incremento según la cadencia
//! más fina necesaria y se detendrá en el primer evento interruptor, devolviendo el tiempo
//! realmente avanzado. Quien llame a `advance` hoy debe esperar ese cambio de tipo de retorno.

use thiserror::Error;

use crate::command::{
    Command, CommandError, CommandId, CommandOutcome, CommandQueue, CommandRecord,
};
use crate::events::{Event, EventLog};
use crate::rng::Rng;
use crate::scenario::Scenario;
use crate::schedule::Scheduler;
use crate::time::{Calendar, CalendarDate, CalendarError, Increment, SimTime};

/// Error de [`Sim::advance`]. Si ocurre, **no** se aplicó ningún comando ni cambió el estado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum AdvanceError {
    /// El avance desbordaría el tiempo de simulación representable (`u64` segundos).
    #[error("el avance desborda el tiempo de simulación")]
    TimeOverflow,
}

/// Estado raíz de la simulación.
///
/// # Ejemplo
///
/// Flujo típico: crear, enviar un comando, avanzar y leer.
///
/// ```
/// use heliarca_sim::{
///     Command, CommandError, CommandOutcome, Increment, Payload, Scenario, Sim, SimTime,
/// };
///
/// // Un comando de ejemplo que no hace nada. Los comandos del juego (investigar, construir...)
/// // se definen dentro de `heliarca-sim`; aquí sólo se muestra el flujo.
/// #[derive(Debug)]
/// struct Nada;
///
/// impl Command for Nada {
///     fn kind(&self) -> &'static str { "ejemplo.nada" }
///     fn args(&self) -> Payload { Payload::new() }
///     fn validate(&self, _sim: &Sim) -> Result<(), CommandError> { Ok(()) }
///     fn apply(&self, _sim: &mut Sim) -> Result<(), CommandError> { Ok(()) }
/// }
///
/// // 1. Crear: escenario y semilla determinan el inicio por completo.
/// let mut sim = Sim::new(Scenario::default(), 42);
/// assert_eq!(sim.time(), SimTime::ZERO);
/// assert_eq!(sim.seed(), 42);
/// assert_eq!(sim.date()?.to_string(), "Año 1, día 1, 00:00:00");
///
/// // 2. Enviar: se valida y se encola, pero todavía no se aplica.
/// let id = sim.submit_command(Box::new(Nada))?;
/// assert_eq!(sim.pending_command_count(), 1);
/// assert_eq!(sim.command_history(), []);
///
/// // 3. Avanzar: aplica los comandos pendientes y avanza el tiempo.
/// sim.advance(Increment::D1)?;
/// assert_eq!(sim.time(), SimTime::from_days(1).unwrap());
/// assert_eq!(sim.date()?.to_string(), "Año 1, día 2, 00:00:00");
/// assert_eq!(sim.command_history()[0].id, id);
/// assert_eq!(sim.command_history()[0].outcome, CommandOutcome::Applied);
///
/// // 4. Leer eventos: devuelve sólo los nuevos desde la lectura anterior. Este comando no
/// // emite ninguno (todavía no hay sistemas que los produzcan), así que no hay novedades.
/// assert_eq!(sim.poll_events(), []);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Si un `apply` entra en pánico
///
/// Un [`Command::apply`] **no debe** entrar en pánico (los fallos esperables son
/// [`CommandError`]). Si ocurre igualmente, el pánico atraviesa [`Sim::advance`] y el `Sim`
/// queda en un estado **consistente pero a medias**:
///
/// - el comando que entró en pánico **se pierde sin registro** en el historial, y también los
///   que seguían en la cola de ese paso (la cola se vació *antes* de aplicar);
/// - los comandos aplicados antes dentro del mismo paso **sí** quedaron en el historial, y sus
///   efectos permanecen (no hay deshacer);
/// - [`Sim::steps`] **no** avanza y el tiempo **no** avanza (el planificador corre después de
///   aplicar los comandos);
/// - los envíos hechos durante ese `apply` quedan pendientes para el paso siguiente.
///
/// Como un historial así ya no reproduce la partida, el pánico se considera un error de
/// programación: no hay recuperación prevista más allá de descartar el `Sim`.
///
/// # Qué ve un comando durante el paso
///
/// Mientras corre [`Command::apply`] (y un [`Command::validate`] llamado desde él, p. ej. al
/// enviar otro comando): la cola de pendientes está **vacía** salvo lo que se envíe durante
/// el propio paso; el historial contiene los comandos de pasos anteriores y los ya aplicados
/// en este paso, pero **no** el comando en curso (se registra al terminar su `apply`);
/// [`Sim::time`] es el del **comienzo** del paso (el tiempo avanza después de aplicar los
/// comandos) y [`Sim::steps`] todavía no se incrementó.
///
/// # Orden de eventos
///
/// El registro de eventos es **append-only y en orden de emisión**. Dentro de un
/// [`Sim::advance`], primero emiten los comandos (en orden de envío, con el tiempo del comienzo
/// del paso) y luego el planificador, cuyas corridas se ejecutan en orden cronológico (instante
/// de la frontera, luego prioridad e id). Por eso, lo que emitan los sistemas sigue el orden
/// del tiempo dentro del avance.
#[derive(Debug)]
pub struct Sim {
    rng: Rng,
    calendar: Calendar,
    scheduler: Scheduler,
    events: EventLog,
    commands: CommandQueue,
    /// Avances (pasos) ejecutados hasta ahora (clave para reproducir un historial de comandos).
    steps: u64,
}

impl Sim {
    /// Crea una simulación a partir de un escenario y una semilla, en [`SimTime::ZERO`].
    ///
    /// El mismo escenario con la misma semilla produce siempre la misma simulación.
    #[must_use]
    pub fn new(scenario: Scenario, seed: u64) -> Self {
        Self {
            rng: Rng::from_seed(seed),
            calendar: scenario.calendar(),
            scheduler: Scheduler::new(),
            events: EventLog::new(),
            commands: CommandQueue::default(),
            steps: 0,
        }
    }

    /// Semilla con la que se creó la simulación.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.rng.seed()
    }

    /// Instante de simulación actual.
    #[must_use]
    pub const fn time(&self) -> SimTime {
        self.scheduler.time()
    }

    /// Calendario del escenario.
    #[must_use]
    pub const fn calendar(&self) -> Calendar {
        self.calendar
    }

    /// Fecha de calendario del instante actual.
    ///
    /// # Errors
    ///
    /// [`CalendarError::Overflow`] si fecha de inicio + tiempo no cabe en `u64` segundos.
    pub fn date(&self) -> Result<CalendarDate, CalendarError> {
        self.calendar.date_at(self.time())
    }

    /// Avances ejecutados con [`Sim::advance`].
    #[must_use]
    pub const fn steps(&self) -> u64 {
        self.steps
    }

    /// Registro de eventos completo (lectura): todo lo emitido desde el inicio.
    ///
    /// Para consumir sólo las novedades, usar [`Sim::poll_events`].
    #[must_use]
    pub const fn events(&self) -> &EventLog {
        &self.events
    }

    /// Eventos emitidos desde la llamada anterior (todos, en la primera), en orden de emisión.
    /// Avanza el cursor: no repite eventos y, sin novedades, devuelve un slice vacío.
    ///
    /// El slice toma prestado el `Sim`: copiar lo necesario (p. ej. a un modelo de vista) y
    /// soltarlo antes de seguir usando la simulación.
    pub fn poll_events(&mut self) -> &[Event] {
        self.events.poll_events()
    }

    /// Registro de eventos (escritura): para los comandos al aplicarse. Hoy sólo lo usan los
    /// tests; cuando haya comandos reales (M0.3+) se quita el `cfg(test)`.
    #[cfg(test)]
    pub(crate) fn events_mut(&mut self) -> &mut EventLog {
        &mut self.events
    }

    /// Valida un comando y, si es válido, lo encola. **No lo aplica**: se aplicará al comienzo
    /// del siguiente [`Sim::advance`], en orden de envío (FIFO).
    ///
    /// Un comando inválido no deja rastro: no entra en la cola ni en el historial, no consume
    /// un [`CommandId`] y no toca el estado.
    ///
    /// # Errors
    ///
    /// El [`CommandError`] de [`Command::validate`], o [`CommandError::IdsExhausted`] si ya no
    /// quedan ids de comando.
    pub fn submit_command(&mut self, command: Box<dyn Command>) -> Result<CommandId, CommandError> {
        command.validate(self)?;
        let queued = self.commands.prepare(command, self.time())?;
        let id = queued.id;
        self.commands.push(queued);
        Ok(id)
    }

    /// Cantidad de comandos aceptados que aún esperan su paso.
    #[must_use]
    pub fn pending_command_count(&self) -> usize {
        self.commands.pending_len()
    }

    /// Historial append-only (sólo lectura) de los comandos ya procesados, en orden de
    /// aplicación.
    #[must_use]
    pub fn command_history(&self) -> &[CommandRecord] {
        self.commands.history()
    }

    /// Avanza un incremento: aplica los comandos pendientes al comienzo (en orden de envío) y
    /// luego avanza el planificador. Cuenta un paso ([`Sim::steps`]).
    ///
    /// **Provisional** (ver la documentación del módulo): #27 lo reemplaza por
    /// `advance(Increment) -> AdvanceResult`, con subdivisión del incremento e interrupciones.
    /// Hoy avanza siempre el incremento completo.
    ///
    /// Si un `apply` falla se registra el error en el historial y se **continúa** con el resto.
    /// Para el comportamiento ante un pánico y lo que ve cada comando durante el paso, ver la
    /// documentación de [`Sim`].
    ///
    /// # Errors
    ///
    /// [`AdvanceError::TimeOverflow`] si el incremento desborda el tiempo. Se comprueba
    /// **antes** de tocar nada: no se aplica ningún comando y el estado no cambia.
    pub fn advance(&mut self, increment: Increment) -> Result<(), AdvanceError> {
        self.time()
            .checked_advance(increment)
            .ok_or(AdvanceError::TimeOverflow)?;
        self.apply_pending_commands();
        // Con el desborde ya descartado, el planificador no tiene otra forma de fallar.
        self.scheduler
            .advance(increment)
            .map_err(|_| AdvanceError::TimeOverflow)
    }

    /// Aplica, en orden de envío, los comandos encolados antes de este paso.
    ///
    /// Se vacía la cola con [`std::mem::take`] *antes* de aplicar: así `apply(&self, &mut Sim)`
    /// recibe el `Sim` completo sin préstamos pendientes sobre la cola, y un comando que envíe
    /// otro durante su `apply` lo deja para el paso siguiente (nunca se reentra).
    ///
    /// Si un `apply` falla se registra el error en el historial y se **continúa** con el resto.
    /// Si entra en pánico, ver la documentación de [`Sim`].
    fn apply_pending_commands(&mut self) {
        let step = self.steps;
        for queued in self.commands.take_pending() {
            let outcome = match queued.command.apply(self) {
                Ok(()) => CommandOutcome::Applied,
                Err(error) => CommandOutcome::Failed(error),
            };
            let applied_at = self.time();
            self.commands
                .record(CommandRecord::new(queued, step, applied_at, outcome));
        }
        self.steps = self.steps.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::*;
    use crate::events::{EventKind, Interrupt, Payload, Value};
    use crate::ids::EntityId;

    fn sim(seed: u64) -> Sim {
        Sim::new(Scenario::default(), seed)
    }

    /// Comando de prueba: emite un evento con `n`.
    #[derive(Debug)]
    struct Mark(u64);

    impl Command for Mark {
        fn kind(&self) -> &'static str {
            "test.mark"
        }
        fn args(&self) -> Payload {
            Payload::new().with("n", Value::UInt(self.0))
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
                Payload::new().with("n", Value::UInt(self.0)),
            )?;
            Ok(())
        }
    }

    /// Comando de prueba: emite lo que ve del `Sim` durante su `apply`.
    #[derive(Debug)]
    struct Probe;

    impl Command for Probe {
        fn kind(&self) -> &'static str {
            "test.probe"
        }
        fn args(&self) -> Payload {
            Payload::new()
        }
        fn validate(&self, _: &Sim) -> Result<(), CommandError> {
            Ok(())
        }
        fn apply(&self, sim: &mut Sim) -> Result<(), CommandError> {
            let pending = u64::try_from(sim.pending_command_count()).unwrap();
            let history = u64::try_from(sim.command_history().len()).unwrap();
            let payload = Payload::new()
                .with("pending", Value::UInt(pending))
                .with("history", Value::UInt(history))
                .with("steps", Value::UInt(sim.steps()))
                .with("time", Value::UInt(sim.time().as_seconds()));
            let time = sim.time();
            sim.events_mut().emit(
                time,
                EventKind::SurveyCompleted,
                None,
                Interrupt::None,
                payload,
            )?;
            Ok(())
        }
    }

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
            panic!("apply de prueba que entra en pánico");
        }
    }

    fn ns(sim: &mut Sim) -> Vec<Option<Value>> {
        sim.poll_events()
            .iter()
            .map(|e| e.payload.get("n").cloned())
            .collect()
    }

    #[test]
    fn crear_con_escenario_y_semilla() {
        let calendar = Calendar::new(CalendarDate::at_day_start(7, 200)).unwrap();
        let sim = Sim::new(Scenario::new().with_calendar(calendar), 1234);
        assert_eq!(sim.seed(), 1234);
        assert_eq!(sim.time(), SimTime::ZERO);
        assert_eq!(sim.steps(), 0);
        assert_eq!(sim.calendar(), calendar);
        assert_eq!(sim.date().unwrap(), CalendarDate::at_day_start(7, 200));
        assert!(sim.events().is_empty());
        assert_eq!(sim.command_history(), []);
        assert_eq!(sim.pending_command_count(), 0);
    }

    #[test]
    fn advance_avanza_el_tiempo_la_fecha_y_cuenta_pasos() {
        let mut sim = sim(1);
        sim.advance(Increment::D5).unwrap();
        sim.advance(Increment::H1).unwrap();
        assert_eq!(sim.time(), SimTime::from_seconds(5 * 86_400 + 3_600));
        assert_eq!(sim.steps(), 2);
        let date = sim.date().unwrap();
        assert_eq!((date.year, date.day, date.hour), (1, 6, 1));
    }

    #[test]
    fn poll_events_entrega_solo_las_novedades() {
        let mut sim = sim(1);
        assert_eq!(sim.poll_events(), []);
        sim.submit_command(Box::new(Mark(1))).unwrap();
        sim.submit_command(Box::new(Mark(2))).unwrap();
        sim.advance(Increment::S5).unwrap();
        assert_eq!(ns(&mut sim), [Some(Value::UInt(1)), Some(Value::UInt(2))]);
        assert_eq!(sim.poll_events(), [], "no se repiten");
        assert_eq!(sim.events().len(), 2, "el log completo sigue disponible");
        sim.submit_command(Box::new(Mark(3))).unwrap();
        sim.advance(Increment::S5).unwrap();
        assert_eq!(ns(&mut sim), [Some(Value::UInt(3))]);
    }

    #[test]
    fn mismo_escenario_y_semilla_dan_el_mismo_resultado() {
        let run = |seed| {
            let mut sim = sim(seed);
            sim.submit_command(Box::new(Mark(1))).unwrap();
            sim.advance(Increment::D1).unwrap();
            sim.advance(Increment::H1).unwrap();
            (
                sim.time(),
                sim.steps(),
                sim.events().iter().cloned().collect::<Vec<_>>(),
                sim.command_history().to_vec(),
            )
        };
        assert_eq!(run(5), run(5));
    }

    #[test]
    fn un_comando_ve_cola_vacia_e_historial_sin_el_comando_en_curso() {
        let mut sim = sim(1);
        sim.advance(Increment::H1).unwrap(); // paso 0: tiempo 3600, sin comandos
        sim.submit_command(Box::new(Mark(1))).unwrap();
        sim.submit_command(Box::new(Probe)).unwrap();
        sim.advance(Increment::D1).unwrap(); // paso 1
        let probe = sim
            .events()
            .iter()
            .find(|e| e.kind == EventKind::SurveyCompleted)
            .unwrap();
        assert_eq!(probe.payload.get("pending"), Some(&Value::UInt(0)));
        // Sólo `Mark(1)` está en el historial; el propio `Probe` todavía no.
        assert_eq!(probe.payload.get("history"), Some(&Value::UInt(1)));
        assert_eq!(probe.payload.get("steps"), Some(&Value::UInt(1)));
        // Tiempo del comienzo del paso, no el posterior al avance.
        assert_eq!(probe.payload.get("time"), Some(&Value::UInt(3_600)));
        assert_eq!(sim.command_history().len(), 2);
        assert_eq!(sim.time(), SimTime::from_seconds(3_600 + 86_400));
    }

    #[test]
    fn un_apply_que_entra_en_panico_pierde_el_resto_del_paso_sin_registro() {
        let mut sim = sim(1);
        sim.submit_command(Box::new(Mark(1))).unwrap();
        sim.submit_command(Box::new(Boom)).unwrap();
        sim.submit_command(Box::new(Mark(2))).unwrap();

        let result = catch_unwind(AssertUnwindSafe(|| sim.advance(Increment::D1)));
        assert!(result.is_err(), "el pánico atraviesa advance");

        // El anterior al pánico quedó aplicado y registrado.
        assert_eq!(sim.command_history().len(), 1);
        assert_eq!(sim.command_history()[0].kind, "test.mark");
        assert_eq!(sim.events().len(), 1);
        // El que entró en pánico y el siguiente se perdieron sin registro.
        assert_eq!(sim.pending_command_count(), 0);
        // El paso no se contó y el tiempo no avanzó.
        assert_eq!(sim.steps(), 0);
        assert_eq!(sim.time(), SimTime::ZERO);
    }

    #[test]
    fn el_log_de_eventos_es_en_orden_de_emision() {
        let mut sim = sim(1);
        sim.submit_command(Box::new(Mark(2))).unwrap();
        sim.submit_command(Box::new(Mark(1))).unwrap();
        sim.advance(Increment::S5).unwrap();
        let ids: Vec<u32> = sim.events().iter().map(|e| e.id.raw()).collect();
        assert_eq!(ids, [0, 1]);
        assert_eq!(ns(&mut sim), [Some(Value::UInt(2)), Some(Value::UInt(1))]);
    }
}
