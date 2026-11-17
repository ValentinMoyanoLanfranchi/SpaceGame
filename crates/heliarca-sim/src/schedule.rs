//! Trait [`System`] y planificador multi-frecuencia (docs 01 «Tiempo, incrementos y ticks» y 21 §2).
//!
//! # Convención de calendario de cadencias (doc 22 §0)
//!
//! Por defecto semana = [`DAYS_PER_WEEK`] (7) días, mes = [`DAYS_PER_MONTH`] (30) días y año =
//! [`DAYS_PER_YEAR`] (365) días; son los valores de [`Params::DEFAULT`] y salen de
//! `data/params.json` ([`Scheduler::with_params`], módulo `params`). Los meses **no** se alinean con el año (12 meses = 360 días):
//! la cadencia mensual dispara cada 30 días exactos, de modo que en 365 días corre 12 veces.
//!
//! # Qué significa «cruzar una frontera»
//!
//! Las fronteras de una cadencia con período `P` segundos están en `k * P` para `k >= 1`,
//! contadas desde [`SimTime::ZERO`] (el inicio de la partida), sin importar la fecha de inicio
//! del [`Calendar`](crate::time::Calendar). El instante 0 no es una frontera.
//!
//! Al avanzar de `t0` a `t1`, un sistema corre **una vez por cada frontera `k * P` con
//! `t0 < k * P <= t1`** (`floor(t1 / P) - floor(t0 / P)` veces), y el [`StepContext`] lleva el
//! **instante de esa frontera**, no `t1`. Por eso un incremento largo no «se saltea» corridas:
//! avanzar 30 días de una vez ejecuta 30 veces un sistema diario y 1 vez uno mensual, igual que
//! avanzar un día por vez. El resultado total no depende de cómo se parta el avance.
//!
//! [`Cadence::EveryIncrement`] corre exactamente una vez por avance (por llamada a
//! [`Sim::advance`](crate::Sim::advance)), con el instante final del avance, que es el
//! planificado o el de la interrupción si el avance se detuvo antes. Corre **después** de las
//! demás corridas de ese instante, sea cual sea su prioridad.
//!
//! # Avance por tramos
//!
//! El planificador no decide cuándo detenerse: avanza hasta el instante que se le pide
//! ([`Scheduler::advance_to`]) y [`Scheduler::next_boundary`] dice dónde está la próxima
//! frontera de cadencia. [`Sim::advance`](crate::Sim::advance) usa ambas para partir un
//! incremento en tramos y mirar, entre tramo y tramo, si algún sistema emitió una
//! interrupción. Partir el avance no cambia lo que corre: cada frontera se cruza una sola vez.
//!
//! # Orden de ejecución
//!
//! Dentro de un mismo avance las corridas se ordenan cronológicamente (instante de la
//! frontera) y, a igual instante, por **prioridad fija** (menor valor primero) y luego por
//! **id**. El orden no depende del orden de registro. Un sistema registrado más tarde sólo
//! corre por las fronteras que se cruzan después de su registro.

use std::fmt;

use crate::events::{EventError, EventId, EventKind, EventLog, Interrupt, Payload};
use crate::invariants::{FnInvariant, Violation};
use crate::params::Params;
use crate::sim::Sim;
use crate::state_hash::StateWriter;
use crate::time::{SECONDS_PER_DAY, SimTime};

/// Días por semana del GDD (doc 22 §0): valor por defecto de `days_per_week` ([`Params`]).
pub const DAYS_PER_WEEK: u64 = 7;
/// Días por mes del GDD (doc 22 §0): valor por defecto de `days_per_month` ([`Params`]).
pub const DAYS_PER_MONTH: u64 = 30;
/// Segundos por semana.
pub const SECONDS_PER_WEEK: u64 = SECONDS_PER_DAY * DAYS_PER_WEEK;
/// Segundos por mes.
pub const SECONDS_PER_MONTH: u64 = SECONDS_PER_DAY * DAYS_PER_MONTH;

/// Frecuencia con la que corre un sistema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Cadence {
    /// Una vez por incremento de tiempo, sea cual sea su duración.
    EveryIncrement,
    /// Cada día (86 400 s).
    Daily,
    /// Cada semana (7 días).
    Weekly,
    /// Cada mes (30 días).
    Monthly,
    /// Cada año (365 días).
    Yearly,
}

impl Cadence {
    /// Período en segundos con los parámetros por defecto del GDD ([`Params::DEFAULT`]);
    /// `None` para [`Cadence::EveryIncrement`] (no tiene período fijo).
    #[must_use]
    pub const fn period_seconds(self) -> Option<u64> {
        self.period_seconds_with(&Params::DEFAULT)
    }

    /// Período en segundos según los `params` dados (días por semana, mes y año);
    /// `None` para [`Cadence::EveryIncrement`].
    #[must_use]
    pub const fn period_seconds_with(self, params: &Params) -> Option<u64> {
        match self {
            Self::EveryIncrement => None,
            Self::Daily => Some(SECONDS_PER_DAY),
            // `Params` acota los días a `MAX_DAYS`: no desborda; `saturating` por si acaso.
            Self::Weekly => Some(SECONDS_PER_DAY.saturating_mul(params.days_per_week())),
            Self::Monthly => Some(SECONDS_PER_DAY.saturating_mul(params.days_per_month())),
            Self::Yearly => Some(SECONDS_PER_DAY.saturating_mul(params.days_per_year())),
        }
    }
}

/// Identificador estable de un sistema (texto fijo, p. ej. `"sys.produccion"`).
///
/// Se ordena lexicográficamente y desempata el orden de ejecución.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemId(pub &'static str);

impl fmt::Display for SystemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// Prioridad fija de un sistema: el valor menor corre primero entre corridas simultáneas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Priority(pub u16);

/// Contexto de una corrida de [`System::step`].
///
/// Da el instante y la cadencia de la corrida y el registro de eventos, para que el sistema
/// emita (con [`StepContext::emit`]). El resto del estado del mundo se agregará con el `Sim`
/// raíz.
#[derive(Debug)]
pub struct StepContext<'a> {
    time: SimTime,
    cadence: Cadence,
    events: &'a mut EventLog,
}

impl<'a> StepContext<'a> {
    /// Crea un contexto para una corrida en `time` disparada por `cadence`.
    #[must_use]
    pub fn new(time: SimTime, cadence: Cadence, events: &'a mut EventLog) -> Self {
        Self {
            time,
            cadence,
            events,
        }
    }

    /// Instante de la frontera que dispara esta corrida (para
    /// [`Cadence::EveryIncrement`], el instante al final del avance).
    #[must_use]
    pub const fn time(&self) -> SimTime {
        self.time
    }

    /// Cadencia del sistema que corre.
    #[must_use]
    pub const fn cadence(&self) -> Cadence {
        self.cadence
    }

    /// Emite un evento en el instante de esta corrida. Si `interrupt` no es
    /// [`Interrupt::None`], el avance en curso se detiene en este instante.
    ///
    /// # Errors
    ///
    /// El [`EventError`] de [`EventLog::emit`].
    pub fn emit(
        &mut self,
        kind: EventKind,
        cause: Option<EventId>,
        interrupt: Interrupt,
        payload: Payload,
    ) -> Result<EventId, EventError> {
        self.events.emit(self.time, kind, cause, interrupt, payload)
    }
}

/// Un subsistema de la simulación que el planificador ejecuta según su cadencia.
pub trait System: fmt::Debug {
    /// Id estable; único dentro de un [`Scheduler`].
    fn id(&self) -> SystemId;

    /// Prioridad fija; el valor menor corre primero. Por defecto, la más alta (0).
    fn priority(&self) -> Priority {
        Priority::default()
    }

    /// Cadencia del sistema.
    fn cadence(&self) -> Cadence;

    /// Ejecuta una corrida.
    fn step(&mut self, ctx: &mut StepContext<'_>);

    /// Escribe en `writer` el estado propio del sistema para el hash de estado
    /// ([`Sim::state_hash`](crate::Sim::state_hash)). Por defecto no escribe nada.
    ///
    /// Un sistema con estado que afecte a lo que simula (su RNG, acumuladores, colas...) **debe**
    /// sobrescribirlo; si no, ese estado queda fuera del hash y dos partidas que divergen sólo
    /// ahí no se detectan. Escribir siempre en el mismo orden y con los métodos tipados del
    /// [`StateWriter`] (ver el módulo `state_hash`).
    fn hash_state(&self, _writer: &mut StateWriter) {}
}

/// Errores del planificador.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleError {
    /// Ya hay un sistema registrado con ese id.
    DuplicateId(SystemId),
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(f, "ya hay un sistema registrado con id «{id}»"),
        }
    }
}

impl std::error::Error for ScheduleError {}

/// Planificador: registra sistemas, avanza el tiempo y los dispara según su cadencia.
#[derive(Debug, Default)]
pub struct Scheduler {
    time: SimTime,
    /// Parámetros que fijan el período de las cadencias semanal, mensual y anual.
    params: Params,
    /// Siempre ordenado por (prioridad, id): independiente del orden de registro.
    systems: Vec<Box<dyn System>>,
}

/// Invariantes del planificador y del tiempo (ver el módulo `invariants`).
pub(crate) const INVARIANTS: &[FnInvariant] = &[
    FnInvariant::new("time.monotonic", check_time_monotonic),
    FnInvariant::new("schedule.unique_ids", check_unique_ids),
    FnInvariant::new("schedule.order", check_order),
    FnInvariant::new("schedule.params_match_sim", check_params_match_sim),
];

/// El tiempo nunca retrocede respecto del final del último paso (el mojón del `Sim`).
fn check_time_monotonic(sim: &Sim) -> Result<(), Violation> {
    let (now, mark) = (sim.time(), sim.invariant_mark().time());
    if now < mark {
        return Err(Violation::new(
            "time.monotonic",
            Some("sim.time".to_owned()),
            format!(
                "el tiempo retrocedió: al terminar el paso anterior era {} s y ahora es {} s",
                mark.as_seconds(),
                now.as_seconds()
            ),
        ));
    }
    Ok(())
}

/// Los ids de sistema son únicos (`register` lo garantiza; esto lo vigila).
fn check_unique_ids(sim: &Sim) -> Result<(), Violation> {
    let systems = sim.scheduler().systems();
    for (index, system) in systems.iter().enumerate() {
        let id = system.id();
        if systems[..index].iter().any(|earlier| earlier.id() == id) {
            return Err(Violation::new(
                "schedule.unique_ids",
                Some(format!("system.{id}")),
                format!("hay más de un sistema registrado con id «{id}»"),
            ));
        }
    }
    Ok(())
}

/// Los sistemas están ordenados estrictamente por (prioridad, id): de eso depende el orden
/// determinista de ejecución.
fn check_order(sim: &Sim) -> Result<(), Violation> {
    for pair in sim.scheduler().systems().windows(2) {
        let (before, after) = (order_key(pair[0].as_ref()), order_key(pair[1].as_ref()));
        if before >= after {
            return Err(Violation::new(
                "schedule.order",
                Some(format!("system.{}", after.1)),
                format!(
                    "«{}» (prioridad {}) figura después de «{}» (prioridad {}), pero no es mayor en (prioridad, id)",
                    after.1, after.0.0, before.1, before.0.0
                ),
            ));
        }
    }
    Ok(())
}

/// El planificador usa los mismos parámetros que el `Sim` (es una copia que fija las cadencias).
fn check_params_match_sim(sim: &Sim) -> Result<(), Violation> {
    if sim.scheduler().params != *sim.params() {
        return Err(Violation::new(
            "schedule.params_match_sim",
            Some("scheduler.params".to_owned()),
            "los parámetros del planificador difieren de los del Sim: las cadencias no \
             corresponderían a los datos",
        ));
    }
    Ok(())
}

fn order_key(system: &dyn System) -> (Priority, SystemId) {
    (system.priority(), system.id())
}

impl Scheduler {
    /// Planificador vacío en [`SimTime::ZERO`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Planificador vacío cuyas cadencias semanal, mensual y anual salen de `params`.
    #[must_use]
    pub fn with_params(params: Params) -> Self {
        Self {
            params,
            ..Self::default()
        }
    }

    /// Instante actual.
    #[must_use]
    pub const fn time(&self) -> SimTime {
        self.time
    }

    /// Cantidad de sistemas registrados.
    #[must_use]
    pub fn len(&self) -> usize {
        self.systems.len()
    }

    /// `true` si no hay sistemas registrados.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }

    /// Registra un sistema. Su prioridad e id se leen una sola vez, ahora.
    ///
    /// # Errors
    /// [`ScheduleError::DuplicateId`] si ya hay un sistema con el mismo id.
    pub fn register(&mut self, system: Box<dyn System>) -> Result<(), ScheduleError> {
        let key = order_key(system.as_ref());
        if self.systems.iter().any(|s| s.id() == key.1) {
            return Err(ScheduleError::DuplicateId(key.1));
        }
        // Los ids son únicos, así que la clave nunca coincide: `Err` trae la posición.
        let (Ok(pos) | Err(pos)) = self
            .systems
            .binary_search_by(|s| order_key(s.as_ref()).cmp(&key));
        self.systems.insert(pos, system);
        Ok(())
    }

    /// `(instante, sistemas)` para el hash de estado. Desestructura **sin `..`**: un campo nuevo
    /// obliga a decidir si se hashea. `params` se excluye acá a propósito: es copia de los
    /// `Params` del `Sim`, que se hashean en su propia sección.
    pub(crate) fn hash_parts(&self) -> (SimTime, &[Box<dyn System>]) {
        let Self {
            time,
            params: _copia_de_los_params_del_sim,
            systems,
        } = self;
        (*time, systems)
    }

    /// `(instante, cantidad de sistemas)` para el guardado. Desestructura **sin `..`**: un campo
    /// nuevo obliga a decidir si se guarda. Los sistemas son código y no se serializan (el
    /// guardado se niega si hay alguno); `params` es copia de los del `Sim`, que se guardan aparte.
    pub(crate) fn save_parts(&self) -> (SimTime, usize) {
        let Self {
            time,
            params: _copia_de_los_params_del_sim,
            systems,
        } = self;
        (*time, systems.len())
    }

    /// Planificador sin sistemas, en el instante `time` y con las cadencias de `params`
    /// (para cargar un guardado).
    pub(crate) fn from_save_parts(time: SimTime, params: Params) -> Self {
        Self {
            time,
            params,
            systems: Vec::new(),
        }
    }

    /// Sistemas registrados, en orden de ejecución (prioridad, id), para el hash de estado.
    pub(crate) fn systems(&self) -> &[Box<dyn System>] {
        &self.systems
    }

    /// Sistemas (escritura): para corromper el orden en los tests de invariantes.
    #[cfg(test)]
    pub(crate) fn systems_for_test(&mut self) -> &mut Vec<Box<dyn System>> {
        &mut self.systems
    }

    /// Fija los parámetros sin más (para los tests de invariantes).
    #[cfg(test)]
    pub(crate) fn set_params_for_test(&mut self, params: Params) {
        self.params = params;
    }

    /// Fija el instante sin ejecutar nada (para probar el desborde).
    #[cfg(test)]
    pub(crate) fn set_time_for_test(&mut self, time: SimTime) {
        self.time = time;
    }

    /// Próxima frontera de cadencia estrictamente posterior a `after`, o `limit` si llega antes
    /// (o si no hay sistemas periódicos). Si `limit <= after`, devuelve `limit`.
    ///
    /// Los sistemas [`Cadence::EveryIncrement`] no tienen fronteras propias.
    #[must_use]
    pub fn next_boundary(&self, after: SimTime, limit: SimTime) -> SimTime {
        let mut next = limit;
        for period in self
            .systems
            .iter()
            .filter_map(|s| s.cadence().period_seconds_with(&self.params))
        {
            // `None` sólo cerca de `u64::MAX`: esa frontera no es representable, no cuenta.
            let boundary = (after.as_seconds() / period + 1).checked_mul(period);
            if let Some(boundary) = boundary {
                next = next.min(SimTime::from_seconds(boundary));
            }
        }
        next
    }

    /// Avanza el tiempo hasta `to` y ejecuta las corridas periódicas de las fronteras cruzadas
    /// (ver la documentación del módulo). Si `to` no es posterior al instante actual, no hace
    /// nada: el tiempo nunca retrocede.
    ///
    /// Los sistemas [`Cadence::EveryIncrement`] no corren aquí sino en
    /// [`Scheduler::run_every_increment`].
    pub fn advance_to(&mut self, to: SimTime, events: &mut EventLog) {
        let from = self.time;
        if to <= from {
            return;
        }

        // Corridas (instante, índice en `systems`). `k * P <= to` porque `k <= to / P`: no
        // desborda.
        let mut runs: Vec<(SimTime, usize)> = Vec::new();
        for (index, system) in self.systems.iter().enumerate() {
            if let Some(period) = system.cadence().period_seconds_with(&self.params) {
                let first = from.as_seconds() / period + 1;
                let last = to.as_seconds() / period;
                for k in first..=last {
                    runs.push((SimTime::from_seconds(k * period), index));
                }
            }
        }
        // `systems` ya está en orden (prioridad, id): el índice desempata por eso.
        runs.sort_unstable();

        self.time = to;
        for (time, index) in runs {
            let system = &mut self.systems[index];
            let mut ctx = StepContext::new(time, system.cadence(), events);
            system.step(&mut ctx);
        }
    }

    /// Ejecuta una vez cada sistema [`Cadence::EveryIncrement`], con el instante actual, en
    /// orden (prioridad, id). Se llama al final de un avance.
    pub fn run_every_increment(&mut self, events: &mut EventLog) {
        let time = self.time;
        for system in &mut self.systems {
            if system.cadence() == Cadence::EveryIncrement {
                let mut ctx = StepContext::new(time, Cadence::EveryIncrement, events);
                system.step(&mut ctx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::{Increment, SECONDS_PER_YEAR};
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    #[derive(Debug)]
    struct Counter {
        id: &'static str,
        priority: u16,
        cadence: Cadence,
        runs: Rc<Cell<u32>>,
    }

    impl System for Counter {
        fn id(&self) -> SystemId {
            SystemId(self.id)
        }
        fn priority(&self) -> Priority {
            Priority(self.priority)
        }
        fn cadence(&self) -> Cadence {
            self.cadence
        }
        fn step(&mut self, _ctx: &mut StepContext<'_>) {
            self.runs.set(self.runs.get() + 1);
        }
    }

    type Log = Rc<RefCell<Vec<(u64, &'static str)>>>;

    #[derive(Debug)]
    struct Recorder {
        id: &'static str,
        priority: u16,
        cadence: Cadence,
        log: Log,
    }

    impl System for Recorder {
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
            self.log
                .borrow_mut()
                .push((ctx.time().as_seconds(), self.id));
        }
    }

    fn counter(cadence: Cadence) -> (Box<dyn System>, Rc<Cell<u32>>) {
        let runs = Rc::new(Cell::new(0));
        let system = Counter {
            id: match cadence {
                Cadence::EveryIncrement => "c.inc",
                Cadence::Daily => "c.dia",
                Cadence::Weekly => "c.sem",
                Cadence::Monthly => "c.mes",
                Cadence::Yearly => "c.anio",
            },
            priority: 0,
            cadence,
            runs: Rc::clone(&runs),
        };
        (Box::new(system), runs)
    }

    /// Scheduler con un contador por cadencia: [inc, dia, sem, mes, anio].
    fn all_counters() -> (Scheduler, [Rc<Cell<u32>>; 5]) {
        let mut sched = Scheduler::new();
        let mut cells = Vec::new();
        for cadence in [
            Cadence::EveryIncrement,
            Cadence::Daily,
            Cadence::Weekly,
            Cadence::Monthly,
            Cadence::Yearly,
        ] {
            let (system, runs) = counter(cadence);
            sched.register(system).unwrap();
            cells.push(runs);
        }
        let cells: [Rc<Cell<u32>>; 5] = cells.try_into().unwrap();
        (sched, cells)
    }

    fn counts(cells: &[Rc<Cell<u32>>; 5]) -> [u32; 5] {
        [
            cells[0].get(),
            cells[1].get(),
            cells[2].get(),
            cells[3].get(),
            cells[4].get(),
        ]
    }

    /// Avanza un incremento completo, como un `Sim::advance` sin interrupciones.
    fn advance(sched: &mut Scheduler, increment: Increment) {
        let to = sched.time().checked_advance(increment).unwrap();
        let mut events = EventLog::new();
        sched.advance_to(to, &mut events);
        sched.run_every_increment(&mut events);
    }

    fn advance_n(sched: &mut Scheduler, increment: Increment, n: u32) {
        for _ in 0..n {
            advance(sched, increment);
        }
    }

    #[test]
    fn convencion_de_cadencias() {
        assert_eq!(DAYS_PER_WEEK, 7);
        assert_eq!(DAYS_PER_MONTH, 30);
        assert_eq!(Cadence::Daily.period_seconds(), Some(86_400));
        assert_eq!(Cadence::Weekly.period_seconds(), Some(7 * 86_400));
        assert_eq!(Cadence::Monthly.period_seconds(), Some(30 * 86_400));
        assert_eq!(Cadence::Yearly.period_seconds(), Some(365 * 86_400));
        assert_eq!(Cadence::EveryIncrement.period_seconds(), None);
    }

    #[test]
    fn treinta_dias_con_distintas_particiones() {
        let particiones: [&[(Increment, u32)]; 5] = [
            &[(Increment::D1, 30)],
            &[(Increment::D5, 6)],
            &[(Increment::D30, 1)],
            &[(Increment::D5, 5), (Increment::D1, 4), (Increment::H1, 24)],
            // 29 d + 23 h + 720 × 5 s (1 h) = 30 d.
            &[
                (Increment::D1, 29),
                (Increment::H1, 23),
                (Increment::S5, 720),
            ],
        ];
        for (i, particion) in particiones.iter().enumerate() {
            let (mut sched, cells) = all_counters();
            for &(inc, n) in *particion {
                advance_n(&mut sched, inc, n);
            }
            if i == 4 {
                assert_eq!(sched.time(), SimTime::from_days(30).unwrap());
            }
            let c = counts(&cells);
            assert_eq!(c[1], 30, "diario, partición {i}");
            assert_eq!(c[2], 4, "semanal, partición {i}");
            assert_eq!(c[3], 1, "mensual, partición {i}");
            assert_eq!(c[4], 0, "anual, partición {i}");
        }
    }

    #[test]
    fn el_instante_30_es_frontera_mensual_y_el_29_no() {
        let (mut sched, cells) = all_counters();
        advance_n(&mut sched, Increment::D1, 29);
        assert_eq!(cells[3].get(), 0);
        advance(&mut sched, Increment::D1);
        assert_eq!(cells[3].get(), 1);
    }

    #[test]
    fn trescientos_sesenta_y_cinco_dias() {
        // 365 d = 12 × D30 + 5 d; o 73 × D5; o 365 × D1.
        let particiones: [&[(Increment, u32)]; 3] = [
            &[(Increment::D1, 365)],
            &[(Increment::D5, 73)],
            &[(Increment::D30, 12), (Increment::D5, 1)],
        ];
        for (i, particion) in particiones.iter().enumerate() {
            let (mut sched, cells) = all_counters();
            for &(inc, n) in *particion {
                advance_n(&mut sched, inc, n);
            }
            assert_eq!(sched.time(), SimTime::from_days(365).unwrap());
            let c = counts(&cells);
            assert_eq!(c[1], 365, "diario, partición {i}");
            assert_eq!(c[2], 52, "semanal, partición {i}");
            assert_eq!(c[3], 12, "mensual, partición {i}");
            assert_eq!(c[4], 1, "anual, partición {i}");
        }
    }

    #[test]
    fn cada_incremento_corre_una_vez_por_incremento() {
        let (mut sched, cells) = all_counters();
        advance_n(&mut sched, Increment::S5, 3);
        advance(&mut sched, Increment::D30);
        assert_eq!(cells[0].get(), 4);
    }

    #[test]
    fn el_contexto_lleva_el_instante_de_la_frontera() {
        let log: Log = Rc::new(RefCell::new(Vec::new()));
        let mut sched = Scheduler::new();
        sched
            .register(Box::new(Recorder {
                id: "dia",
                priority: 0,
                cadence: Cadence::Daily,
                log: Rc::clone(&log),
            }))
            .unwrap();
        advance(&mut sched, Increment::D5);
        let times: Vec<u64> = log.borrow().iter().map(|e| e.0).collect();
        let expected: Vec<u64> = (1..=5).map(|d| d * SECONDS_PER_DAY).collect();
        assert_eq!(times, expected);
    }

    #[test]
    fn avance_corto_sin_frontera_no_corre_sistemas_periodicos() {
        let (mut sched, cells) = all_counters();
        advance_n(&mut sched, Increment::H1, 23);
        assert_eq!(counts(&cells), [23, 0, 0, 0, 0]);
        advance(&mut sched, Increment::H1);
        assert_eq!(counts(&cells), [24, 1, 0, 0, 0]);
    }

    fn run_with_order(order: &[usize]) -> Vec<(u64, &'static str)> {
        let log: Log = Rc::new(RefCell::new(Vec::new()));
        // (id, prioridad, cadencia)
        let defs = [
            ("b.mes", 5, Cadence::Monthly),
            ("a.dia", 5, Cadence::Daily),
            ("z.dia", 1, Cadence::Daily),
            ("m.sem", 3, Cadence::Weekly),
            ("k.inc", 5, Cadence::EveryIncrement),
            ("y.anio", 0, Cadence::Yearly),
        ];
        let mut sched = Scheduler::new();
        for &i in order {
            let (id, priority, cadence) = defs[i];
            sched
                .register(Box::new(Recorder {
                    id,
                    priority,
                    cadence,
                    log: Rc::clone(&log),
                }))
                .unwrap();
        }
        advance_n(&mut sched, Increment::D30, 13);
        log.take()
    }

    #[test]
    fn el_orden_es_identico_entre_corridas_y_ordenes_de_registro() {
        let base = run_with_order(&[0, 1, 2, 3, 4, 5]);
        assert!(base.len() > 100);
        assert_eq!(base, run_with_order(&[0, 1, 2, 3, 4, 5]));
        assert_eq!(base, run_with_order(&[5, 4, 3, 2, 1, 0]));
        assert_eq!(base, run_with_order(&[3, 0, 5, 1, 4, 2]));
    }

    #[test]
    fn simultaneos_se_ordenan_por_prioridad_y_luego_id() {
        let log: Log = Rc::new(RefCell::new(Vec::new()));
        let mut sched = Scheduler::new();
        for (id, priority) in [("c", 2), ("b", 1), ("a", 2), ("d", 0)] {
            sched
                .register(Box::new(Recorder {
                    id,
                    priority,
                    cadence: Cadence::Daily,
                    log: Rc::clone(&log),
                }))
                .unwrap();
        }
        advance(&mut sched, Increment::D1);
        let ids: Vec<&str> = log.borrow().iter().map(|e| e.1).collect();
        assert_eq!(ids, ["d", "b", "a", "c"]);
    }

    #[test]
    fn en_un_avance_largo_el_orden_es_cronologico_y_luego_por_prioridad() {
        let log: Log = Rc::new(RefCell::new(Vec::new()));
        let mut sched = Scheduler::new();
        for (id, priority, cadence) in [("mes", 0, Cadence::Monthly), ("dia", 1, Cadence::Daily)] {
            sched
                .register(Box::new(Recorder {
                    id,
                    priority,
                    cadence,
                    log: Rc::clone(&log),
                }))
                .unwrap();
        }
        advance(&mut sched, Increment::D30);
        let log = log.borrow();
        assert_eq!(log.len(), 31);
        // Los días 1..=29 primero; el día 30 corre el mensual (prioridad 0) antes que el diario.
        assert!(log[..29].iter().all(|e| e.1 == "dia"));
        assert_eq!(log[29], (30 * SECONDS_PER_DAY, "mes"));
        assert_eq!(log[30], (30 * SECONDS_PER_DAY, "dia"));
        assert!(log.windows(2).all(|w| w[0].0 <= w[1].0));
    }

    #[test]
    fn id_duplicado_se_rechaza_aunque_cambie_la_prioridad() {
        let mut sched = Scheduler::new();
        let (a, _) = counter(Cadence::Daily);
        sched.register(a).unwrap();
        let (b, _) = counter(Cadence::Daily);
        assert_eq!(
            sched.register(b),
            Err(ScheduleError::DuplicateId(SystemId("c.dia")))
        );
        let other = Counter {
            id: "c.dia",
            priority: 9,
            cadence: Cadence::Weekly,
            runs: Rc::new(Cell::new(0)),
        };
        assert_eq!(
            sched.register(Box::new(other)),
            Err(ScheduleError::DuplicateId(SystemId("c.dia")))
        );
        assert_eq!(sched.len(), 1);
        assert!(!sched.is_empty());
    }

    #[test]
    fn cerca_del_maximo_no_desborda_en_las_fronteras() {
        let mut sched = Scheduler::new();
        let (system, runs) = counter(Cadence::Daily);
        sched.register(system).unwrap();
        sched.time = SimTime::from_seconds(u64::MAX - 5);
        advance(&mut sched, Increment::S5);
        assert_eq!(sched.time(), SimTime::MAX);
        assert_eq!(runs.get(), 0);
    }

    #[test]
    fn registro_tardio_cuenta_desde_el_registro() {
        let mut sched = Scheduler::new();
        advance_n(&mut sched, Increment::D1, 10);
        let (system, runs) = counter(Cadence::Daily);
        sched.register(system).unwrap();
        advance_n(&mut sched, Increment::D1, 3);
        assert_eq!(runs.get(), 3);
    }

    /// El resultado no depende de la partición: secuencia pseudoaleatoria determinista.
    #[test]
    fn particiones_arbitrarias_dan_los_mismos_conteos() {
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        for _ in 0..50 {
            let (mut sched, cells) = all_counters();
            let mut incrementos = 0u32;
            while sched.time().as_seconds() < 400 * SECONDS_PER_DAY {
                // xorshift64: sin dependencias externas.
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let inc = Increment::ALL[(state % 5) as usize];
                advance(&mut sched, inc);
                incrementos += 1;
            }
            let t = sched.time().as_seconds();
            let c = counts(&cells);
            assert_eq!(c[0], incrementos);
            assert_eq!(u64::from(c[1]), t / SECONDS_PER_DAY);
            assert_eq!(u64::from(c[2]), t / SECONDS_PER_WEEK);
            assert_eq!(u64::from(c[3]), t / SECONDS_PER_MONTH);
            assert_eq!(u64::from(c[4]), t / SECONDS_PER_YEAR);
        }
    }

    #[test]
    fn la_proxima_frontera_es_la_mas_cercana_o_el_limite() {
        let mut sched = Scheduler::new();
        let day = SimTime::from_seconds(SECONDS_PER_DAY);
        let far = SimTime::from_seconds(30 * SECONDS_PER_DAY);
        // Sin sistemas periódicos: el límite.
        assert_eq!(sched.next_boundary(SimTime::ZERO, far), far);
        sched.register(counter(Cadence::EveryIncrement).0).unwrap();
        assert_eq!(sched.next_boundary(SimTime::ZERO, far), far);
        sched.register(counter(Cadence::Weekly).0).unwrap();
        assert_eq!(
            sched.next_boundary(SimTime::ZERO, far),
            SimTime::from_seconds(SECONDS_PER_WEEK)
        );
        sched.register(counter(Cadence::Daily).0).unwrap();
        assert_eq!(sched.next_boundary(SimTime::ZERO, far), day);
        // Estrictamente posterior: desde una frontera, la siguiente.
        assert_eq!(
            sched.next_boundary(day, far),
            SimTime::from_seconds(2 * SECONDS_PER_DAY)
        );
        // El límite gana si está antes.
        assert_eq!(
            sched.next_boundary(SimTime::ZERO, SimTime::from_seconds(5)),
            SimTime::from_seconds(5)
        );
        // Cerca del máximo la frontera no es representable: se ignora.
        assert_eq!(
            sched.next_boundary(SimTime::MAX, SimTime::MAX),
            SimTime::MAX
        );
    }

    #[test]
    fn advance_to_no_retrocede_ni_repite() {
        let (mut sched, cells) = all_counters();
        let mut events = EventLog::new();
        sched.advance_to(SimTime::from_seconds(3 * SECONDS_PER_DAY), &mut events);
        sched.advance_to(SimTime::from_seconds(SECONDS_PER_DAY), &mut events);
        sched.advance_to(SimTime::from_seconds(3 * SECONDS_PER_DAY), &mut events);
        assert_eq!(sched.time(), SimTime::from_seconds(3 * SECONDS_PER_DAY));
        assert_eq!(cells[1].get(), 3);
        assert_eq!(cells[0].get(), 0, "EveryIncrement no corre en advance_to");
    }
}
