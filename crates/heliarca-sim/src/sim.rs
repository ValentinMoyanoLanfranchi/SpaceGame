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
//! 3. [`Sim::advance`]: aplica los comandos pendientes (FIFO) y avanza el tiempo hasta un
//!    [`Increment`], deteniéndose en la primera interrupción.
//! 4. [`Sim::poll_events`]: eventos nuevos desde la última lectura; más los accesores de
//!    lectura ([`Sim::time`], [`Sim::date`], [`Sim::command_history`]...).
//!
//! # Avance e interrupciones
//!
//! [`Sim::advance`] avanza un [`Increment`] **por tramos**: hasta la próxima frontera de
//! cadencia de los sistemas registrados (o el final del incremento) y, tras cada tramo, mira
//! si algún sistema emitió un evento con [`Interrupt`] distinto de `None`. Si es así se
//! detiene **en ese instante** y devuelve un [`AdvanceResult`] con el tiempo realmente
//! avanzado y el evento que lo detuvo, que sigue disponible en [`Sim::poll_events`] (doc 01,
//! «cualquier evento marcado como relevante detiene el avance»). Ver [`Sim::advance`] para las
//! reglas exactas.

use thiserror::Error;

use crate::command::{
    Command, CommandError, CommandId, CommandOutcome, CommandQueue, CommandRecord,
};
use crate::events::{Event, EventId, EventLog, Interrupt};
use crate::rng::Rng;
use crate::scenario::Scenario;
use crate::schedule::Scheduler;
use crate::state_hash::{StateHash, hash_sim};
use crate::time::{Calendar, CalendarDate, CalendarError, Increment, SimTime};

/// Error de [`Sim::advance`]. Si ocurre, **no** se aplicó ningún comando ni cambió el estado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum AdvanceError {
    /// El avance desbordaría el tiempo de simulación representable (`u64` segundos).
    #[error("el avance desborda el tiempo de simulación")]
    TimeOverflow,
}

/// Resultado de un [`Sim::advance`] que se llevó a cabo.
///
/// # Ejemplo
///
/// ```
/// use heliarca_sim::{Increment, Scenario, Sim};
///
/// let mut sim = Sim::new(Scenario::default(), 1);
/// let result = sim.advance(Increment::D30)?;
/// // Sin sistemas que emitan interrupciones, se avanza el incremento completo.
/// assert!(result.completed());
/// assert_eq!(result.advanced, 30 * 86_400);
/// assert_eq!(result.interrupted_by, None);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct AdvanceResult {
    /// Incremento pedido.
    pub requested: Increment,
    /// Tiempo realmente avanzado, en segundos de simulación (nunca más que `requested`).
    pub advanced: u64,
    /// Primer evento con [`Interrupt`] distinto de `None` emitido por un sistema durante el
    /// avance, en orden de emisión; `None` si no hubo. Es un id del registro de eventos
    /// ([`Sim::events`]); el evento también llega por [`Sim::poll_events`].
    pub interrupted_by: Option<EventId>,
}

impl AdvanceResult {
    /// `true` si se avanzó el incremento completo.
    ///
    /// Es independiente de [`AdvanceResult::interrupted_by`]: una interrupción que ocurre
    /// justo en el último instante del incremento no recorta nada, así que `completed()` sigue
    /// siendo `true` aunque `interrupted_by` sea `Some`.
    #[must_use]
    pub const fn completed(&self) -> bool {
        self.advanced >= self.requested.seconds()
    }
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
/// let result = sim.advance(Increment::D1)?;
/// assert!(result.completed());
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
    /// Avances (llamadas a `advance` que se llevaron a cabo) hasta ahora (clave para reproducir un historial de comandos).
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

    /// Hash estable de 128 bits de **todo el estado simulado**: semilla y estado del RNG, pasos,
    /// calendario, tiempo y sistemas, log de eventos, cola e historial de comandos.
    ///
    /// Es la base de la repetibilidad: con el mismo escenario y semilla, las mismas operaciones
    /// dan el mismo hash en cada paso, en cualquier plataforma y ejecución. Se calcula sobre una
    /// serialización canónica de formato fijo con FNV-1a de 128 bits (no aleatorio) y **no**
    /// modifica el `Sim`. No es criptográfico. Recorre todo el estado, así que cuesta O(tamaño):
    /// es para tests y puntos de control, no para cada cuadro. Qué entra y qué no (por ejemplo,
    /// el cursor de [`Sim::poll_events`] no) y cuándo cambia el formato: ver el módulo
    /// interno `state_hash`.
    ///
    /// # Ejemplo
    ///
    /// ```
    /// use heliarca_sim::{Increment, Scenario, Sim};
    ///
    /// let run = |seed| -> Result<_, Box<dyn std::error::Error>> {
    ///     let mut sim = Sim::new(Scenario::default(), seed);
    ///     sim.advance(Increment::D5)?;
    ///     sim.advance(Increment::H1)?;
    ///     Ok(sim)
    /// };
    ///
    /// // Mismas operaciones, misma semilla: mismo hash (también entre ejecuciones y plataformas).
    /// let (a, b) = (run(42)?, run(42)?);
    /// assert_eq!(a.state_hash(), b.state_hash());
    /// // Llamarlo no cambia nada.
    /// assert_eq!(a.state_hash(), a.state_hash());
    ///
    /// // Otra semilla, o una operación más: otro hash.
    /// assert_ne!(a.state_hash(), run(43)?.state_hash());
    /// let mut c = run(42)?;
    /// c.advance(Increment::S5)?;
    /// assert_ne!(a.state_hash(), c.state_hash());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn state_hash(&self) -> StateHash {
        hash_sim(
            &self.rng,
            self.calendar,
            &self.scheduler,
            &self.events,
            &self.commands,
            self.steps,
        )
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

    /// Avances ejecutados con [`Sim::advance`]: uno por llamada que no devolvió error, aunque
    /// se haya detenido por una interrupción o los tramos internos sean muchos.
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

    /// RNG raíz (lectura): para derivar subflujos de sistemas de prueba. Los sistemas reales
    /// recibirán su subflujo por el contexto de paso cuando existan (M0.3+).
    #[cfg(test)]
    pub(crate) fn rng(&self) -> &Rng {
        &self.rng
    }

    /// RNG raíz (escritura): para que un comando o sistema de prueba consuma el flujo.
    #[cfg(test)]
    pub(crate) fn rng_mut(&mut self) -> &mut Rng {
        &mut self.rng
    }

    /// Registro de eventos (escritura): para los comandos al aplicarse. Hoy sólo lo usan los
    /// tests; cuando haya comandos reales (M0.3+) se quita el `cfg(test)`.
    #[cfg(test)]
    pub(crate) fn events_mut(&mut self) -> &mut EventLog {
        &mut self.events
    }

    /// Planificador (escritura): para registrar sistemas de prueba. Los sistemas reales los
    /// registrará la construcción del `Sim` cuando existan (M0.3+).
    #[cfg(test)]
    pub(crate) fn scheduler_mut(&mut self) -> &mut Scheduler {
        &mut self.scheduler
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

    /// Avanza hasta un [`Increment`]: aplica los comandos pendientes al comienzo (en orden de
    /// envío) y luego avanza el planificador por tramos, deteniéndose en la primera
    /// interrupción.
    ///
    /// # Tramos
    ///
    /// El incremento se parte en tramos que terminan en la próxima frontera de cadencia de los
    /// sistemas registrados (o en el final del incremento, lo que ocurra antes). Así un D30 con
    /// un sistema diario son 30 tramos; sin sistemas periódicos, uno. Partir el avance no
    /// cambia qué corre ni en qué orden (ver el módulo del planificador).
    ///
    /// # Interrupciones
    ///
    /// Tras cada tramo se mira si un sistema emitió un evento con [`Interrupt`] distinto de
    /// `None` (`Halt` o `SlowDown`). Si hay alguno, el avance se detiene **en el instante de
    /// ese tramo** (el `time` del evento: la frontera donde lo emitió el sistema) y no más
    /// allá:
    ///
    /// - Si varios sistemas emiten interrupciones en el mismo instante, todos corren (el
    ///   instante se completa), todos quedan en el registro y [`AdvanceResult::interrupted_by`]
    ///   es el primero **en orden de emisión**.
    /// - Si la interrupción cae exactamente al final del incremento, [`AdvanceResult::completed`]
    ///   es `true` y `interrupted_by` es `Some`.
    /// - Los sistemas `Cadence::EveryIncrement`
    ///   corren una sola vez por llamada, al final (en el instante donde se detuvo, si hubo
    ///   interrupción); lo que emitan también puede interrumpir.
    /// - Los eventos de los **comandos** no interrumpen: son consecuencia de lo que el jugador
    ///   acaba de ordenar. Sólo cuentan los emitidos por sistemas.
    /// - `SlowDown` detiene igual que `Halt`: la ralentización automática y la ventana de
    ///   decisión (doc 01, doc 16) son del control de velocidad de la interfaz, que todavía no
    ///   existe; mientras tanto, detenerse es lo seguro (quien llama puede seguir con un
    ///   incremento menor). Queda como decisión pendiente.
    ///
    /// El evento interruptor queda disponible en [`Sim::poll_events`]; un avance posterior
    /// continúa desde donde quedó.
    ///
    /// # Pasos
    ///
    /// Cada llamada que no devuelve error cuenta **un** paso ([`Sim::steps`]) y aplica los
    /// comandos pendientes **una sola vez**, al comienzo, con el mismo `step` en el historial,
    /// sin importar cuántos tramos tenga ni si se interrumpió. Reproducir una partida es
    /// repetir las mismas llamadas con los mismos incrementos.
    ///
    /// Si un `apply` falla se registra el error en el historial y se **continúa** con el resto.
    /// Para el comportamiento ante un pánico y lo que ve cada comando durante el paso, ver la
    /// documentación de [`Sim`].
    ///
    /// # Errors
    ///
    /// [`AdvanceError::TimeOverflow`] si el incremento desborda el tiempo. Se comprueba
    /// **antes** de tocar nada: no se aplica ningún comando y el estado no cambia.
    pub fn advance(&mut self, increment: Increment) -> Result<AdvanceResult, AdvanceError> {
        let start = self.time();
        let target = start
            .checked_advance(increment)
            .ok_or(AdvanceError::TimeOverflow)?;
        self.apply_pending_commands();
        self.steps = self.steps.saturating_add(1);

        // Sólo cuentan los eventos que emitan los sistemas, no los de los comandos.
        let first_system_event = self.events.len();
        loop {
            let next = self.scheduler.next_boundary(self.time(), target);
            self.scheduler.advance_to(next, &mut self.events);
            if next >= target || self.first_interrupt_since(first_system_event).is_some() {
                break;
            }
        }
        // Una sola vez por avance, al final (o donde se detuvo).
        self.scheduler.run_every_increment(&mut self.events);
        let interrupted_by = self.first_interrupt_since(first_system_event);

        Ok(AdvanceResult {
            requested: increment,
            advanced: self.time().as_seconds().saturating_sub(start.as_seconds()),
            interrupted_by,
        })
    }

    /// Primer evento interruptor (en orden de emisión) entre los emitidos desde el índice
    /// `first`.
    fn first_interrupt_since(&self, first: usize) -> Option<EventId> {
        self.events
            .iter()
            .skip(first)
            .find(|event| event.interrupt != Interrupt::None)
            .map(|event| event.id)
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

    // ---- Avance por tramos e interrupciones ----

    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::schedule::{Cadence, Priority, StepContext, System, SystemId};

    const DAY: u64 = 86_400;

    type Runs = Rc<RefCell<Vec<(&'static str, u64)>>>;

    /// Sistema de prueba: registra cada corrida `(id, instante)` y, en los instantes
    /// indicados, emite un evento con la interrupción dada.
    #[derive(Debug)]
    struct Scripted {
        id: &'static str,
        priority: u16,
        cadence: Cadence,
        fire: Vec<(u64, Interrupt)>,
        runs: Runs,
    }

    impl System for Scripted {
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
            for &(at, interrupt) in &self.fire {
                if at == now {
                    ctx.emit(
                        EventKind::JumpPointCrossed,
                        None,
                        interrupt,
                        Payload::new().with("system", Value::Text(self.id.into())),
                    )
                    .unwrap();
                }
            }
        }
    }

    fn add(
        sim: &mut Sim,
        id: &'static str,
        priority: u16,
        cadence: Cadence,
        fire: &[(u64, Interrupt)],
    ) -> Runs {
        let runs = Runs::default();
        sim.scheduler_mut()
            .register(Box::new(Scripted {
                id,
                priority,
                cadence,
                fire: fire.to_vec(),
                runs: Rc::clone(&runs),
            }))
            .unwrap();
        runs
    }

    fn count(runs: &Runs, id: &str) -> usize {
        runs.borrow().iter().filter(|r| r.0 == id).count()
    }

    #[test]
    fn un_halt_a_los_3_dias_detiene_un_avance_de_30() {
        let mut sim = sim(1);
        let runs = add(&mut sim, "dia", 5, Cadence::Daily, &[]);
        add(
            &mut sim,
            "alerta",
            0,
            Cadence::Daily,
            &[(3 * DAY, Interrupt::Halt)],
        );
        let result = sim.advance(Increment::D30).unwrap();
        assert_eq!(result.advanced, 3 * DAY);
        assert_eq!(result.requested, Increment::D30);
        assert!(!result.completed());
        assert_eq!(sim.time(), SimTime::from_seconds(3 * DAY));
        // El avance no pasó del instante del evento: el diario corrió 3 veces, no 30.
        assert_eq!(count(&runs, "dia"), 3);
        let events = sim.poll_events().to_vec();
        assert_eq!(events.len(), 1);
        assert_eq!(Some(events[0].id), result.interrupted_by);
        assert_eq!(events[0].time, SimTime::from_seconds(3 * DAY));
        assert_eq!(events[0].interrupt, Interrupt::Halt);
    }

    #[test]
    fn sin_eventos_relevantes_se_avanza_el_incremento_completo() {
        let mut sim = sim(1);
        let runs = add(&mut sim, "dia", 0, Cadence::Daily, &[]);
        let result = sim.advance(Increment::D30).unwrap();
        assert_eq!(result.advanced, 30 * DAY);
        assert!(result.completed());
        assert_eq!(result.interrupted_by, None);
        assert_eq!(count(&runs, "dia"), 30);
        assert_eq!(sim.poll_events(), []);
    }

    #[test]
    fn sin_sistemas_se_avanza_el_incremento_completo() {
        for inc in Increment::ALL {
            let mut sim = sim(1);
            let result = sim.advance(inc).unwrap();
            assert_eq!(result.advanced, inc.seconds());
            assert!(result.completed());
            assert_eq!(sim.time(), SimTime::from_seconds(inc.seconds()));
        }
    }

    #[test]
    fn un_evento_no_relevante_no_detiene() {
        let mut sim = sim(1);
        add(
            &mut sim,
            "aviso",
            0,
            Cadence::Daily,
            &[(3 * DAY, Interrupt::None)],
        );
        let result = sim.advance(Increment::D30).unwrap();
        assert!(result.completed());
        assert_eq!(result.interrupted_by, None);
        assert_eq!(sim.poll_events().len(), 1);
    }

    #[test]
    fn slow_down_detiene_como_halt() {
        let mut sim = sim(1);
        add(
            &mut sim,
            "hostil",
            0,
            Cadence::Daily,
            &[(2 * DAY, Interrupt::SlowDown)],
        );
        let result = sim.advance(Increment::D5).unwrap();
        assert_eq!(result.advanced, 2 * DAY);
        let id = result.interrupted_by.unwrap();
        assert_eq!(sim.events().get(id).unwrap().interrupt, Interrupt::SlowDown);
    }

    #[test]
    fn la_primera_interrupcion_gana_y_las_posteriores_no_ocurren() {
        let mut sim = sim(1);
        add(
            &mut sim,
            "a",
            0,
            Cadence::Daily,
            &[(4 * DAY, Interrupt::Halt), (2 * DAY, Interrupt::Halt)],
        );
        let result = sim.advance(Increment::D30).unwrap();
        assert_eq!(result.advanced, 2 * DAY);
        assert_eq!(sim.events().len(), 1);
    }

    #[test]
    fn interrupcion_en_el_primer_tramo_de_un_incremento_corto() {
        // Un sistema por incremento que interrumpe en el primer (y único) tramo de S5.
        let mut sim = sim(1);
        add(
            &mut sim,
            "inc",
            0,
            Cadence::EveryIncrement,
            &[(5, Interrupt::Halt)],
        );
        let result = sim.advance(Increment::S5).unwrap();
        assert_eq!(result.advanced, 5);
        assert!(
            result.completed(),
            "justo al final: completo e interrumpido"
        );
        assert!(result.interrupted_by.is_some());
    }

    #[test]
    fn interrupcion_en_el_primer_dia_recorta_un_d5() {
        let mut sim = sim(1);
        add(
            &mut sim,
            "alerta",
            0,
            Cadence::Daily,
            &[(DAY, Interrupt::Halt)],
        );
        let result = sim.advance(Increment::D5).unwrap();
        assert_eq!(result.advanced, DAY);
        assert!(!result.completed());
    }

    #[test]
    fn interrupcion_justo_al_final_del_incremento_es_completa_e_interrumpida() {
        let mut sim = sim(1);
        add(
            &mut sim,
            "alerta",
            0,
            Cadence::Daily,
            &[(5 * DAY, Interrupt::Halt)],
        );
        let result = sim.advance(Increment::D5).unwrap();
        assert_eq!(result.advanced, 5 * DAY);
        assert!(result.completed());
        assert!(result.interrupted_by.is_some());
    }

    #[test]
    fn empate_en_el_mismo_instante_deja_todo_en_el_log_y_gana_el_primero_emitido() {
        let mut sim = sim(1);
        // Misma frontera; por prioridad corre primero "b" (0) y luego "a" (1).
        add(
            &mut sim,
            "a",
            1,
            Cadence::Daily,
            &[(2 * DAY, Interrupt::Halt)],
        );
        add(
            &mut sim,
            "b",
            0,
            Cadence::Daily,
            &[(2 * DAY, Interrupt::SlowDown)],
        );
        let result = sim.advance(Increment::D30).unwrap();
        assert_eq!(result.advanced, 2 * DAY);
        let events = sim.poll_events().to_vec();
        assert_eq!(events.len(), 2, "el instante se completa");
        assert_eq!(result.interrupted_by, Some(events[0].id));
        assert_eq!(
            events[0].payload.get("system"),
            Some(&Value::Text("b".into()))
        );
        assert_eq!(
            events[1].payload.get("system"),
            Some(&Value::Text("a".into()))
        );
    }

    #[test]
    fn un_avance_posterior_continua_desde_donde_quedo() {
        let mut sim = sim(1);
        let runs = add(
            &mut sim,
            "alerta",
            0,
            Cadence::Daily,
            &[(3 * DAY, Interrupt::Halt), (10 * DAY, Interrupt::Halt)],
        );
        let first = sim.advance(Increment::D30).unwrap();
        assert_eq!(first.advanced, 3 * DAY);
        let second = sim.advance(Increment::D30).unwrap();
        assert_eq!(second.advanced, 7 * DAY, "hasta el día 10");
        assert_eq!(sim.time(), SimTime::from_seconds(10 * DAY));
        let third = sim.advance(Increment::D30).unwrap();
        assert!(third.completed());
        assert_eq!(third.interrupted_by, None);
        assert_eq!(sim.time(), SimTime::from_seconds(40 * DAY));
        // Ninguna frontera se corrió dos veces ni se salteó.
        assert_eq!(count(&runs, "alerta"), 40);
        assert_eq!(sim.events().len(), 2);
    }

    #[test]
    fn el_sistema_por_incremento_corre_una_vez_por_avance_donde_termina() {
        let mut sim = sim(1);
        let runs = add(&mut sim, "inc", 0, Cadence::EveryIncrement, &[]);
        add(
            &mut sim,
            "alerta",
            0,
            Cadence::Daily,
            &[(3 * DAY, Interrupt::Halt)],
        );
        sim.advance(Increment::D30).unwrap();
        sim.advance(Increment::S5).unwrap();
        assert_eq!(
            *runs.borrow(),
            [("inc", 3 * DAY), ("inc", 3 * DAY + 5)],
            "una vez por llamada, en el instante donde terminó"
        );
    }

    #[test]
    fn un_evento_interruptor_de_un_comando_no_detiene() {
        #[derive(Debug)]
        struct HaltCmd;
        impl Command for HaltCmd {
            fn kind(&self) -> &'static str {
                "test.halt"
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
                    EventKind::LaunchFailed,
                    None,
                    Interrupt::Halt,
                    Payload::new(),
                )?;
                Ok(())
            }
        }
        let mut sim = sim(1);
        sim.submit_command(Box::new(HaltCmd)).unwrap();
        let result = sim.advance(Increment::D1).unwrap();
        assert!(result.completed());
        assert_eq!(result.interrupted_by, None);
        assert_eq!(sim.poll_events().len(), 1, "el evento del comando sí queda");
    }

    #[test]
    fn los_comandos_se_aplican_una_vez_por_llamada_aunque_haya_interrupcion() {
        let mut sim = sim(1);
        add(
            &mut sim,
            "alerta",
            0,
            Cadence::Daily,
            &[(3 * DAY, Interrupt::Halt)],
        );
        sim.submit_command(Box::new(Mark(1))).unwrap();
        sim.advance(Increment::D30).unwrap(); // interrumpido en el día 3
        assert_eq!(sim.steps(), 1);
        sim.submit_command(Box::new(Mark(2))).unwrap();
        sim.advance(Increment::D5).unwrap();
        assert_eq!(sim.steps(), 2);
        let h = sim.command_history();
        assert_eq!((h[0].step, h[0].applied_at), (0, SimTime::ZERO));
        assert_eq!(
            (h[1].step, h[1].applied_at),
            (1, SimTime::from_seconds(3 * DAY))
        );
    }

    #[test]
    fn el_desborde_es_error_y_no_cambia_el_estado() {
        let mut sim = sim(1);
        add(&mut sim, "dia", 0, Cadence::Daily, &[]);
        sim.submit_command(Box::new(Mark(1))).unwrap();
        sim.scheduler_mut().set_time_for_test(SimTime::MAX);
        assert_eq!(sim.advance(Increment::S5), Err(AdvanceError::TimeOverflow));
        assert_eq!(sim.time(), SimTime::MAX);
        assert_eq!(sim.steps(), 0);
        assert_eq!(sim.pending_command_count(), 1, "el comando sigue pendiente");
        assert_eq!(sim.command_history(), []);
        assert!(sim.events().is_empty());
    }

    #[test]
    fn dos_sim_iguales_dan_los_mismos_resultados() {
        let run = || {
            let mut sim = sim(7);
            add(&mut sim, "dia", 1, Cadence::Daily, &[]);
            add(
                &mut sim,
                "alerta",
                0,
                Cadence::Weekly,
                &[(14 * DAY, Interrupt::Halt), (28 * DAY, Interrupt::None)],
            );
            sim.submit_command(Box::new(Mark(1))).unwrap();
            let results: Vec<AdvanceResult> = (0..4)
                .map(|_| sim.advance(Increment::D30).unwrap())
                .collect();
            (
                results,
                sim.time(),
                sim.events().iter().cloned().collect::<Vec<_>>(),
            )
        };
        assert_eq!(run(), run());
    }

    /// Propiedad: lo avanzado suma el tiempo total, nada corre dos veces ni se saltea, y la
    /// interrupción recorta sólo hasta su instante, para secuencias pseudoaleatorias.
    #[test]
    fn lo_avanzado_suma_el_total_y_no_se_pierden_corridas() {
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        for round in 0..50 {
            let mut sim = sim(round);
            let runs = add(&mut sim, "dia", 5, Cadence::Daily, &[]);
            let sem = add(&mut sim, "sem", 3, Cadence::Weekly, &[]);
            // Interrumpe en los días 9, 23 y 100 (que caen dentro de algún avance largo).
            let halts = [9 * DAY, 23 * DAY, 100 * DAY].map(|t| (t, Interrupt::Halt));
            add(&mut sim, "alerta", 0, Cadence::Daily, &halts);
            let (mut total, mut calls) = (0u64, 0u64);
            while sim.time().as_seconds() < 150 * DAY {
                // xorshift64: sin dependencias externas.
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let inc = Increment::ALL[(state % 5) as usize];
                let r = sim.advance(inc).unwrap();
                assert!(r.advanced <= inc.seconds());
                assert_eq!(r.completed(), r.advanced == inc.seconds());
                if !r.completed() {
                    assert!(r.interrupted_by.is_some());
                }
                total += r.advanced;
                calls += 1;
            }
            let t = sim.time().as_seconds();
            assert_eq!(total, t);
            assert_eq!(sim.steps(), calls);
            assert_eq!(count(&runs, "dia") as u64, t / DAY);
            assert_eq!(count(&sem, "sem") as u64, t / (7 * DAY));
            let halted = sim.events().len();
            assert_eq!(
                halted,
                [9, 23, 100].iter().filter(|d| **d * DAY <= t).count()
            );
        }
    }
}
