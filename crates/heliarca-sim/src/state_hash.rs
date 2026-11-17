//! Hash de estado estable de la simulación (doc 21 §2 «Determinista», issue #28).
//!
//! [`Sim::state_hash`](crate::Sim::state_hash) resume **todo el estado simulado** en 128 bits.
//! Es la base de la repetibilidad («misma semilla + mismos comandos ⇒ mismo resultado»,
//! CONTRIBUTING.md): dos corridas idénticas dan el mismo hash en cada paso, y de ahí cuelgan los
//! tests de repetición, la detección de divergencias y la ida y vuelta de guardado/carga (#50,
//! #53), que comprueba que `cargar(guardar(sim))` tiene el mismo hash que `sim`.
//!
//! # Serialización canónica
//!
//! El hash **no** se calcula sobre la memoria ni sobre `Debug`/`Hash` derivados, sino sobre un
//! flujo de bytes con formato fijo que escribe un `StateWriter`:
//!
//! - enteros en **little-endian** de ancho explícito (nunca `usize`/`isize`: las longitudes van
//!   como `u64`); `bool` como un byte `0`/`1`;
//! - **texto** como longitud (`u64`) + bytes UTF-8; **secuencias** como cantidad (`u64`) +
//!   elementos, de modo que el flujo es autodelimitado y dos estados distintos no pueden dar el
//!   mismo flujo por «correr» un límite;
//! - **variantes** de enum con un **tag `u8` explícito** escrito a mano (un `match`, no
//!   `as u8`): reordenar o agregar variantes en el código no cambia los tags existentes;
//! - **sin flotantes** (el estado actual no los tiene; `Value` no los admite a propósito). Si
//!   algún día hace falta uno, se escribirá su patrón de bits normalizado (un solo `NaN`, `-0`
//!   como `+0`), nunca `f64::to_bits` sin normalizar;
//! - orden de iteración **definido por el modelo** (log de eventos en orden de emisión, cola en
//!   orden FIFO, sistemas por (prioridad, id), campos de un payload en orden de inserción), nunca
//!   el de un `HashMap` (que el crate prohíbe).
//!
//! El flujo empieza con una cabecera (`MAGIC` + `FORMAT_VERSION`) y sigue con secciones, cada
//! una con su tag: núcleo (semilla y estado del RNG), calendario, planificador (tiempo y
//! sistemas), eventos, comandos y parámetros globales (`Params`).
//! Las estructuras se desestructuran **sin `..`** en su módulo (`hash_parts`): agregar un campo
//! sin decidir si se hashea no compila (lo excluido a propósito lleva un comentario).
//!
//! # Algoritmo de hash: FNV-1a de 128 bits
//!
//! Se usa **FNV-1a de 128 bits** (especificación pública de Fowler–Noll–Vo; offset
//! `6c62272e07bb014262b821756295c58d`, primo `2^88 + 0x13b`), implementado a mano con aritmética
//! `u128` envolvente, de modo que es idéntico en cualquier plataforma y versión de Rust y no
//! depende de ninguna semilla de proceso (a diferencia de `DefaultHasher`/`RandomState`). Se eligió
//! sobre FNV-1a de 64 bits (el de `rng::fnv1a_64`, pensado para etiquetas cortas) porque un hash
//! de *estado* se compara en miles de pasos y de partidas: 64 bits tienen una probabilidad de
//! colisión por par de ~2^-64, que con 128 queda fuera de toda preocupación práctica. **No es
//! criptográfico**: sirve para detectar divergencias accidentales, no ataques.
//!
//! # Qué entra al hash
//!
//! - la **semilla** y el **estado interno del RNG** (no sólo la semilla original: dos `Sim` con
//!   la misma semilla pero distinto consumo del generador divergirán, y con semillas distintas el
//!   hash difiere desde el inicio);
//! - el **calendario** (fecha de inicio);
//! - el **tiempo** del planificador y los **sistemas registrados** (id, prioridad, cadencia) más
//!   lo que cada sistema aporte con `System::hash_state`;
//! - el **log de eventos completo** (id, instante, tipo, causa, interrupción, payload);
//! - la **cola de comandos pendientes** (id, tipo, argumentos, instante de envío) y el
//!   **historial** (lo mismo, más paso, instante de aplicación y resultado, con el error
//!   completo) y el contador de ids de comando.
//!
//! # Estado frente a contabilidad de llamadas
//!
//! El criterio para decidir qué se hashea es: **¿es parte de lo que se simuló o de cómo se le
//! pidió avanzar?** El hash cubre el **estado simulado** y el **historial de entradas** del
//! jugador (los comandos), pero **no** la contabilidad de *cómo se partió el avance* en llamadas.
//! Por eso `steps` ([`Sim::steps`](crate::Sim::steps), cuántas veces se llamó a
//! [`Sim::advance`](crate::Sim::advance)) **no entra**: avanzar 30 días como 30 × `D1` o como 1 ×
//! `D30` simula exactamente lo mismo, y los dos `Sim` tienen que tener el mismo hash (es la
//! propiedad de `tests/properties.rs`, issue #45). `Sim::steps()` sigue siendo un accesor
//! observable (por ejemplo, para la interfaz o para correlacionar con el historial).
//!
//! Lo que **sí** depende de la partición y sigue en el hash, a propósito:
//!
//! - `CommandRecord::step` (el índice de la llamada en que se aplicó un comando): es parte del
//!   **historial de entradas** (los comandos son entradas explícitas del jugador y su momento de
//!   aplicación es parte de ellas). Dos corridas con los mismos comandos aplicados en llamadas
//!   distintas son, correctamente, historias distintas. Por eso la propiedad de equivalencia de
//!   particiones se enuncia **sin comandos**.
//! - Los sistemas con cadencia [`Cadence::EveryIncrement`] corren **una vez por llamada**: su
//!   efecto depende de la partición **por diseño** (la cadencia es «por incremento pedido»). La
//!   propiedad vale para sistemas **periódicos** (diario, semanal, mensual, anual), no para
//!   `EveryIncrement`.
//!
//! # Qué NO entra, y por qué
//!
//! - El **contador de pasos** (`Sim::steps`): contabilidad de llamadas, no estado simulado (ver
//!   arriba). Se desestructura nombrado y descartado en `Sim::state_hash`.
//! - El **cursor de consumo** de [`Sim::poll_events`](crate::Sim::poll_events): es un detalle de
//!   la interfaz (cuántos eventos ya se mostraron), no estado simulado; que la UI sondee o no
//!   no debe cambiar la repetibilidad.
//! - El comando en sí (`Box<dyn Command>`) más allá de su `kind` y sus `args`: por contrato
//!   ([`Command::args`](crate::Command::args)) esos datos bastan para reconstruirlo.
//! - El **estado interno de cada `System`**: el trait no lo expone. **Limitación:** un sistema
//!   con estado propio que no lo escriba en `System::hash_state` tiene ese estado fuera del
//!   hash (lo que emita igualmente queda en el log). Es el **punto de extensión** para el estado
//!   del mundo de M0.3+ / M1: cada sistema (o el `Sim`, cuando tenga cuerpos, naves, etc.) escribe
//!   su parte con el `StateWriter`; el estado del mundo debe entrar al hash en un orden
//!   determinista y con un tag de sección propio.
//!
//! # Cambiar el formato es un cambio de versión
//!
//! Cualquier cambio en lo que se escribe o en cómo (orden, tags, anchos, algoritmo) cambia todos
//! los hashes: invalida las repeticiones y los guardados que los hayan registrado. Por eso el
//! formato está versionado (`FORMAT_VERSION`, que entra al hash) y los tests «golden» de este
//! módulo fijan hashes concretos para detectar cualquier cambio accidental. Un cambio deliberado
//! sube `FORMAT_VERSION` y regenera los golden (ver el comentario junto a ellos).
//!
//! # Costo
//!
//! Recorre el estado completo (el log y el historial crecen sin cota): es O(tamaño del estado).
//! Está pensado para tests, puntos de control y guardados, no para llamarse a cada cuadro.

use std::fmt;

use crate::command::{CommandError, CommandOutcome, CommandQueue, CommandRecord, Queued};
use crate::events::{Event, EventError, EventKind, EventLog, Interrupt, Payload, Value};
use crate::ids::EntityId;
use crate::params::Params;
use crate::rng::Rng;
use crate::schedule::{Cadence, Scheduler};
use crate::time::Calendar;

/// Versión del formato canónico. Entra al hash; subirla es un cambio incompatible.
///
/// Historial: 2 = agrega los parámetros globales; 3 = quita `steps` (contabilidad de llamadas,
/// no estado simulado; #45).
pub const FORMAT_VERSION: u32 = 3;

/// Marca fija al comienzo del flujo canónico.
const MAGIC: &str = "heliarca.sim.state";

/// Offset basis de FNV-1a de 128 bits (especificación de Fowler–Noll–Vo).
const FNV128_OFFSET_BASIS: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
/// Primo de FNV de 128 bits: `2^88 + 2^8 + 0x3b`.
const FNV128_PRIME: u128 = (1u128 << 88) + 0x13b;

// Tags de sección.
const SECTION_CORE: u8 = 1;
const SECTION_CALENDAR: u8 = 2;
const SECTION_SCHEDULER: u8 = 3;
const SECTION_EVENTS: u8 = 4;
const SECTION_COMMANDS: u8 = 5;
const SECTION_PARAMS: u8 = 6;

/// Hash de 128 bits del estado de una simulación (ver el módulo `state_hash`).
///
/// Es un valor opaco: sólo se compara por igualdad. Estable entre plataformas y ejecuciones.
/// Se muestra como 32 dígitos hexadecimales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateHash(u128);

impl StateHash {
    /// Valor numérico del hash (p. ej. para registrarlo en un archivo de repetición).
    #[must_use]
    pub const fn as_u128(self) -> u128 {
        self.0
    }

    /// Reconstruye un hash a partir de su valor numérico (p. ej. leído de un archivo).
    #[must_use]
    pub const fn from_u128(value: u128) -> Self {
        Self(value)
    }
}

impl fmt::Display for StateHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

/// Escritor del flujo canónico: acumula bytes directamente en un FNV-1a de 128 bits.
///
/// Lo reciben los sistemas en `System::hash_state` para aportar su estado. Escribir con estos
/// métodos (y no con bytes crudos) garantiza anchos fijos y little-endian.
#[derive(Debug, Clone)]
pub struct StateWriter {
    hash: u128,
}

impl Default for StateWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl StateWriter {
    /// Escritor vacío.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            hash: FNV128_OFFSET_BASIS,
        }
    }

    /// Hash de todo lo escrito hasta ahora.
    #[must_use]
    pub const fn finish(&self) -> StateHash {
        StateHash(self.hash)
    }

    /// Escribe bytes crudos. Preferir los métodos tipados.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.hash ^= u128::from(byte);
            self.hash = self.hash.wrapping_mul(FNV128_PRIME);
        }
    }

    /// Escribe un `u8` (también sirve para tags de variante).
    pub fn write_u8(&mut self, value: u8) {
        self.write_bytes(&[value]);
    }

    /// Escribe un `u16` en little-endian.
    pub fn write_u16(&mut self, value: u16) {
        self.write_bytes(&value.to_le_bytes());
    }

    /// Escribe un `u32` en little-endian.
    pub fn write_u32(&mut self, value: u32) {
        self.write_bytes(&value.to_le_bytes());
    }

    /// Escribe un `u64` en little-endian.
    pub fn write_u64(&mut self, value: u64) {
        self.write_bytes(&value.to_le_bytes());
    }

    /// Escribe un `u128` en little-endian.
    pub fn write_u128(&mut self, value: u128) {
        self.write_bytes(&value.to_le_bytes());
    }

    /// Escribe un `i64` en little-endian (complemento a dos).
    pub fn write_i64(&mut self, value: i64) {
        self.write_bytes(&value.to_le_bytes());
    }

    /// Escribe un booleano como un byte `0`/`1`.
    pub fn write_bool(&mut self, value: bool) {
        self.write_u8(u8::from(value));
    }

    /// Escribe una cantidad o longitud como `u64` (nunca `usize`).
    pub fn write_len(&mut self, len: usize) {
        // `usize` cabe en `u64` en todas las plataformas soportadas; si no, satura.
        self.write_u64(u64::try_from(len).unwrap_or(u64::MAX));
    }

    /// Escribe un texto: longitud en bytes (`u64`) y bytes UTF-8.
    pub fn write_str(&mut self, text: &str) {
        self.write_len(text.len());
        self.write_bytes(text.as_bytes());
    }

    /// Escribe un opcional: tag `0` (ausente) o `1` seguido del valor que escribe `inner`.
    pub fn write_option<T>(&mut self, value: Option<T>, inner: impl FnOnce(&mut Self, T)) {
        match value {
            None => self.write_u8(0),
            Some(value) => {
                self.write_u8(1);
                inner(self, value);
            }
        }
    }
}

fn write_cadence(w: &mut StateWriter, cadence: Cadence) {
    w.write_u8(match cadence {
        Cadence::EveryIncrement => 0,
        Cadence::Daily => 1,
        Cadence::Weekly => 2,
        Cadence::Monthly => 3,
        Cadence::Yearly => 4,
    });
}

fn write_kind(w: &mut StateWriter, kind: EventKind) {
    w.write_u8(match kind {
        EventKind::ResearchCompleted => 1,
        EventKind::LaunchSucceeded => 2,
        EventKind::LaunchFailed => 3,
        EventKind::SurveyCompleted => 4,
        EventKind::ColonyFounded => 5,
        EventKind::PublicSupportChanged => 6,
        EventKind::InternalCrisis => 7,
        EventKind::JumpPointCrossed => 8,
    });
}

fn write_interrupt(w: &mut StateWriter, interrupt: Interrupt) {
    w.write_u8(match interrupt {
        Interrupt::None => 0,
        Interrupt::Halt => 1,
        Interrupt::SlowDown => 2,
    });
}

fn write_value(w: &mut StateWriter, value: &Value) {
    match value {
        Value::Int(v) => {
            w.write_u8(1);
            w.write_i64(*v);
        }
        Value::UInt(v) => {
            w.write_u8(2);
            w.write_u64(*v);
        }
        Value::Bool(v) => {
            w.write_u8(3);
            w.write_bool(*v);
        }
        Value::Text(v) => {
            w.write_u8(4);
            w.write_str(v);
        }
    }
}

fn write_payload(w: &mut StateWriter, payload: &Payload) {
    w.write_len(payload.len());
    for (key, value) in payload.iter() {
        w.write_str(key);
        write_value(w, value);
    }
}

fn write_event(w: &mut StateWriter, event: &Event) {
    // Sin `..`: un campo nuevo de `Event` obliga a decidir si se hashea.
    let Event {
        id,
        time,
        kind,
        cause,
        interrupt,
        payload,
    } = event;
    w.write_u32(id.raw());
    w.write_u64(time.as_seconds());
    write_kind(w, *kind);
    w.write_option(*cause, |w, cause| w.write_u32(cause.raw()));
    write_interrupt(w, *interrupt);
    write_payload(w, payload);
}

fn write_event_error(w: &mut StateWriter, error: &EventError) {
    match error {
        EventError::UnknownCause(id) => {
            w.write_u8(1);
            w.write_u32(id.raw());
        }
        EventError::CauseAfterEvent {
            cause,
            cause_time,
            time,
        } => {
            w.write_u8(2);
            w.write_u32(cause.raw());
            w.write_u64(cause_time.as_seconds());
            w.write_u64(time.as_seconds());
        }
        EventError::LogFull => w.write_u8(3),
    }
}

fn write_command_error(w: &mut StateWriter, error: &CommandError) {
    match error {
        CommandError::InvalidTarget { target } => {
            w.write_u8(1);
            w.write_str(target);
        }
        CommandError::InvalidArgument { argument } => {
            w.write_u8(2);
            w.write_str(argument);
        }
        CommandError::NotAllowedNow { reason } => {
            w.write_u8(3);
            w.write_str(reason);
        }
        CommandError::EventLog(inner) => {
            w.write_u8(4);
            write_event_error(w, inner);
        }
        CommandError::IdsExhausted => w.write_u8(5),
    }
}

fn write_outcome(w: &mut StateWriter, outcome: &CommandOutcome) {
    match outcome {
        CommandOutcome::Applied => w.write_u8(0),
        CommandOutcome::Failed(error) => {
            w.write_u8(1);
            write_command_error(w, error);
        }
    }
}

fn write_calendar(w: &mut StateWriter, calendar: &Calendar) {
    let start = calendar.hash_parts();
    w.write_u64(start.year);
    w.write_u16(start.day);
    w.write_u8(start.hour);
    w.write_u8(start.minute);
    w.write_u8(start.second);
}

fn write_params(w: &mut StateWriter, params: &Params) {
    for days in params.hash_parts() {
        w.write_u64(days);
    }
}

fn write_scheduler(w: &mut StateWriter, scheduler: &Scheduler) {
    let (time, systems) = scheduler.hash_parts();
    w.write_u64(time.as_seconds());
    w.write_len(systems.len());
    for system in systems {
        w.write_str(system.id().0);
        w.write_u16(system.priority().0);
        write_cadence(w, system.cadence());
        // El estado propio del sistema va en un sub-hash: así lo que escriba (o no) no puede
        // desalinear el resto del flujo.
        let mut own = StateWriter::new();
        system.hash_state(&mut own);
        w.write_u128(own.finish().as_u128());
    }
}

fn write_events(w: &mut StateWriter, events: &EventLog) {
    let events = events.hash_parts();
    w.write_len(events.len());
    for event in events {
        write_event(w, event);
    }
}

fn write_commands(w: &mut StateWriter, commands: &CommandQueue) {
    let (next_id, pending, history) = commands.hash_parts();
    w.write_u32(next_id);
    w.write_len(pending.len());
    for queued in pending {
        // Sin `..`. `command` se excluye a propósito: por contrato (`Command::args`) `kind` y
        // `args` bastan para reconstruirlo.
        let Queued {
            id,
            kind,
            args,
            submitted_at,
            command: _reconstruible_desde_kind_y_args,
        } = queued;
        w.write_u32(id.raw());
        w.write_str(kind);
        write_payload(w, args);
        w.write_u64(submitted_at.as_seconds());
    }
    w.write_len(history.len());
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
        w.write_u32(id.raw());
        w.write_str(kind);
        write_payload(w, args);
        w.write_u64(submitted_at.as_seconds());
        w.write_u64(*step);
        w.write_u64(applied_at.as_seconds());
        write_outcome(w, outcome);
    }
}

/// Calcula el hash del estado a partir de las partes de un [`Sim`](crate::Sim).
pub(crate) fn hash_sim(
    rng: &Rng,
    calendar: &Calendar,
    params: &Params,
    scheduler: &Scheduler,
    events: &EventLog,
    commands: &CommandQueue,
) -> StateHash {
    let mut w = StateWriter::new();
    w.write_str(MAGIC);
    w.write_u32(FORMAT_VERSION);

    w.write_u8(SECTION_CORE);
    let (seed, rng_state) = rng.hash_parts();
    w.write_u64(seed);
    w.write_u128(rng_state);

    w.write_u8(SECTION_CALENDAR);
    write_calendar(&mut w, calendar);

    w.write_u8(SECTION_SCHEDULER);
    write_scheduler(&mut w, scheduler);

    w.write_u8(SECTION_EVENTS);
    write_events(&mut w, events);

    w.write_u8(SECTION_COMMANDS);
    write_commands(&mut w, commands);

    w.write_u8(SECTION_PARAMS);
    write_params(&mut w, params);

    w.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;
    use crate::events::EventId;
    use crate::scenario::Scenario;
    use crate::schedule::System;
    use crate::schedule::{Priority, StepContext, SystemId};
    use crate::sim::Sim;
    use crate::time::{CalendarDate, Increment};

    // ---- Vectores del algoritmo ----

    #[test]
    fn fnv1a_128_coincide_con_vectores_publicados() {
        let hash = |bytes: &[u8]| {
            let mut w = StateWriter::new();
            w.write_bytes(bytes);
            w.finish().as_u128()
        };
        assert_eq!(hash(b""), 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d);
        assert_eq!(hash(b"a"), 0xd228_cb69_6f1a_8caf_7891_2b70_4e4a_8964);
    }

    #[test]
    fn la_escritura_es_little_endian_de_ancho_fijo() {
        let bytes = |f: &dyn Fn(&mut StateWriter)| {
            let mut w = StateWriter::new();
            f(&mut w);
            w.finish()
        };
        let mut raw = StateWriter::new();
        raw.write_bytes(&[0x78, 0x56, 0x34, 0x12]);
        assert_eq!(bytes(&|w| w.write_u32(0x1234_5678)), raw.finish());
        let mut raw = StateWriter::new();
        raw.write_bytes(&[2, 0, 0, 0, 0, 0, 0, 0, b'h', b'i']);
        assert_eq!(bytes(&|w| w.write_str("hi")), raw.finish());
        // Una longitud no se confunde con el contenido.
        assert_ne!(
            bytes(&|w| {
                w.write_str("ab");
                w.write_str("c");
            }),
            bytes(&|w| {
                w.write_str("a");
                w.write_str("bc");
            })
        );
    }

    #[test]
    fn display_son_32_digitos_hexadecimales() {
        assert_eq!(
            StateHash::from_u128(0xab).to_string(),
            format!("{:0>32}", "ab")
        );
    }

    // ---- Piezas de prueba ----

    /// Comando de prueba: emite un evento y consume el RNG raíz.
    #[derive(Debug)]
    struct Poke(u64);

    impl Command for Poke {
        fn kind(&self) -> &'static str {
            "test.poke"
        }
        fn args(&self) -> Payload {
            Payload::new().with("n", Value::UInt(self.0))
        }
        fn validate(&self, _: &Sim) -> Result<(), CommandError> {
            Ok(())
        }
        fn apply(&self, sim: &mut Sim) -> Result<(), CommandError> {
            let roll = sim.rng_mut().next_u64() % 1_000;
            let time = sim.time();
            sim.events_mut().emit(
                time,
                EventKind::ResearchCompleted,
                None,
                Interrupt::None,
                Payload::new()
                    .with("n", Value::UInt(self.0))
                    .with("roll", Value::UInt(roll)),
            )?;
            Ok(())
        }
    }

    /// Comando de prueba que pasa la validación y falla al aplicarse.
    #[derive(Debug)]
    struct Fails;

    impl Command for Fails {
        fn kind(&self) -> &'static str {
            "test.fails"
        }
        fn args(&self) -> Payload {
            Payload::new().with("flag", Value::Bool(true))
        }
        fn validate(&self, _: &Sim) -> Result<(), CommandError> {
            Ok(())
        }
        fn apply(&self, _: &mut Sim) -> Result<(), CommandError> {
            Err(CommandError::NotAllowedNow {
                reason: "reason.test",
            })
        }
    }

    /// Sistema de prueba con aleatoriedad propia: un subflujo derivado del RNG del `Sim` por
    /// etiqueta. En cada corrida consume el RNG, emite un evento (encadenado al anterior propio
    /// como causa) y, de vez en cuando, interrumpe. Escribe su RNG en `System::hash_state`.
    #[derive(Debug)]
    struct Noisy {
        id: &'static str,
        cadence: Cadence,
        rng: Rng,
        last: Option<EventId>,
        interrupts: bool,
    }

    impl System for Noisy {
        fn id(&self) -> SystemId {
            SystemId(self.id)
        }
        fn priority(&self) -> Priority {
            Priority(2)
        }
        fn cadence(&self) -> Cadence {
            self.cadence
        }
        fn step(&mut self, ctx: &mut StepContext<'_>) {
            let roll = self.rng.next_u64();
            let interrupt = if !self.interrupts {
                Interrupt::None
            } else if roll.is_multiple_of(41) {
                Interrupt::Halt
            } else if roll.is_multiple_of(7) {
                Interrupt::SlowDown
            } else {
                Interrupt::None
            };
            let payload = Payload::new()
                .with("roll", Value::UInt(roll % 1_000))
                .with(
                    "neg",
                    Value::Int(-i64::from(u32::try_from(roll % 100).unwrap())),
                )
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

    fn noisy(sim: &mut Sim, id: &'static str, cadence: Cadence) {
        noisy_with(sim, id, cadence, true);
    }

    /// Como [`noisy`]; con `interrupts = false` nunca interrumpe el avance.
    fn noisy_with(sim: &mut Sim, id: &'static str, cadence: Cadence, interrupts: bool) {
        let rng = sim.rng().derive(id);
        sim.scheduler_mut()
            .register(Box::new(Noisy {
                id,
                cadence,
                rng,
                last: None,
                interrupts,
            }))
            .unwrap();
    }

    fn new_sim(seed: u64) -> Sim {
        new_sim_with(Scenario::default(), seed)
    }

    fn new_sim_with(scenario: Scenario, seed: u64) -> Sim {
        let mut sim = Sim::new(scenario, seed);
        noisy(&mut sim, "test.diario", Cadence::Daily);
        noisy(&mut sim, "test.semanal", Cadence::Weekly);
        noisy(&mut sim, "test.por_incremento", Cadence::EveryIncrement);
        sim
    }

    /// Operación de una corrida de prueba.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Op {
        Advance(Increment),
        Poke(u64),
        Fails,
    }

    fn apply_op(sim: &mut Sim, op: Op) {
        match op {
            Op::Advance(inc) => {
                sim.advance(inc).unwrap();
            }
            Op::Poke(n) => {
                sim.submit_command(Box::new(Poke(n))).unwrap();
            }
            Op::Fails => {
                sim.submit_command(Box::new(Fails)).unwrap();
            }
        }
    }

    /// Hashes tras cada operación.
    fn run(seed: u64, ops: &[Op]) -> Vec<StateHash> {
        let mut sim = new_sim(seed);
        let mut hashes = vec![sim.state_hash()];
        for &op in ops {
            apply_op(&mut sim, op);
            hashes.push(sim.state_hash());
        }
        hashes
    }

    /// xorshift64 (sin dependencias): operaciones pseudoaleatorias deterministas.
    fn xorshift(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    fn random_ops(seed: u64, count: usize) -> Vec<Op> {
        let mut state = seed | 1;
        let mut next_poke = 0;
        (0..count)
            .map(|_| {
                let r = xorshift(&mut state);
                match r % 10 {
                    0 | 1 => {
                        next_poke += 1;
                        Op::Poke(next_poke)
                    }
                    2 => Op::Fails,
                    // D30 es raro para no recorrer décadas.
                    3..=5 => Op::Advance(Increment::ALL[usize::try_from((r >> 8) % 3).unwrap()]),
                    6 | 7 => Op::Advance(Increment::D1),
                    8 => Op::Advance(Increment::D5),
                    _ => Op::Advance(Increment::ALL[usize::try_from((r >> 8) % 5).unwrap()]),
                }
            })
            .collect()
    }

    // ---- Repetición ----

    /// Criterio del issue: dos corridas con la misma semilla y los mismos comandos coinciden en
    /// CADA paso (1000 operaciones con mezcla de incrementos y comandos).
    #[test]
    fn dos_corridas_identicas_coinciden_en_cada_paso() {
        let ops = random_ops(0xDEAD_BEEF, 1_000);
        let a = run(42, &ops);
        let b = run(42, &ops);
        assert_eq!(a.len(), 1_001);
        assert_eq!(a, b);
        // La corrida es interesante: hubo eventos, interrupciones y comandos fallidos.
        let mut sim = new_sim(42);
        for &op in &ops {
            apply_op(&mut sim, op);
        }
        assert!(sim.events().len() > 100, "{}", sim.events().len());
        assert!(sim.events().iter().any(|e| e.interrupt != Interrupt::None));
        assert!(
            sim.command_history()
                .iter()
                .any(|r| r.outcome != CommandOutcome::Applied)
        );
        assert!(sim.time().as_seconds() > 365 * 86_400, "al menos un año");
    }

    #[test]
    fn un_ano_de_pasos_diarios_coincide_paso_a_paso() {
        let mut ops = Vec::new();
        for day in 0..365 {
            if day % 17 == 0 {
                ops.push(Op::Poke(day));
            }
            ops.push(Op::Advance(Increment::D1));
        }
        assert_eq!(run(7, &ops), run(7, &ops));
    }

    #[test]
    fn cada_paso_del_historial_produce_un_hash_distinto() {
        // Cada operación (avance o envío) cambia el estado: no hay dos hashes iguales seguidos.
        let ops = random_ops(5, 200);
        let hashes = run(3, &ops);
        for pair in hashes.windows(2) {
            assert_ne!(pair[0], pair[1]);
        }
    }

    #[test]
    fn cambiar_la_semilla_cambia_el_hash_desde_el_inicio_y_en_cada_paso() {
        let ops = random_ops(11, 100);
        let a = run(1, &ops);
        let b = run(2, &ops);
        for (x, y) in a.iter().zip(&b) {
            assert_ne!(x, y);
        }
    }

    #[test]
    fn cambiar_la_semilla_cambia_los_eventos_aleatorios() {
        // Misma semilla del escenario no: acá lo que difiere es el contenido aleatorio.
        let ops = [Op::Advance(Increment::D30), Op::Advance(Increment::D30)];
        assert_ne!(run(1, &ops).last().unwrap(), run(2, &ops).last().unwrap());
    }

    // ---- Parámetros globales ----

    fn con_params(days_per_month: u64) -> Scenario {
        Scenario::new().with_params(Params::new(7, days_per_month, 365).unwrap())
    }

    #[test]
    fn los_params_entran_al_hash_desde_el_estado_inicial() {
        let a = Sim::new(con_params(30), 1);
        let b = Sim::new(con_params(30), 1);
        let c = Sim::new(con_params(10), 1);
        assert_eq!(a.state_hash(), b.state_hash());
        assert_ne!(a.state_hash(), c.state_hash());
        // Los params por defecto son los del GDD.
        assert_eq!(
            a.state_hash(),
            Sim::new(Scenario::default(), 1).state_hash()
        );
        // Cada parámetro cuenta.
        let p = |w, m, y| {
            Sim::new(
                Scenario::new().with_params(Params::new(w, m, y).unwrap()),
                1,
            )
            .state_hash()
        };
        assert_ne!(p(7, 30, 365), p(6, 30, 365));
        assert_ne!(p(7, 30, 365), p(7, 30, 366));
    }

    #[test]
    fn params_cargados_de_data_dan_la_misma_secuencia_de_hashes() {
        let data = crate::data::DataLoader::load_dir(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data"),
        )
        .unwrap();
        let ops = [
            Op::Poke(1),
            Op::Advance(Increment::D30),
            Op::Advance(Increment::D5),
            Op::Fails,
            Op::Advance(Increment::D30),
        ];
        let run_with = |scenario: Scenario| {
            let mut sim = new_sim_with(scenario, 9);
            let mut hashes = vec![sim.state_hash()];
            for op in ops {
                apply_op(&mut sim, op);
                hashes.push(sim.state_hash());
            }
            hashes
        };
        let a = run_with(Scenario::from_game_data(&data));
        assert_eq!(a, run_with(Scenario::from_game_data(&data)));
        // `data/params.json` = valores del GDD = escenario por defecto.
        assert_eq!(a, run_with(Scenario::default()));
        // Con otro parámetro la secuencia difiere (también cambia lo que corre).
        assert_ne!(a, run_with(con_params(10)));
    }

    // ---- Sensibilidad ----

    #[test]
    fn el_hash_cambia_al_enviar_un_comando_avanzar_o_emitir_un_evento() {
        let mut sim = new_sim(9);
        let h0 = sim.state_hash();
        sim.submit_command(Box::new(Poke(1))).unwrap();
        let h1 = sim.state_hash();
        assert_ne!(h0, h1, "enviar");
        sim.advance(Increment::S5).unwrap();
        let h2 = sim.state_hash();
        assert_ne!(h1, h2, "avanzar y aplicar");

        let mut sim = Sim::new(Scenario::default(), 9);
        let before = sim.state_hash();
        let time = sim.time();
        sim.events_mut()
            .emit(
                time,
                EventKind::LaunchFailed,
                None,
                Interrupt::None,
                Payload::new(),
            )
            .unwrap();
        assert_ne!(before, sim.state_hash(), "emitir un evento");

        let mut sim = Sim::new(Scenario::default(), 9);
        let before = sim.state_hash();
        sim.advance(Increment::S5).unwrap();
        assert_ne!(before, sim.state_hash(), "sólo el paso y el tiempo");
    }

    #[test]
    fn el_calendario_del_escenario_entra_al_hash() {
        let calendar = Calendar::new(CalendarDate::at_day_start(2, 3)).unwrap();
        let a = Sim::new(Scenario::default(), 1).state_hash();
        let b = Sim::new(Scenario::new().with_calendar(calendar), 1).state_hash();
        assert_ne!(a, b);
    }

    #[test]
    fn el_estado_del_rng_entra_al_hash_aunque_no_cambie_nada_mas() {
        let mut sim = Sim::new(Scenario::default(), 4);
        let before = sim.state_hash();
        let _ = sim.rng_mut().next_u64();
        assert_ne!(before, sim.state_hash());
    }

    #[test]
    fn los_sistemas_registrados_entran_al_hash() {
        let plain = Sim::new(Scenario::default(), 4).state_hash();
        let mut with_system = Sim::new(Scenario::default(), 4);
        noisy(&mut with_system, "test.diario", Cadence::Daily);
        assert_ne!(plain, with_system.state_hash());
        // También su cadencia.
        let mut other = Sim::new(Scenario::default(), 4);
        noisy(&mut other, "test.diario", Cadence::Weekly);
        assert_ne!(with_system.state_hash(), other.state_hash());
    }

    #[test]
    fn el_estado_propio_de_un_sistema_entra_via_hash_state() {
        let mut a = new_sim(4);
        let mut b = new_sim(4);
        assert_eq!(a.state_hash(), b.state_hash());
        // Cada sistema consume su subflujo: el sistema es lo único que difiere entre a y b.
        a.advance(Increment::D1).unwrap();
        b.advance(Increment::D1).unwrap();
        assert_eq!(a.state_hash(), b.state_hash());
        b.advance(Increment::D1).unwrap();
        assert_ne!(a.state_hash(), b.state_hash());
    }

    #[test]
    fn el_orden_de_envio_de_los_comandos_importa() {
        let hash = |first: u64, second: u64| {
            let mut sim = new_sim(1);
            sim.submit_command(Box::new(Poke(first))).unwrap();
            sim.submit_command(Box::new(Poke(second))).unwrap();
            let pending = sim.state_hash();
            sim.advance(Increment::D1).unwrap();
            (pending, sim.state_hash())
        };
        let (p12, h12) = hash(1, 2);
        let (p21, h21) = hash(2, 1);
        assert_ne!(p12, p21, "pendientes");
        assert_ne!(h12, h21, "tras aplicarlos");
    }

    #[test]
    fn el_resultado_y_los_argumentos_de_un_comando_entran_al_hash() {
        let hash = |ok: bool| {
            let mut sim = new_sim(1);
            let command: Box<dyn Command> = if ok {
                Box::new(Poke(0))
            } else {
                Box::new(Fails)
            };
            sim.submit_command(command).unwrap();
            sim.advance(Increment::D1).unwrap();
            sim.state_hash()
        };
        assert_ne!(hash(true), hash(false));
        let with = |n| {
            let mut sim = new_sim(1);
            sim.submit_command(Box::new(Poke(n))).unwrap();
            sim.state_hash()
        };
        assert_ne!(with(1), with(2));
    }

    #[test]
    fn el_cursor_de_poll_events_no_entra_al_hash() {
        let mut a = new_sim(6);
        let mut b = new_sim(6);
        a.advance(Increment::D30).unwrap();
        b.advance(Increment::D30).unwrap();
        assert_ne!(a.poll_events(), []);
        assert_eq!(a.state_hash(), b.state_hash());
    }

    #[test]
    fn los_pasos_son_contabilidad_de_llamadas_y_no_entran_al_hash() {
        // Mismo tiempo simulado, partido en 1 llamada o en 30: `steps` difiere, el hash no.
        // Sólo sistemas periódicos y sin interrupciones (`EveryIncrement` corre una vez por
        // llamada, así que depende de la partición por diseño).
        let periodic = || {
            let mut sim = Sim::new(Scenario::default(), 6);
            noisy_with(&mut sim, "test.diario", Cadence::Daily, false);
            noisy_with(&mut sim, "test.semanal", Cadence::Weekly, false);
            sim
        };
        let (mut a, mut b) = (periodic(), periodic());
        a.advance(Increment::D30).unwrap();
        for _ in 0..30 {
            b.advance(Increment::D1).unwrap();
        }
        assert_eq!((a.steps(), b.steps()), (1, 30));
        assert!(!a.events().is_empty());
        assert_eq!(a.state_hash(), b.state_hash());
        // Pero el tiempo simulado sí cuenta: un día más cambia el hash.
        b.advance(Increment::D1).unwrap();
        assert_ne!(a.state_hash(), b.state_hash());
    }

    #[test]
    fn los_comandos_y_las_cadencias_por_incremento_dependen_de_la_particion() {
        // `CommandRecord::step` es parte del historial de entradas: el mismo comando enviado en
        // el mismo instante pero aplicado en otra llamada es otra historia.
        let hash = |calls_before: &[Increment]| {
            let mut sim = Sim::new(Scenario::default(), 6);
            for &inc in calls_before {
                sim.advance(inc).unwrap();
            }
            sim.submit_command(Box::new(Poke(1))).unwrap();
            sim.advance(Increment::D1).unwrap();
            (sim.time(), sim.state_hash())
        };
        let ((t_a, h_a), (t_b, h_b)) = (hash(&[Increment::D5]), hash(&[Increment::D1; 5]));
        assert_eq!(t_a, t_b);
        assert_ne!(h_a, h_b, "step 1 contra step 5");

        // `EveryIncrement` corre una vez por llamada: 1 × D5 y 5 × D1 no son equivalentes.
        let por_incremento = |calls: &[Increment]| {
            let mut sim = Sim::new(Scenario::default(), 6);
            noisy_with(
                &mut sim,
                "test.por_incremento",
                Cadence::EveryIncrement,
                false,
            );
            for &inc in calls {
                sim.advance(inc).unwrap();
            }
            sim.state_hash()
        };
        assert_ne!(
            por_incremento(&[Increment::D5]),
            por_incremento(&[Increment::D1; 5])
        );
    }
    #[test]
    fn state_hash_es_idempotente_y_no_muta() {
        let mut sim = new_sim(8);
        sim.submit_command(Box::new(Poke(1))).unwrap();
        sim.advance(Increment::D5).unwrap();
        let h = sim.state_hash();
        for _ in 0..5 {
            assert_eq!(sim.state_hash(), h);
        }
        let (steps, events, time) = (sim.steps(), sim.events().len(), sim.time());
        let _ = sim.state_hash();
        assert_eq!(
            (sim.steps(), sim.events().len(), sim.time()),
            (steps, events, time)
        );
        // Y no consume el RNG: la siguiente corrida sigue igual que una que nunca lo llamó.
        let mut other = new_sim(8);
        other.submit_command(Box::new(Poke(1))).unwrap();
        other.advance(Increment::D5).unwrap();
        sim.advance(Increment::D30).unwrap();
        other.advance(Increment::D30).unwrap();
        assert_eq!(sim.state_hash(), other.state_hash());
    }

    // ---- Propiedad con xorshift ----

    #[test]
    fn propiedad_misma_secuencia_mismo_hash_y_una_alteracion_lo_cambia() {
        for round in 1..=20u64 {
            let ops = random_ops(round.wrapping_mul(0x9E37_79B9), 150);
            let base = run(round, &ops);
            assert_eq!(base, run(round, &ops), "ronda {round}");
            let last = base.last().unwrap();

            // Alterar un envío (otro argumento).
            if let Some(i) = ops.iter().position(|op| matches!(op, Op::Poke(_))) {
                let mut altered = ops.clone();
                altered[i] = Op::Poke(9_999);
                let h = run(round, &altered);
                assert_eq!(h[..=i], base[..=i], "ronda {round}: el prefijo no cambia");
                assert_ne!(h.last().unwrap(), last, "ronda {round}: argumento");
            }
            // Quitar la primera operación.
            let mut shorter = ops.clone();
            shorter.remove(0);
            assert_ne!(
                run(round, &shorter).last().unwrap(),
                last,
                "ronda {round}: quitar"
            );
            // Agregar un avance extra.
            let mut longer = ops.clone();
            longer.insert(ops.len() / 2, Op::Advance(Increment::S5));
            assert_ne!(
                run(round, &longer).last().unwrap(),
                last,
                "ronda {round}: agregar"
            );
        }
    }

    // ---- GOLDEN ----

    /// Hash fijo del escenario simple de [`golden_sim`]. Detecta cambios ACCIDENTALES del
    /// formato canónico o del algoritmo (corre igual en Linux y Windows: no hay nada que dependa
    /// de la plataforma).
    ///
    /// Si este test falla, rompiste el determinismo/el formato: revertí el cambio. Si el cambio es
    /// DELIBERADO (nuevo campo en el hash, otro orden, otro algoritmo), es un cambio de versión:
    /// subí `FORMAT_VERSION`, avisá en el PR que invalida repeticiones y guardados previos y
    /// regenerá el valor: `cargo test -p heliarca-sim golden -- --nocapture` imprime el hash
    /// actual (`GOLDEN actual = ...`); pegá el literal acá.
    const GOLDEN_HASH: u128 = 0xebda_10a4_ed7c_ea8a_de8a_b83e_31fc_b310;
    /// Hash del `Sim` recién creado con semilla 42 (sin sistemas), mismo uso.
    const GOLDEN_HASH_INICIAL: u128 = 0x3ab2_acd4_02c7_f6c8_9d71_0a48_0529_a287;

    fn golden_sim() -> Sim {
        let mut sim = new_sim(42);
        let ops = [
            Op::Poke(1),
            Op::Advance(Increment::D5),
            Op::Poke(2),
            Op::Fails,
            Op::Advance(Increment::D1),
            Op::Advance(Increment::H1),
            Op::Advance(Increment::D30),
        ];
        for op in ops {
            apply_op(&mut sim, op);
        }
        sim
    }

    #[test]
    fn golden_hash_de_un_escenario_simple() {
        let actual = golden_sim().state_hash();
        println!("GOLDEN actual = {:#034x}", actual.as_u128());
        assert_eq!(
            actual.as_u128(),
            GOLDEN_HASH,
            "ver el comentario de GOLDEN_HASH"
        );
    }

    #[test]
    fn golden_hash_del_estado_inicial() {
        let actual = Sim::new(Scenario::default(), 42).state_hash();
        println!("GOLDEN inicial = {:#034x}", actual.as_u128());
        assert_eq!(
            actual.as_u128(),
            GOLDEN_HASH_INICIAL,
            "ver el comentario de GOLDEN_HASH"
        );
    }
}
