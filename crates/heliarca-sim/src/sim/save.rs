//! Guardado versionado del estado de [`Sim`] a JSON (issue #50, doc 21 §2).
//!
//! [`Sim::to_save_json`] serializa un `Sim` a un texto JSON y [`Sim::from_save_json`] lo carga de
//! vuelta, de modo que `cargar(guardar(sim))` tiene el mismo [`Sim::state_hash`] que `sim` y,
//! avanzando igual, ambos siguen dando el mismo hash. Escribir/leer archivos, comprimir, la
//! atomicidad (#51), el rechazo de versiones futuras y las migraciones (#52) **no** son de este
//! módulo: trabaja con `&str`/`String`.
//!
//! # Formato (versión 1)
//!
//! Un objeto JSON con las claves **en este orden** (el orden de salida es determinista) y sin
//! claves de más (al cargar, una clave desconocida o faltante es un error):
//!
//! | Clave | Contenido |
//! |---|---|
//! | `format_version` | entero: versión de **este** formato de guardado (hoy `1`; ver [`SAVE_FORMAT_VERSION`]). No es `FORMAT_VERSION` de `state_hash`, la del hash |
//! | `seed` | semilla (`u64`, número) |
//! | `rng_state` | estado interno del RNG: 32 dígitos hexadecimales (`u128`; no cabe en un número JSON) |
//! | `calendar` | fecha de inicio: `year`, `day`, `hour`, `minute`, `second` |
//! | `params` | `days_per_week`, `days_per_month`, `days_per_year` |
//! | `time` | segundos de simulación del planificador |
//! | `steps` | llamadas a `advance` (contabilidad: no entra al hash, pero es observable) |
//! | `events` | `cursor` de `poll_events` y `log`: todos los eventos (entran al hash) |
//! | `commands` | `next_id`, `pending` (cola FIFO) e `history`; cada comando como `kind` + `args` |
//! | `data_fingerprint` | huella de `data/` (32 hex) o `null` |
//! | `state_hash` | `format_version` del hash y `value` (32 hex), para verificar la carga |
//!
//! Los payloads son listas `[{"key": .., "value": {"type": "uint", "value": 5}}]` (orden de
//! inserción conservado). Los `u64` van como números JSON: el formato canónico lo lee Rust; un
//! lector que use `f64` (p. ej. JavaScript) pierde precisión sobre 2^53.
//!
//! # Qué se guarda y qué no
//!
//! Se decidió cada campo de `Sim` y de sus partes con una **desestructuración exhaustiva** (sin
//! `..`) en [`Sim::to_save_json`] y en la construcción de [`Sim::from_save_json`]: agregar un
//! campo sin decidirlo **no compila**. Hoy:
//!
//! - **Se guarda todo lo que entra al hash** (semilla, estado del RNG, calendario, parámetros,
//!   tiempo, log de eventos, cola e historial de comandos, próximo id) **más** `steps` y el
//!   cursor de eventos (observables: un `Sim` cargado debe comportarse igual que el original).
//! - **No se guarda** el mojón de invariantes (`invariant_mark`): es contabilidad derivada; al
//!   cargar se fija en el tiempo actual. Tampoco la copia de `Params` del planificador (se
//!   reconstruye desde los del `Sim`) ni el `start_offset` del calendario (se deriva de `start`).
//! - **Datos estáticos de `data/`**: no se copian; se guarda su **huella** (ver abajo).
//! - **Sistemas del planificador**: son código, no datos. Hoy ninguna API pública los registra;
//!   si un `Sim` tiene alguno, el guardado falla con [`SaveError::SystemsNotSerializable`] en
//!   lugar de perderlos en silencio. Cuando existan sistemas reales los creará la construcción
//!   del `Sim` (M0.3+) y el estado propio de cada uno deberá guardarse aquí.
//!
//! # Huella de `data/`
//!
//! [`GameData::fingerprint`] es un FNV-1a de 128 bits sobre una serialización canónica de los
//! datos **ya cargados** (no de los bytes del archivo: así el espaciado, el orden de claves o los
//! saltos de línea no la cambian). Al guardar y al cargar, el llamador pasa los datos con los que
//! trabaja: `Some(&data)` verifica; si la huella guardada difiere, o el guardado no tiene huella,
//! falla con [`SaveError::DataMismatch`]. **Decisión:** pasar `None` **omite la comprobación**
//! (y guarda `null` al guardar). Hoy un `Sim` no conserva una referencia a `GameData` (sólo toma
//! de ahí los `Params`, que se guardan completos), así que exigirla siempre obligaría a la CLI y
//! a los tests a cargar datos que el `Sim` no usa; el llamador que sí los usa (el juego) debe
//! pasarlos. Cambiar el algoritmo o el contenido que entra a la huella invalida las huellas
//! guardadas: subir `DATA_FINGERPRINT_VERSION` y la versión del guardado.
//!
//! # Comandos
//!
//! Los comandos pendientes se guardan como `kind` + `args` y se reconstruyen con
//! [`CommandRegistry::builtin`]; si no se puede (kind desconocido, argumentos de forma
//! incorrecta, o que `build` no reproduzca exactamente `kind` y `args`) falla con un error claro,
//! **al guardar y al cargar**. El historial se guarda igual (sólo se comprueba que el `kind`
//! exista, porque el historial no se reaplica). Los comandos reconstruidos **no** se vuelven a
//! validar contra el estado: se restauran tal como estaban en la cola. Un [`CommandError`]
//! guardado en el historial sólo puede llevar los textos de `KNOWN_STATIC_TEXTS`
//! (`command_registry`), porque sus campos son `&'static str`.
//!
//! # Verificaciones al cargar
//!
//! Nunca entra en pánico: versión desconocida, JSON malformado o truncado, tipos incorrectos,
//! campos faltantes o sobrantes, valores inválidos (calendario, parámetros, RNG), huella de datos
//! distinta, [`Sim::check_invariants`] (el `Sim` cargado debe cumplirlos) y, si la versión del
//! hash coincide con la actual, que el `state_hash` recalculado sea el guardado.
//!
//! # Agregar estado nuevo al guardado
//!
//! 1. Agregá el campo al tipo (p. ej. `Sim`). La desestructuración exhaustiva de
//!    [`Sim::to_save_json`] (y la del tipo, `save_parts`/`from_save_parts`) deja de compilar:
//!    decidí si se guarda (y cómo) o se descarta con un comentario que lo justifique.
//! 2. Si se guarda: agregalo al DTO correspondiente de este módulo (con `deny_unknown_fields`),
//!    a su conversión en ambos sentidos y a la tabla de arriba. Sumar **claves** cambia el
//!    formato: subí [`SAVE_FORMAT_VERSION`] (y, cuando existan, agregá la migración, #52).
//! 3. Si el estado entra al hash, el hash de un `Sim` cargado debe seguir coincidiendo: el test
//!    de ida y vuelta lo comprueba.

use std::collections::VecDeque;
use std::fmt::Write as _;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value as Json;
use thiserror::Error;

use super::{CommandQueue, EventLog, Mark, Params, Rng, Scheduler, Sim};
use crate::command::{CommandError, CommandId, CommandOutcome, CommandRecord, Queued};
use crate::command_registry::{CommandBuildError, CommandRegistry, known_static_text};
use crate::data::{Body, Entry, GameData, Orbit, OrbitKind};
use crate::events::{Event, EventError, EventId, EventKind, Interrupt, Payload, Value};
use crate::ids::EntityId;
use crate::invariants::Violation;
use crate::params::ParamsError;
use crate::state_hash::{self, StateWriter};
use crate::time::{Calendar, CalendarDate, CalendarError, SimTime};

/// Versión del formato de guardado que escribe este código (la única que sabe leer).
pub const SAVE_FORMAT_VERSION: u32 = 1;

/// Versión del algoritmo de la huella de `data/` (entra a la huella).
const DATA_FINGERPRINT_VERSION: u32 = 1;
/// Marca fija al comienzo del flujo canónico de la huella.
const DATA_FINGERPRINT_MAGIC: &str = "heliarca.data.fingerprint";

/// Huella determinista de los datos de `data/` ya cargados (ver [`GameData::fingerprint`]).
///
/// Valor opaco, comparable por igualdad. Se muestra como 32 dígitos hexadecimales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DataFingerprint(u128);

impl std::fmt::Display for DataFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

/// Error al guardar o cargar un `Sim`. Es `#[non_exhaustive]`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum SaveError {
    /// El `Sim` tiene sistemas registrados en el planificador, que son código y no se guardan.
    #[error("el estado tiene {count} sistema(s) registrado(s): no se pueden guardar")]
    SystemsNotSerializable {
        /// Cantidad de sistemas registrados.
        count: usize,
    },
    /// No se pudo serializar a JSON.
    #[error("no se pudo serializar el guardado: {0}")]
    Serialize(String),
    /// El texto no es JSON, está truncado, o no tiene la forma del formato (tipo incorrecto,
    /// campo faltante o sobrante); el mensaje incluye la ruta del campo.
    #[error("guardado malformado: {0}")]
    Malformed(String),
    /// El guardado no tiene `format_version`.
    #[error("el guardado no tiene `format_version`")]
    MissingFormatVersion,
    /// `format_version` no es una versión que este código sepa leer.
    #[error("versión de guardado desconocida: {found} (se sabe leer la {supported})")]
    UnknownFormatVersion {
        /// Lo que dice el archivo.
        found: String,
        /// La versión que se sabe leer.
        supported: u32,
    },
    /// Un valor del guardado es inválido (calendario, parámetros, estado del RNG, hexadecimal...).
    #[error("campo `{field}` inválido: {reason}")]
    InvalidField {
        /// Campo del guardado.
        field: &'static str,
        /// Qué le pasa.
        reason: String,
    },
    /// Los datos de `data/` no son los del guardado.
    #[error(
        "los datos de data/ no coinciden con los del guardado (guardado: {}, actuales: {current})",
        .saved.as_deref().unwrap_or("sin huella")
    )]
    DataMismatch {
        /// Huella guardada, si el guardado tenía una.
        saved: Option<String>,
        /// Huella de los datos que se pasaron al cargar.
        current: String,
    },
    /// Un comando (pendiente o del historial) tiene un `kind` que el registro no conoce.
    #[error("el comando {id} tiene un kind desconocido: {kind}")]
    UnknownCommandKind {
        /// Id del comando.
        id: u32,
        /// `kind` guardado.
        kind: String,
    },
    /// No se pudo reconstruir un comando pendiente desde su `kind` y `args`.
    #[error("no se pudo reconstruir el comando {id}: {source}")]
    CommandRebuild {
        /// Id del comando.
        id: u32,
        /// Motivo.
        source: CommandBuildError,
    },
    /// El comando reconstruido no reproduce el `kind` y los `args` guardados.
    #[error("el comando {id} reconstruido no coincide con el guardado")]
    CommandMismatch {
        /// Id del comando.
        id: u32,
    },
    /// Un `CommandError` del historial lleva un texto estático que el guardado no conoce.
    #[error("texto estático desconocido en un error de comando: «{text}»")]
    UnknownStaticText {
        /// El texto.
        text: String,
    },
    /// El estado cargado rompe un invariante: el guardado está corrupto o es incoherente.
    #[error("el estado guardado rompe un invariante: {0}")]
    Inconsistent(Violation),
    /// El `state_hash` recalculado tras cargar no es el guardado.
    #[error("el hash del estado cargado ({computed}) no coincide con el guardado ({saved})")]
    HashMismatch {
        /// Hash guardado.
        saved: String,
        /// Hash recalculado.
        computed: String,
    },
}

impl GameData {
    /// Huella determinista del contenido de `data/` ya cargado (FNV-1a de 128 bits sobre una
    /// serialización canónica: enteros little-endian, listas con longitud, campos en un orden
    /// fijo, claves de objeto ordenadas y flotantes por su patrón de bits normalizado).
    ///
    /// Desestructura **sin `..`** cada tipo: un campo nuevo de los datos obliga a decidir si
    /// entra a la huella. No es criptográfica. Ver el módulo `save`.
    #[must_use]
    pub fn fingerprint(&self) -> DataFingerprint {
        let Self {
            bodies,
            goods,
            recipes,
            facilities,
            techs,
            designs,
            companies,
            scenario_earth,
            events,
            loc_es,
            params,
        } = self;
        let mut w = StateWriter::new();
        w.write_str(DATA_FINGERPRINT_MAGIC);
        w.write_u32(DATA_FINGERPRINT_VERSION);
        w.write_len(bodies.len());
        for body in bodies {
            write_body(&mut w, body);
        }
        for entries in [goods, recipes, facilities, techs, designs, companies] {
            write_entries(&mut w, entries);
        }
        write_object(&mut w, scenario_earth.iter());
        write_entries(&mut w, events);
        w.write_len(loc_es.len());
        for (key, text) in loc_es {
            w.write_str(key);
            w.write_str(text);
        }
        for days in params.hash_parts() {
            w.write_u64(days);
        }
        DataFingerprint(w.finish().as_u128())
    }
}

fn write_f64(w: &mut StateWriter, value: f64) {
    // Un solo NaN y -0 como +0: la misma cantidad siempre da los mismos bytes.
    let bits = if value.is_nan() {
        0x7ff8_0000_0000_0000
    } else if value == 0.0 {
        0
    } else {
        value.to_bits()
    };
    w.write_u64(bits);
}

fn write_orbit(w: &mut StateWriter, orbit: &Orbit) {
    let Orbit {
        kind,
        radius_km,
        period_days,
    } = orbit;
    w.write_u8(match kind {
        OrbitKind::Circular => 1,
    });
    write_f64(w, *radius_km);
    write_f64(w, *period_days);
}

fn write_body(w: &mut StateWriter, body: &Body) {
    let Body {
        id,
        kind,
        parent,
        orbit,
        radius_km,
        detect_difficulty,
        habitability,
        deposits,
    } = body;
    w.write_str(id);
    w.write_str(kind);
    w.write_option(parent.as_deref(), StateWriter::write_str);
    w.write_option(orbit.as_ref(), write_orbit);
    write_f64(w, *radius_km);
    w.write_u8(*detect_difficulty);
    w.write_option(*habitability, write_f64);
    w.write_len(deposits.len());
    for deposit in deposits {
        let crate::data::Deposit {
            good,
            reserve_mt,
            access,
        } = deposit;
        w.write_str(good);
        write_f64(w, *reserve_mt);
        write_f64(w, *access);
    }
}

fn write_entries(w: &mut StateWriter, entries: &[Entry]) {
    w.write_len(entries.len());
    for entry in entries {
        let Entry { id, fields } = entry;
        w.write_str(id);
        write_object(w, fields.iter());
    }
}

/// Escribe un objeto JSON con las claves **ordenadas** (no se depende del orden del mapa).
fn write_object<'a>(w: &mut StateWriter, fields: impl Iterator<Item = (&'a String, &'a Json)>) {
    let mut fields: Vec<(&String, &Json)> = fields.collect();
    fields.sort_by(|a, b| a.0.cmp(b.0));
    w.write_len(fields.len());
    for (key, value) in fields {
        w.write_str(key);
        write_json(w, value);
    }
}

fn write_json(w: &mut StateWriter, value: &Json) {
    match value {
        Json::Null => w.write_u8(0),
        Json::Bool(v) => {
            w.write_u8(1);
            w.write_bool(*v);
        }
        Json::Number(n) => {
            if let Some(v) = n.as_u64() {
                w.write_u8(2);
                w.write_u64(v);
            } else if let Some(v) = n.as_i64() {
                w.write_u8(3);
                w.write_i64(v);
            } else {
                w.write_u8(4);
                write_f64(w, n.as_f64().unwrap_or(f64::NAN));
            }
        }
        Json::String(v) => {
            w.write_u8(5);
            w.write_str(v);
        }
        Json::Array(items) => {
            w.write_u8(6);
            w.write_len(items.len());
            for item in items {
                write_json(w, item);
            }
        }
        Json::Object(map) => {
            w.write_u8(7);
            write_object(w, map.iter());
        }
    }
}

// ---- DTO: el formato en disco. `deny_unknown_fields` en todos; el orden de los campos es el
// orden de salida. ----

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveFile {
    format_version: u32,
    seed: u64,
    rng_state: String,
    calendar: CalendarDto,
    params: ParamsDto,
    time: u64,
    steps: u64,
    events: EventsDto,
    commands: CommandsDto,
    #[serde(deserialize_with = "required")]
    data_fingerprint: Option<String>,
    state_hash: StateHashDto,
}

/// Como `Option<T>`, pero la clave es obligatoria (con `null` explícito): un campo faltante
/// no debe confundirse con «ausente».
fn required<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CalendarDto {
    year: u64,
    day: u16,
    hour: u8,
    minute: u8,
    second: u8,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_field_names)] // los nombres son los de `params.json`
struct ParamsDto {
    days_per_week: u64,
    days_per_month: u64,
    days_per_year: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StateHashDto {
    format_version: u32,
    value: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EventsDto {
    cursor: u64,
    log: Vec<EventDto>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EventDto {
    id: u32,
    time: u64,
    kind: EventKindDto,
    #[serde(deserialize_with = "required")]
    cause: Option<u32>,
    interrupt: InterruptDto,
    payload: Vec<FieldDto>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EventKindDto {
    ResearchCompleted,
    LaunchSucceeded,
    LaunchFailed,
    SurveyCompleted,
    ColonyFounded,
    PublicSupportChanged,
    InternalCrisis,
    JumpPointCrossed,
    DebugMark,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum InterruptDto {
    None,
    Halt,
    SlowDown,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldDto {
    key: String,
    value: ValueDto,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
enum ValueDto {
    Int(i64),
    UInt(u64),
    Bool(bool),
    Text(String),
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CommandsDto {
    next_id: u32,
    pending: Vec<PendingDto>,
    history: Vec<RecordDto>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PendingDto {
    id: u32,
    kind: String,
    args: Vec<FieldDto>,
    submitted_at: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordDto {
    id: u32,
    kind: String,
    args: Vec<FieldDto>,
    submitted_at: u64,
    step: u64,
    applied_at: u64,
    outcome: OutcomeDto,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum OutcomeDto {
    Applied,
    Failed { error: CommandErrorDto },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CommandErrorDto {
    InvalidTarget { target: String },
    InvalidArgument { argument: String },
    NotAllowedNow { reason: String },
    EventLog { error: EventErrorDto },
    IdsExhausted,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum EventErrorDto {
    UnknownCause {
        cause: u32,
    },
    CauseAfterEvent {
        cause: u32,
        cause_time: u64,
        time: u64,
    },
    LogFull,
}

// ---- Dominio -> DTO ----

fn hex128(value: u128) -> String {
    let mut out = String::with_capacity(32);
    // Escribir en un `String` no falla.
    let _ = write!(out, "{value:032x}");
    out
}

fn parse_hex128(field: &'static str, text: &str) -> Result<u128, SaveError> {
    let invalid = |reason: &str| SaveError::InvalidField {
        field,
        reason: reason.to_owned(),
    };
    if text.len() != 32 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid("debe ser un hexadecimal de 32 dígitos"));
    }
    u128::from_str_radix(text, 16).map_err(|_| invalid("hexadecimal inválido"))
}

const fn kind_to_dto(kind: EventKind) -> EventKindDto {
    match kind {
        EventKind::ResearchCompleted => EventKindDto::ResearchCompleted,
        EventKind::LaunchSucceeded => EventKindDto::LaunchSucceeded,
        EventKind::LaunchFailed => EventKindDto::LaunchFailed,
        EventKind::SurveyCompleted => EventKindDto::SurveyCompleted,
        EventKind::ColonyFounded => EventKindDto::ColonyFounded,
        EventKind::PublicSupportChanged => EventKindDto::PublicSupportChanged,
        EventKind::InternalCrisis => EventKindDto::InternalCrisis,
        EventKind::JumpPointCrossed => EventKindDto::JumpPointCrossed,
        EventKind::DebugMark => EventKindDto::DebugMark,
    }
}

const fn kind_from_dto(kind: &EventKindDto) -> EventKind {
    match kind {
        EventKindDto::ResearchCompleted => EventKind::ResearchCompleted,
        EventKindDto::LaunchSucceeded => EventKind::LaunchSucceeded,
        EventKindDto::LaunchFailed => EventKind::LaunchFailed,
        EventKindDto::SurveyCompleted => EventKind::SurveyCompleted,
        EventKindDto::ColonyFounded => EventKind::ColonyFounded,
        EventKindDto::PublicSupportChanged => EventKind::PublicSupportChanged,
        EventKindDto::InternalCrisis => EventKind::InternalCrisis,
        EventKindDto::JumpPointCrossed => EventKind::JumpPointCrossed,
        EventKindDto::DebugMark => EventKind::DebugMark,
    }
}

const fn interrupt_to_dto(interrupt: Interrupt) -> InterruptDto {
    match interrupt {
        Interrupt::None => InterruptDto::None,
        Interrupt::Halt => InterruptDto::Halt,
        Interrupt::SlowDown => InterruptDto::SlowDown,
    }
}

const fn interrupt_from_dto(interrupt: &InterruptDto) -> Interrupt {
    match interrupt {
        InterruptDto::None => Interrupt::None,
        InterruptDto::Halt => Interrupt::Halt,
        InterruptDto::SlowDown => Interrupt::SlowDown,
    }
}

fn payload_to_dto(payload: &Payload) -> Vec<FieldDto> {
    payload
        .iter()
        .map(|(key, value)| FieldDto {
            key: key.to_owned(),
            value: match value {
                Value::Int(v) => ValueDto::Int(*v),
                Value::UInt(v) => ValueDto::UInt(*v),
                Value::Bool(v) => ValueDto::Bool(*v),
                Value::Text(v) => ValueDto::Text(v.clone()),
            },
        })
        .collect()
}

fn payload_from_dto(fields: Vec<FieldDto>) -> Payload {
    fields.into_iter().fold(Payload::new(), |payload, field| {
        let value = match field.value {
            ValueDto::Int(v) => Value::Int(v),
            ValueDto::UInt(v) => Value::UInt(v),
            ValueDto::Bool(v) => Value::Bool(v),
            ValueDto::Text(v) => Value::Text(v),
        };
        payload.with(field.key, value)
    })
}

fn event_error_to_dto(error: &EventError) -> EventErrorDto {
    match error {
        EventError::UnknownCause(id) => EventErrorDto::UnknownCause { cause: id.raw() },
        EventError::CauseAfterEvent {
            cause,
            cause_time,
            time,
        } => EventErrorDto::CauseAfterEvent {
            cause: cause.raw(),
            cause_time: cause_time.as_seconds(),
            time: time.as_seconds(),
        },
        EventError::LogFull => EventErrorDto::LogFull,
    }
}

fn event_error_from_dto(error: &EventErrorDto) -> EventError {
    match error {
        EventErrorDto::UnknownCause { cause } => {
            EventError::UnknownCause(EventId::from_raw(*cause))
        }
        EventErrorDto::CauseAfterEvent {
            cause,
            cause_time,
            time,
        } => EventError::CauseAfterEvent {
            cause: EventId::from_raw(*cause),
            cause_time: SimTime::from_seconds(*cause_time),
            time: SimTime::from_seconds(*time),
        },
        EventErrorDto::LogFull => EventError::LogFull,
    }
}

fn static_text(text: &str) -> Result<&'static str, SaveError> {
    known_static_text(text).ok_or_else(|| SaveError::UnknownStaticText {
        text: text.to_owned(),
    })
}

fn command_error_to_dto(error: &CommandError) -> Result<CommandErrorDto, SaveError> {
    Ok(match error {
        CommandError::InvalidTarget { target } => CommandErrorDto::InvalidTarget {
            target: target.clone(),
        },
        CommandError::InvalidArgument { argument } => CommandErrorDto::InvalidArgument {
            argument: static_text(argument)?.to_owned(),
        },
        CommandError::NotAllowedNow { reason } => CommandErrorDto::NotAllowedNow {
            reason: static_text(reason)?.to_owned(),
        },
        CommandError::EventLog(inner) => CommandErrorDto::EventLog {
            error: event_error_to_dto(inner),
        },
        CommandError::IdsExhausted => CommandErrorDto::IdsExhausted,
    })
}

fn command_error_from_dto(error: &CommandErrorDto) -> Result<CommandError, SaveError> {
    Ok(match error {
        CommandErrorDto::InvalidTarget { target } => CommandError::InvalidTarget {
            target: target.clone(),
        },
        CommandErrorDto::InvalidArgument { argument } => CommandError::InvalidArgument {
            argument: static_text(argument)?,
        },
        CommandErrorDto::NotAllowedNow { reason } => CommandError::NotAllowedNow {
            reason: static_text(reason)?,
        },
        CommandErrorDto::EventLog { error } => CommandError::EventLog(event_error_from_dto(error)),
        CommandErrorDto::IdsExhausted => CommandError::IdsExhausted,
    })
}

/// El `kind` estático del registro igual a `kind`.
fn registry_kind(
    registry: &CommandRegistry,
    id: u32,
    kind: &str,
) -> Result<&'static str, SaveError> {
    registry
        .kinds()
        .find(|known| *known == kind)
        .ok_or_else(|| SaveError::UnknownCommandKind {
            id,
            kind: kind.to_owned(),
        })
}

/// Reconstruye un comando pendiente y comprueba que reproduce `kind` y `args` exactos.
fn rebuild_command(
    registry: &CommandRegistry,
    id: u32,
    kind: &str,
    args: &Payload,
) -> Result<Queued, SaveError> {
    registry_kind(registry, id, kind)?;
    let command = registry
        .build(kind, args)
        .map_err(|source| SaveError::CommandRebuild { id, source })?;
    if command.kind() != kind || command.args() != *args {
        return Err(SaveError::CommandMismatch { id });
    }
    Ok(Queued {
        id: CommandId::from_raw(id),
        kind: command.kind(),
        args: args.clone(),
        submitted_at: SimTime::ZERO, // lo fija quien llama
        command,
    })
}

fn malformed(error: &serde_path_to_error::Error<serde_json::Error>) -> SaveError {
    SaveError::Malformed(format!("{}: {}", error.path(), error.inner()))
}

fn field_error(field: &'static str, reason: &(impl ToString + ?Sized)) -> SaveError {
    SaveError::InvalidField {
        field,
        reason: reason.to_string(),
    }
}

fn pending_to_dto(
    pending: &VecDeque<Queued>,
    registry: &CommandRegistry,
) -> Result<Vec<PendingDto>, SaveError> {
    let mut out = Vec::with_capacity(pending.len());
    for queued in pending {
        let Queued {
            id,
            kind,
            args,
            submitted_at,
            command: _reconstruible_desde_kind_y_args,
        } = queued;
        // Debe poder volver a armarse: se comprueba ahora y no recién al cargar.
        rebuild_command(registry, id.raw(), kind, args)?;
        out.push(PendingDto {
            id: id.raw(),
            kind: (*kind).to_owned(),
            args: payload_to_dto(args),
            submitted_at: submitted_at.as_seconds(),
        });
    }
    Ok(out)
}

fn history_to_dto(
    history: &[CommandRecord],
    registry: &CommandRegistry,
) -> Result<Vec<RecordDto>, SaveError> {
    let mut out = Vec::with_capacity(history.len());
    for record in history {
        let CommandRecord {
            id,
            kind,
            args,
            submitted_at,
            step,
            applied_at,
            outcome,
        } = record;
        registry_kind(registry, id.raw(), kind)?;
        out.push(RecordDto {
            id: id.raw(),
            kind: (*kind).to_owned(),
            args: payload_to_dto(args),
            submitted_at: submitted_at.as_seconds(),
            step: *step,
            applied_at: applied_at.as_seconds(),
            outcome: match outcome {
                CommandOutcome::Applied => OutcomeDto::Applied,
                CommandOutcome::Failed(error) => OutcomeDto::Failed {
                    error: command_error_to_dto(error)?,
                },
            },
        });
    }
    Ok(out)
}

fn events_to_dto(events: &[Event]) -> Vec<EventDto> {
    events
        .iter()
        .map(|event| {
            let Event {
                id,
                time,
                kind,
                cause,
                interrupt,
                payload,
            } = event;
            EventDto {
                id: id.raw(),
                time: time.as_seconds(),
                kind: kind_to_dto(*kind),
                cause: cause.map(EventId::raw),
                interrupt: interrupt_to_dto(*interrupt),
                payload: payload_to_dto(payload),
            }
        })
        .collect()
}

fn events_from_dto(events: Vec<EventDto>) -> Vec<Event> {
    events
        .into_iter()
        .map(|event| {
            let EventDto {
                id,
                time,
                kind,
                cause,
                interrupt,
                payload,
            } = event;
            Event {
                id: EventId::from_raw(id),
                time: SimTime::from_seconds(time),
                kind: kind_from_dto(&kind),
                cause: cause.map(EventId::from_raw),
                interrupt: interrupt_from_dto(&interrupt),
                payload: payload_from_dto(payload),
            }
        })
        .collect()
}

type Restored = (VecDeque<Queued>, Vec<CommandRecord>);

fn commands_from_dto(
    pending: Vec<PendingDto>,
    history: Vec<RecordDto>,
    registry: &CommandRegistry,
) -> Result<Restored, SaveError> {
    let mut queue = VecDeque::with_capacity(pending.len());
    for entry in pending {
        let args = payload_from_dto(entry.args);
        let mut queued = rebuild_command(registry, entry.id, &entry.kind, &args)?;
        queued.submitted_at = SimTime::from_seconds(entry.submitted_at);
        queue.push_back(queued);
    }
    let mut records = Vec::with_capacity(history.len());
    for entry in history {
        let outcome = match &entry.outcome {
            OutcomeDto::Applied => CommandOutcome::Applied,
            OutcomeDto::Failed { error } => CommandOutcome::Failed(command_error_from_dto(error)?),
        };
        records.push(CommandRecord {
            id: CommandId::from_raw(entry.id),
            kind: registry_kind(registry, entry.id, &entry.kind)?,
            args: payload_from_dto(entry.args),
            submitted_at: SimTime::from_seconds(entry.submitted_at),
            step: entry.step,
            applied_at: SimTime::from_seconds(entry.applied_at),
            outcome,
        });
    }
    Ok((queue, records))
}

impl Sim {
    /// Serializa el estado a un JSON de guardado (ver el módulo `save` para el formato).
    ///
    /// Pasar `Some(datos)` guarda su huella ([`GameData::fingerprint`]) para comprobarla al
    /// cargar; `None` guarda `null` y la carga no verifica nada. No modifica el `Sim`. La salida
    /// es determinista: el mismo estado da el mismo texto, byte por byte. Cuesta O(tamaño del
    /// estado). Escribirlo a un archivo es trabajo del llamador (#51).
    ///
    /// # Ejemplo
    ///
    /// ```
    /// use heliarca_sim::{Increment, Scenario, Sim};
    ///
    /// let mut sim = Sim::new(Scenario::default(), 42);
    /// sim.advance(Increment::D30)?;
    /// let json = sim.to_save_json(None)?;
    /// let loaded = Sim::from_save_json(&json, None)?;
    /// assert_eq!(loaded.state_hash(), sim.state_hash());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`SaveError::SystemsNotSerializable`] si hay sistemas registrados;
    /// [`SaveError::UnknownCommandKind`], [`SaveError::CommandRebuild`] o
    /// [`SaveError::CommandMismatch`] si un comando pendiente no se puede reconstruir;
    /// [`SaveError::UnknownStaticText`] si un error de comando del historial lleva un texto que
    /// el guardado no admite; [`SaveError::Serialize`] si falla la serialización.
    pub fn to_save_json(&self, data: Option<&GameData>) -> Result<String, SaveError> {
        // Sin `..`: agregar un campo a `Sim` sin decidir si se guarda no compila.
        let Self {
            rng,
            calendar,
            params,
            scheduler,
            events,
            commands,
            steps,
            // Contabilidad derivada (el tiempo al final del último paso): al cargar se fija en
            // el tiempo actual.
            invariant_mark: _mojon_de_invariantes,
        } = self;
        let (time, system_count) = scheduler.save_parts();
        if system_count > 0 {
            return Err(SaveError::SystemsNotSerializable {
                count: system_count,
            });
        }
        let (seed, rng_state) = rng.hash_parts();
        let start = calendar.hash_parts();
        let [days_per_week, days_per_month, days_per_year] = params.hash_parts();
        let (event_list, cursor) = events.save_parts();
        let (next_id, pending, history) = commands.save_parts();

        let registry = CommandRegistry::builtin();
        let pending_dto = pending_to_dto(pending, &registry)?;
        let history_dto = history_to_dto(history, &registry)?;

        let file = SaveFile {
            format_version: SAVE_FORMAT_VERSION,
            seed,
            rng_state: hex128(rng_state),
            calendar: CalendarDto {
                year: start.year,
                day: start.day,
                hour: start.hour,
                minute: start.minute,
                second: start.second,
            },
            params: ParamsDto {
                days_per_week,
                days_per_month,
                days_per_year,
            },
            time: time.as_seconds(),
            steps: *steps,
            events: EventsDto {
                cursor: u64::try_from(cursor)
                    .map_err(|_| field_error("events.cursor", "no cabe en u64"))?,
                log: events_to_dto(event_list),
            },
            commands: CommandsDto {
                next_id,
                pending: pending_dto,
                history: history_dto,
            },
            data_fingerprint: data.map(|data| data.fingerprint().to_string()),
            state_hash: StateHashDto {
                format_version: state_hash::FORMAT_VERSION,
                value: hex128(self.state_hash().as_u128()),
            },
        };
        serde_json::to_string(&file).map_err(|error| SaveError::Serialize(error.to_string()))
    }

    /// Carga un `Sim` desde un JSON de guardado ([`Sim::to_save_json`]). Nunca entra en pánico.
    ///
    /// Con `Some(datos)` comprueba que la huella guardada sea la de esos datos; con `None` no
    /// comprueba nada (ver el módulo `save`). Tras cargar verifica los invariantes y, si la
    /// versión del hash coincide con la actual, que el `state_hash` sea el guardado. El `Sim`
    /// resultante continúa como el original: mismas respuestas de [`Sim::poll_events`], mismo
    /// RNG, misma cola de comandos.
    ///
    /// # Errors
    ///
    /// [`SaveError::Malformed`] (JSON inválido o truncado, tipo incorrecto, campo faltante o
    /// sobrante), [`SaveError::MissingFormatVersion`], [`SaveError::UnknownFormatVersion`],
    /// [`SaveError::InvalidField`], [`SaveError::DataMismatch`], los errores de comandos
    /// ([`SaveError::UnknownCommandKind`], [`SaveError::CommandRebuild`],
    /// [`SaveError::CommandMismatch`], [`SaveError::UnknownStaticText`]),
    /// [`SaveError::Inconsistent`] y [`SaveError::HashMismatch`].
    pub fn from_save_json(json: &str, data: Option<&GameData>) -> Result<Self, SaveError> {
        let raw: Json =
            serde_json::from_str(json).map_err(|error| SaveError::Malformed(error.to_string()))?;
        match raw.get("format_version") {
            None if raw.is_object() => return Err(SaveError::MissingFormatVersion),
            Some(version) if version.as_u64() == Some(u64::from(SAVE_FORMAT_VERSION)) => {}
            Some(version) => {
                return Err(SaveError::UnknownFormatVersion {
                    found: version.to_string(),
                    supported: SAVE_FORMAT_VERSION,
                });
            }
            None => {}
        }
        let file: SaveFile =
            serde_path_to_error::deserialize(raw).map_err(|error| malformed(&error))?;
        let SaveFile {
            format_version: _comprobada_arriba,
            seed,
            rng_state,
            calendar,
            params,
            time,
            steps,
            events,
            commands,
            data_fingerprint,
            state_hash: saved_hash,
        } = file;

        if let Some(data) = data {
            let current = data.fingerprint().to_string();
            if data_fingerprint.as_deref() != Some(current.as_str()) {
                return Err(SaveError::DataMismatch {
                    saved: data_fingerprint,
                    current,
                });
            }
        }

        let rng = Rng::from_parts(seed, parse_hex128("rng_state", &rng_state)?)
            .ok_or_else(|| field_error("rng_state", "el estado del generador debe ser impar"))?;
        let calendar = Calendar::new(CalendarDate {
            year: calendar.year,
            day: calendar.day,
            hour: calendar.hour,
            minute: calendar.minute,
            second: calendar.second,
        })
        .map_err(|error: CalendarError| field_error("calendar", &error))?;
        let params = Params::new(
            params.days_per_week,
            params.days_per_month,
            params.days_per_year,
        )
        .map_err(|error: ParamsError| field_error("params", &error))?;
        let time = SimTime::from_seconds(time);

        let event_list = events_from_dto(events.log);
        let cursor = usize::try_from(events.cursor)
            .map_err(|_| field_error("events.cursor", "no cabe en usize"))?;

        let (pending, history) = commands_from_dto(
            commands.pending,
            commands.history,
            &CommandRegistry::builtin(),
        )?;

        let mut invariant_mark = Mark::default();
        invariant_mark.record(time);
        // Literal sin `..`: un campo nuevo de `Sim` obliga a decidir cómo se restaura.
        let sim = Self {
            rng,
            calendar,
            params,
            scheduler: Scheduler::from_save_parts(time, params),
            events: EventLog::from_save_parts(event_list, cursor),
            commands: CommandQueue::from_save_parts(commands.next_id, pending, history),
            steps,
            invariant_mark,
        };

        sim.check_invariants().map_err(SaveError::Inconsistent)?;
        if saved_hash.format_version == state_hash::FORMAT_VERSION {
            let saved = parse_hex128("state_hash.value", &saved_hash.value)?;
            let computed = sim.state_hash().as_u128();
            if saved != computed {
                return Err(SaveError::HashMismatch {
                    saved: hex128(saved),
                    computed: hex128(computed),
                });
            }
        }
        Ok(sim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;
    use crate::scenario::Scenario;
    use crate::schedule::{Cadence, Priority, StepContext, System, SystemId};
    use crate::time::Increment;

    #[derive(Debug)]
    struct Nada;

    impl System for Nada {
        fn id(&self) -> SystemId {
            SystemId("nada")
        }
        fn priority(&self) -> Priority {
            Priority(0)
        }
        fn cadence(&self) -> Cadence {
            Cadence::Daily
        }
        fn step(&mut self, _: &mut StepContext<'_>) {}
    }

    #[test]
    fn un_sim_con_sistemas_no_se_guarda() {
        let mut sim = Sim::new(Scenario::default(), 1);
        sim.scheduler_mut().register(Box::new(Nada)).unwrap();
        assert_eq!(
            sim.to_save_json(None),
            Err(SaveError::SystemsNotSerializable { count: 1 })
        );
    }

    /// Un comando de prueba que el registro no conoce: no se puede reconstruir.
    #[derive(Debug)]
    struct Ajeno;

    impl Command for Ajeno {
        fn kind(&self) -> &'static str {
            "test.ajeno"
        }
        fn args(&self) -> Payload {
            Payload::new()
        }
        fn validate(&self, _: &Sim) -> Result<(), CommandError> {
            Ok(())
        }
        fn apply(&self, _: &mut Sim) -> Result<(), CommandError> {
            Ok(())
        }
    }

    #[test]
    fn un_comando_que_no_se_puede_reconstruir_falla_al_guardar() {
        let mut sim = Sim::new(Scenario::default(), 1);
        sim.submit_command(Box::new(Ajeno)).unwrap();
        assert!(matches!(
            sim.to_save_json(None),
            Err(SaveError::UnknownCommandKind { id: 0, .. })
        ));
        // Ya aplicado, queda en el historial con un kind desconocido: tampoco se guarda.
        sim.advance(Increment::S5).unwrap();
        assert!(matches!(
            sim.to_save_json(None),
            Err(SaveError::UnknownCommandKind { .. })
        ));
    }

    #[test]
    fn los_errores_de_comando_van_y_vuelven() {
        let errors = [
            CommandError::InvalidTarget {
                target: "body.luna".into(),
            },
            CommandError::InvalidArgument { argument: "label" },
            CommandError::NotAllowedNow { reason: "label" },
            CommandError::EventLog(EventError::LogFull),
            CommandError::EventLog(EventError::UnknownCause(EventId::from_raw(4))),
            CommandError::EventLog(EventError::CauseAfterEvent {
                cause: EventId::from_raw(1),
                cause_time: SimTime::from_seconds(9),
                time: SimTime::from_seconds(3),
            }),
            CommandError::IdsExhausted,
        ];
        for error in errors {
            let dto = command_error_to_dto(&error).unwrap();
            let json = serde_json::to_string(&dto).unwrap();
            let back: CommandErrorDto = serde_json::from_str(&json).unwrap();
            assert_eq!(command_error_from_dto(&back).unwrap(), error, "{json}");
        }
        // Un texto estático que el guardado no conoce se rechaza, al guardar y al cargar.
        let unknown = CommandError::NotAllowedNow {
            reason: "reason.desconocido",
        };
        assert!(matches!(
            command_error_to_dto(&unknown),
            Err(SaveError::UnknownStaticText { .. })
        ));
        let dto = CommandErrorDto::InvalidArgument {
            argument: "otro".into(),
        };
        assert!(matches!(
            command_error_from_dto(&dto),
            Err(SaveError::UnknownStaticText { .. })
        ));
    }

    #[test]
    fn el_formato_del_hash_sigue_en_3() {
        assert_eq!(state_hash::FORMAT_VERSION, 3);
        assert_eq!(SAVE_FORMAT_VERSION, 1);
    }

    #[test]
    fn huella_de_datos_vacios_fija() {
        // GOLDEN: fija el algoritmo de la huella (ver `DATA_FINGERPRINT_VERSION`).
        assert_eq!(
            GameData::default().fingerprint().to_string(),
            "d3c9544e9f89c47605631988173cc2df"
        );
    }
}
