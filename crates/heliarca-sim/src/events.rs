//! Bus de eventos con causa (docs 01 «Interrupciones», 15 «Registro de eventos» y 16
//! «Inspección causal»; doc 21 §2).
//!
//! El [`EventLog`] es un registro **append-only**: sólo se agregan eventos ([`EventLog::emit`]);
//! no hay API para borrar ni mutar los ya registrados (el log sólo entrega `&Event`).
//!
//! # Ids de evento
//!
//! [`EventId`] se crea con [`define_id!`](crate::define_id) pero **no** pasa por una
//! `Arena` (interna del crate): una arena reutiliza slots (con otra generación) y un log
//! append-only jamás reutiliza un id, porque la causa de un evento debe seguir apuntando al
//! mismo hecho para siempre. Los ids son secuenciales desde 0 y el id es el índice en el log.
//! Con `u32` caben 2^32 eventos; al agotarse, [`EventLog::emit`] devuelve
//! [`EventError::LogFull`] en vez de envolver.
//!
//! # Causas
//!
//! Cada evento puede citar su causa (`cause`), que debe ser un evento **ya registrado** y no
//! posterior en el tiempo. Como un evento sólo puede apuntar a ids menores que el suyo, la
//! cadena causal es acíclica por construcción y se recorre de forma iterativa
//! ([`EventLog::causal_chain`], [`EventLog::root_cause`]).
//!
//! # Consumo por la UI
//!
//! [`EventLog::poll_events`] devuelve un `&[Event]` con lo registrado desde la llamada anterior:
//! no asigna memoria ni clona (los eventos viven contiguos), y la capa de Godot lo copiará a sus
//! modelos de vista. El préstamo es `&mut self` mientras el slice esté vivo; se copia y se suelta.

use std::fmt;

use crate::define_id;
use crate::ids::EntityId;
use crate::invariants::{FnInvariant, Violation, entity_event};
use crate::sim::Sim;
use crate::time::SimTime;

define_id! {
    /// Id de un evento: secuencial, estable y nunca reutilizado. Texto: `event.<nombre>`.
    EventId, "event"
}

/// Tipo de evento.
///
/// Por ahora sólo los del registro mínimo del MVP (doc 15). Los tipos reales se agregan junto
/// con cada sistema que los emite; por eso el enum es `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EventKind {
    /// Una investigación se completó.
    ResearchCompleted,
    /// Un lanzamiento fue exitoso.
    LaunchSucceeded,
    /// Un lanzamiento falló.
    LaunchFailed,
    /// Un relevamiento se completó.
    SurveyCompleted,
    /// Se fundó una colonia.
    ColonyFounded,
    /// Cambió el apoyo público.
    PublicSupportChanged,
    /// Estalló una crisis interna.
    InternalCrisis,
    /// Se cruzó el punto de salto.
    JumpPointCrossed,
}

/// Efecto del evento sobre el avance del tiempo (doc 01 «Interrupciones»).
///
/// Aquí sólo es el dato; la lógica que detiene o ralentiza el avance es de otra tarea (#27).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[non_exhaustive]
pub enum Interrupt {
    /// No interrumpe.
    #[default]
    None,
    /// Evento relevante: detiene el avance.
    Halt,
    /// Ralentiza el avance y abre una ventana de decisión (p. ej. contacto hostil).
    SlowDown,
}

/// Valor de un campo del [`Payload`]. Sin flotantes: igualdad exacta y determinista.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// Entero con signo.
    Int(i64),
    /// Entero sin signo.
    UInt(u64),
    /// Booleano.
    Bool(bool),
    /// Texto (p. ej. una clave de datos como `body.luna`).
    Text(String),
}

/// Datos del evento: pares clave-valor que conservan el orden de inserción.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Payload {
    fields: Vec<(String, Value)>,
}

impl Payload {
    /// Payload vacío.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Agrega un campo al final (builder).
    #[must_use]
    pub fn with(mut self, key: impl Into<String>, value: Value) -> Self {
        self.fields.push((key.into(), value));
        self
    }

    /// Primer valor con esa clave.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.fields.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Campos en orden de inserción.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.fields.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Cantidad de campos.
    #[must_use]
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// `true` si no tiene campos.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// Un evento registrado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// Id secuencial.
    pub id: EventId,
    /// Instante de simulación en que ocurrió.
    pub time: SimTime,
    /// Tipo de evento.
    pub kind: EventKind,
    /// Evento que lo causó, si lo hay.
    pub cause: Option<EventId>,
    /// Efecto sobre el avance del tiempo.
    pub interrupt: Interrupt,
    /// Datos del evento.
    pub payload: Payload,
}

/// Error al registrar un evento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventError {
    /// La causa no existe en el log.
    UnknownCause(EventId),
    /// La causa ocurrió después que el evento.
    CauseAfterEvent {
        /// Causa citada.
        cause: EventId,
        /// Instante de la causa.
        cause_time: SimTime,
        /// Instante del evento nuevo.
        time: SimTime,
    },
    /// Se agotaron los ids (2^32 eventos).
    LogFull,
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCause(id) => write!(f, "la causa {} no existe en el log", id.raw()),
            Self::CauseAfterEvent {
                cause,
                cause_time,
                time,
            } => write!(
                f,
                "la causa {} ({} s) es posterior al evento ({} s)",
                cause.raw(),
                cause_time.as_seconds(),
                time.as_seconds()
            ),
            Self::LogFull => f.write_str("el log de eventos agotó sus ids"),
        }
    }
}

impl std::error::Error for EventError {}

/// Invariantes del registro de eventos (ver el módulo `invariants`).
pub(crate) const INVARIANTS: &[FnInvariant] = &[
    FnInvariant::new("events.sequential_ids", check_sequential_ids),
    FnInvariant::new("events.causes_precede", check_causes_precede),
    FnInvariant::new("events.not_in_future", check_not_in_future),
    FnInvariant::new("events.cursor_in_range", check_cursor_in_range),
];

/// El id de cada evento es su posición en el log (secuencial desde 0, sin huecos ni repetidos).
fn check_sequential_ids(sim: &Sim) -> Result<(), Violation> {
    for (index, event) in sim.events().iter().enumerate() {
        if usize::try_from(event.id.raw()).ok() != Some(index) {
            return Err(Violation::new(
                "events.sequential_ids",
                Some(entity_event(event.id.raw())),
                format!(
                    "el evento en la posición {index} del registro tiene id {}; los ids deben ser secuenciales desde 0 (id = posición)",
                    event.id.raw()
                ),
            ));
        }
    }
    Ok(())
}

/// La causa de un evento existe, tiene un id **menor** y ocurrió en un instante **≤**.
fn check_causes_precede(sim: &Sim) -> Result<(), Violation> {
    let log = sim.events();
    for event in log {
        let Some(cause) = event.cause else { continue };
        let entity = Some(entity_event(event.id.raw()));
        if cause.raw() >= event.id.raw() {
            return Err(Violation::new(
                "events.causes_precede",
                entity,
                format!(
                    "la causa {} no es anterior al evento: su id debe ser menor",
                    entity_event(cause.raw())
                ),
            ));
        }
        match log.get(cause) {
            None => {
                return Err(Violation::new(
                    "events.causes_precede",
                    entity,
                    format!(
                        "la causa {} no existe en el registro",
                        entity_event(cause.raw())
                    ),
                ));
            }
            Some(parent) if parent.time > event.time => {
                return Err(Violation::new(
                    "events.causes_precede",
                    entity,
                    format!(
                        "la causa {} ocurrió a los {} s, después del evento ({} s)",
                        entity_event(cause.raw()),
                        parent.time.as_seconds(),
                        event.time.as_seconds()
                    ),
                ));
            }
            Some(_) => {}
        }
    }
    Ok(())
}

/// Ningún evento ocurre después del instante actual de la simulación.
fn check_not_in_future(sim: &Sim) -> Result<(), Violation> {
    for event in sim.events() {
        if event.time > sim.time() {
            return Err(Violation::new(
                "events.not_in_future",
                Some(entity_event(event.id.raw())),
                format!(
                    "el evento ocurrió a los {} s, pero el tiempo actual es {} s",
                    event.time.as_seconds(),
                    sim.time().as_seconds()
                ),
            ));
        }
    }
    Ok(())
}

/// El cursor de consumo no pasa del final del log.
fn check_cursor_in_range(sim: &Sim) -> Result<(), Violation> {
    let log = sim.events();
    if log.cursor > log.events.len() {
        return Err(Violation::new(
            "events.cursor_in_range",
            Some("events.cursor".to_owned()),
            format!(
                "el cursor de consumo ({}) supera la cantidad de eventos ({})",
                log.cursor,
                log.events.len()
            ),
        ));
    }
    Ok(())
}

/// Registro append-only de eventos con cursor de consumo.
#[derive(Debug, Clone, Default)]
pub struct EventLog {
    events: Vec<Event>,
    /// Cantidad de eventos ya entregados por [`EventLog::poll_events`].
    cursor: usize,
}

impl EventLog {
    /// Eventos para el hash de estado. Desestructura **sin `..`**: un campo nuevo obliga a decidir
    /// si se hashea. `cursor` se excluye a propósito: es estado de la interfaz (cuántos eventos
    /// ya se mostraron), no estado simulado; sondear o no no debe cambiar la repetibilidad.
    pub(crate) fn hash_parts(&self) -> &[Event] {
        let Self {
            events,
            cursor: _cursor_de_la_interfaz,
        } = self;
        events
    }

    /// Eventos (escritura): para corromper el log en los tests de invariantes.
    #[cfg(test)]
    pub(crate) fn events_for_test(&mut self) -> &mut Vec<Event> {
        &mut self.events
    }

    /// Cursor (escritura): para los tests de invariantes.
    #[cfg(test)]
    pub(crate) fn cursor_for_test(&mut self) -> &mut usize {
        &mut self.cursor
    }

    /// Log vacío.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra un evento y devuelve su id.
    ///
    /// # Errors
    ///
    /// - [`EventError::UnknownCause`] si `cause` no existe todavía.
    /// - [`EventError::CauseAfterEvent`] si la causa es posterior a `time`.
    /// - [`EventError::LogFull`] si no quedan ids.
    pub fn emit(
        &mut self,
        time: SimTime,
        kind: EventKind,
        cause: Option<EventId>,
        interrupt: Interrupt,
        payload: Payload,
    ) -> Result<EventId, EventError> {
        if let Some(cause_id) = cause {
            let parent = self
                .get(cause_id)
                .ok_or(EventError::UnknownCause(cause_id))?;
            if parent.time > time {
                return Err(EventError::CauseAfterEvent {
                    cause: cause_id,
                    cause_time: parent.time,
                    time,
                });
            }
        }
        let raw = u32::try_from(self.events.len()).map_err(|_| EventError::LogFull)?;
        let id = EventId::from_raw(raw);
        self.events.push(Event {
            id,
            time,
            kind,
            cause,
            interrupt,
            payload,
        });
        Ok(id)
    }

    /// Eventos registrados desde la llamada anterior (todos, en la primera). Avanza el cursor:
    /// no repite eventos ya entregados y, sin novedades, devuelve un slice vacío.
    pub fn poll_events(&mut self) -> &[Event] {
        let start = self.cursor.min(self.events.len());
        self.cursor = self.events.len();
        &self.events[start..]
    }

    /// Cantidad de eventos aún no entregados por [`EventLog::poll_events`].
    #[must_use]
    pub fn pending(&self) -> usize {
        self.events.len().saturating_sub(self.cursor)
    }

    /// Evento por id.
    #[must_use]
    pub fn get(&self, id: EventId) -> Option<&Event> {
        let index = usize::try_from(id.raw()).ok()?;
        self.events.get(index)
    }

    /// Todos los eventos, en orden de inserción.
    pub fn iter(&self) -> std::slice::Iter<'_, Event> {
        self.events.iter()
    }

    /// Cantidad de eventos.
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// `true` si no hay eventos.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Cadena causal desde `id` hasta la raíz, inclusive ambos. Vacía si `id` no existe.
    #[must_use]
    pub fn causal_chain(&self, id: EventId) -> CausalChain<'_> {
        CausalChain {
            log: self,
            next: Some(id),
        }
    }

    /// Raíz de la cadena causal de `id` (el propio evento si no tiene causa); `None` si no existe.
    #[must_use]
    pub fn root_cause(&self, id: EventId) -> Option<&Event> {
        self.causal_chain(id).last()
    }
}

impl<'a> IntoIterator for &'a EventLog {
    type Item = &'a Event;
    type IntoIter = std::slice::Iter<'a, Event>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterador (no recursivo) de la cadena causal: del evento a su raíz.
#[derive(Debug, Clone)]
pub struct CausalChain<'a> {
    log: &'a EventLog,
    next: Option<EventId>,
}

impl<'a> Iterator for CausalChain<'a> {
    type Item = &'a Event;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.log.get(self.next?);
        // Una causa siempre tiene id menor que su efecto, así que termina; si no existe, corta.
        self.next = event.and_then(|e| e.cause);
        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: u64) -> SimTime {
        SimTime::from_seconds(s)
    }

    fn emit(log: &mut EventLog, time: u64, cause: Option<EventId>) -> EventId {
        log.emit(
            t(time),
            EventKind::ResearchCompleted,
            cause,
            Interrupt::None,
            Payload::new(),
        )
        .unwrap()
    }

    #[test]
    fn cadena_de_uno() {
        let mut log = EventLog::new();
        let a = emit(&mut log, 0, None);
        let chain: Vec<_> = log.causal_chain(a).map(|e| e.id).collect();
        assert_eq!(chain, [a]);
        assert_eq!(log.root_cause(a).unwrap().id, a);
    }

    #[test]
    fn cadena_de_dos_llega_a_la_raiz() {
        let mut log = EventLog::new();
        let root = emit(&mut log, 5, None);
        let child = emit(&mut log, 10, Some(root));
        assert_eq!(log.get(child).unwrap().cause, Some(root));
        let chain: Vec<_> = log.causal_chain(child).map(|e| e.id).collect();
        assert_eq!(chain, [child, root]);
        assert_eq!(log.root_cause(child).unwrap().id, root);
    }

    #[test]
    fn cadena_larga_sin_desbordar_la_pila() {
        let mut log = EventLog::new();
        let root = emit(&mut log, 0, None);
        let mut last = root;
        for i in 1..10_000_u64 {
            last = emit(&mut log, i, Some(last));
        }
        assert_eq!(log.len(), 10_000);
        assert_eq!(log.causal_chain(last).count(), 10_000);
        assert_eq!(log.root_cause(last).unwrap().id, root);
    }

    #[test]
    fn ramas_comparten_raiz() {
        let mut log = EventLog::new();
        let root = emit(&mut log, 0, None);
        let a = emit(&mut log, 1, Some(root));
        let b = emit(&mut log, 1, Some(root));
        assert_eq!(log.root_cause(a).unwrap().id, root);
        assert_eq!(log.root_cause(b).unwrap().id, root);
    }

    #[test]
    fn id_inexistente_da_cadena_vacia_y_sin_raiz() {
        let log = EventLog::new();
        let ghost = EventId::from_raw(3);
        assert_eq!(log.causal_chain(ghost).count(), 0);
        assert!(log.root_cause(ghost).is_none());
        assert!(log.get(ghost).is_none());
    }

    #[test]
    fn causa_inexistente_se_rechaza() {
        let mut log = EventLog::new();
        let ghost = EventId::from_raw(0);
        let err = log
            .emit(
                t(0),
                EventKind::LaunchFailed,
                Some(ghost),
                Interrupt::None,
                Payload::new(),
            )
            .unwrap_err();
        assert_eq!(err, EventError::UnknownCause(ghost));
        assert!(log.is_empty());
    }

    #[test]
    fn causa_posterior_se_rechaza() {
        let mut log = EventLog::new();
        let a = emit(&mut log, 100, None);
        let err = log
            .emit(
                t(50),
                EventKind::LaunchFailed,
                Some(a),
                Interrupt::None,
                Payload::new(),
            )
            .unwrap_err();
        assert_eq!(
            err,
            EventError::CauseAfterEvent {
                cause: a,
                cause_time: t(100),
                time: t(50)
            }
        );
        assert_eq!(log.len(), 1);
        // Mismo instante es válido.
        emit(&mut log, 100, Some(a));
    }

    #[test]
    fn el_error_no_consume_id() {
        let mut log = EventLog::new();
        let _ = log.emit(
            t(0),
            EventKind::LaunchFailed,
            Some(EventId::from_raw(9)),
            Interrupt::None,
            Payload::new(),
        );
        assert_eq!(emit(&mut log, 0, None), EventId::from_raw(0));
    }

    #[test]
    fn ids_secuenciales_y_orden_de_insercion() {
        let mut log = EventLog::new();
        let ids: Vec<_> = (0..5).map(|i| emit(&mut log, i, None)).collect();
        let raws: Vec<_> = ids.iter().map(|id| id.raw()).collect();
        assert_eq!(raws, [0, 1, 2, 3, 4]);
        let order: Vec<_> = log.iter().map(|e| e.id).collect();
        assert_eq!(order, ids);
        for id in &ids {
            assert_eq!(log.get(*id).unwrap().id, *id);
        }
    }

    #[test]
    fn append_only_los_eventos_previos_no_cambian() {
        let mut log = EventLog::new();
        let a = emit(&mut log, 1, None);
        let before = log.get(a).unwrap().clone();
        emit(&mut log, 2, Some(a));
        let _ = log.poll_events();
        assert_eq!(log.get(a).unwrap(), &before);
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn poll_no_repite_y_vacio_sin_novedades() {
        let mut log = EventLog::new();
        assert_eq!(log.poll_events().len(), 0);
        let a = emit(&mut log, 0, None);
        let b = emit(&mut log, 1, Some(a));
        assert_eq!(log.pending(), 2);
        let ids: Vec<_> = log.poll_events().iter().map(|e| e.id).collect();
        assert_eq!(ids, [a, b]);
        assert_eq!(log.pending(), 0);
        assert_eq!(log.poll_events().len(), 0);
        let c = emit(&mut log, 2, Some(b));
        let ids: Vec<_> = log.poll_events().iter().map(|e| e.id).collect();
        assert_eq!(ids, [c]);
        assert_eq!(log.poll_events().len(), 0);
        // El log completo sigue consultable.
        assert_eq!(log.len(), 3);
    }

    #[test]
    fn payload_conserva_orden_y_valores() {
        let p = Payload::new()
            .with("body", Value::Text("body.luna".into()))
            .with("n", Value::UInt(3))
            .with("d", Value::Int(-2))
            .with("ok", Value::Bool(true));
        assert_eq!(p.len(), 4);
        assert!(!p.is_empty());
        let keys: Vec<_> = p.iter().map(|(k, _)| k).collect();
        assert_eq!(keys, ["body", "n", "d", "ok"]);
        assert_eq!(p.get("n"), Some(&Value::UInt(3)));
        assert_eq!(p.get("x"), None);
        assert_eq!(p.clone(), p);
    }

    #[test]
    fn el_evento_guarda_kind_interrupt_y_payload() {
        let mut log = EventLog::new();
        let id = log
            .emit(
                t(7),
                EventKind::JumpPointCrossed,
                None,
                Interrupt::Halt,
                Payload::new().with("k", Value::Bool(false)),
            )
            .unwrap();
        let e = log.get(id).unwrap();
        assert_eq!(e.time, t(7));
        assert_eq!(e.kind, EventKind::JumpPointCrossed);
        assert_eq!(e.interrupt, Interrupt::Halt);
        assert_eq!(e.payload.get("k"), Some(&Value::Bool(false)));
        assert_eq!(Interrupt::default(), Interrupt::None);
    }

    #[test]
    fn errores_tienen_texto() {
        let e = EventError::UnknownCause(EventId::from_raw(4));
        assert!(e.to_string().contains('4'));
        assert_ne!(EventError::LogFull.to_string(), "");
    }
}
