//! Comandos del jugador: validación, cola e historial (doc 21 §2 «El jugador actúa por
//! comandos»).
//!
//! La UI, la CLI, la IA y el *autoplayer* no tocan el estado: envían [`Command`]s con
//! [`Sim::submit_command`], que los **valida** contra el estado actual y los **encola**. Los
//! comandos se **aplican al comienzo del siguiente paso** ([`Sim::advance`]), en **orden de
//! envío** (FIFO), nunca al enviarlos.
//!
//! # Contrato de un comando
//!
//! - [`Command::validate`] es pura: lee el `Sim` y no lo modifica.
//! - [`Command::apply`] puede fallar porque el estado cambió entre el envío y el paso (p. ej. otro
//!   comando anterior de la misma cola consumió el recurso). Debe **comprobar todo antes de
//!   mutar** y devolver el error sin haber cambiado nada: el `Sim` no puede deshacer efectos
//!   parciales. Si falla, el error queda en el historial ([`CommandOutcome::Failed`]) y el resto
//!   de la cola **se sigue aplicando**.
//! - Un `apply` **no debe entrar en pánico**. Si lo hace, el comando y los que seguían en la cola
//!   de ese paso se pierden **sin registro** en el historial y el paso no se cuenta (los
//!   anteriores del paso sí quedaron aplicados y registrados). Ver [`Sim`] para el detalle.
//! - Durante `validate`/`apply` el `Sim` está en pleno paso: la cola de pendientes está vacía
//!   (salvo lo que se envíe en el propio paso), el historial no incluye el comando en curso y
//!   [`Sim::time`] es el del comienzo del paso. Ver [`Sim`].
//! - Un comando enviado desde dentro de un `apply` queda para el paso siguiente. Aun así, un
//!   comando no debería enviar otros: queda en el historial como un envío más y un reproductor
//!   lo reenviaría dos veces; las consecuencias de un comando son efectos, no comandos nuevos.
//!
//! # Historial y reproducción
//!
//! Cada comando procesado deja un [`CommandRecord`] (append-only, en orden de aplicación) con su
//! `kind` y sus argumentos como [`Payload`] (datos planos, serializables más adelante), el paso
//! y el instante de aplicación y el resultado. Con la misma semilla, reenviar esos registros en
//! los mismos pasos (`record.step`) y en el mismo orden reproduce el mismo estado. Un comando se
//! reconstruye con [`crate::CommandRegistry::build`] a partir de `kind` + `args`; serializar el
//! historial es trabajo posterior. El hash de estado ([`Sim::state_hash`]) cubre cola e historial.
//!
//! Los envíos rechazados por `validate` **no** se registran: no cambiaron nada.
//!
//! # Errores y localización
//!
//! [`CommandError`] usa `thiserror`; su `Display` es texto técnico en español para logs. Lo que
//! ve el jugador se resuelve con [`CommandError::loc_key`] (clave estable de localización, doc
//! 21 §2 «Datos y localización») más los datos de contexto de cada variante.
//!
//! ```
//! use heliarca_sim::{Command, CommandError, CommandOutcome, Increment, Payload, Scenario, Sim};
//!
//! #[derive(Debug)]
//! struct Noop;
//!
//! impl Command for Noop {
//!     fn kind(&self) -> &'static str { "example.noop" }
//!     fn args(&self) -> Payload { Payload::new() }
//!     fn validate(&self, _sim: &Sim) -> Result<(), CommandError> { Ok(()) }
//!     fn apply(&self, _sim: &mut Sim) -> Result<(), CommandError> { Ok(()) }
//! }
//!
//! let mut sim = Sim::new(Scenario::default(), 42)?;
//! sim.submit_command(Box::new(Noop))?;
//! assert_eq!(sim.pending_command_count(), 1); // todavía no se aplicó
//! sim.advance(Increment::S5)?;
//! assert_eq!(sim.command_history()[0].outcome, CommandOutcome::Applied);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::collections::VecDeque;
use std::fmt;

use thiserror::Error;

use crate::define_id;
use crate::events::{EventError, Payload};
use crate::ids::EntityId;
use crate::invariants::{FnInvariant, Violation, entity_command};
use crate::sim::Sim;
use crate::time::SimTime;

define_id! {
    /// Id de un comando aceptado: secuencial desde 0 y nunca reutilizado. Texto: `command.<n>`.
    CommandId, "command"
}

/// Un comando del jugador (o de la IA / el *autoplayer*).
pub trait Command: fmt::Debug {
    /// Tipo de comando: clave estable `dominio.verbo` (p. ej. `research.start`). Va al historial.
    fn kind(&self) -> &'static str;

    /// Argumentos como datos planos, suficientes (con [`Command::kind`]) para reconstruir el
    /// comando al reproducir el historial.
    fn args(&self) -> Payload;

    /// Comprueba, sin modificar nada, que el comando es válido en el estado actual.
    ///
    /// # Errors
    /// El [`CommandError`] que describe por qué se rechaza.
    fn validate(&self, sim: &Sim) -> Result<(), CommandError>;

    /// Aplica el comando. Se llama al comienzo del paso siguiente al envío; el estado pudo
    /// cambiar desde `validate`, así que debe volver a comprobar y, si falla, **no haber
    /// mutado nada** (ver la documentación del módulo).
    ///
    /// # Errors
    /// El [`CommandError`] si ya no se puede aplicar.
    fn apply(&self, sim: &mut Sim) -> Result<(), CommandError>;
}

/// Error de un comando. Cada variante tiene una clave de localización ([`CommandError::loc_key`]).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum CommandError {
    /// El objetivo no existe o no es del tipo esperado.
    #[error("objetivo inválido: {target}")]
    InvalidTarget {
        /// Clave de datos del objetivo (p. ej. `body.luna`) o su descripción.
        target: String,
    },
    /// Un argumento está fuera de rango o mal formado.
    #[error("argumento inválido: {argument}")]
    InvalidArgument {
        /// Nombre del argumento.
        argument: &'static str,
    },
    /// El comando no se puede ejecutar ahora (precondición no cumplida).
    #[error("no permitido ahora: {reason}")]
    NotAllowedNow {
        /// Clave de localización del motivo (p. ej. `reason.already_in_progress`).
        reason: &'static str,
    },
    /// Falló el registro de un evento al aplicar el comando.
    #[error("error en el registro de eventos: {0}")]
    EventLog(#[from] EventError),
    /// Se agotaron los ids de comando (2^32).
    #[error("se agotaron los ids de comando")]
    IdsExhausted,
}

impl CommandError {
    /// Clave de localización estable de la variante (el jugador ve el texto de esa clave).
    #[must_use]
    pub const fn loc_key(&self) -> &'static str {
        match self {
            Self::InvalidTarget { .. } => "command.error.invalid_target",
            Self::InvalidArgument { .. } => "command.error.invalid_argument",
            Self::NotAllowedNow { .. } => "command.error.not_allowed_now",
            Self::EventLog(_) => "command.error.event_log",
            Self::IdsExhausted => "command.error.ids_exhausted",
        }
    }
}

/// Resultado de procesar un comando en su paso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOutcome {
    /// Se aplicó.
    Applied,
    /// Pasó la validación del envío pero falló al aplicarse (no mutó nada).
    Failed(CommandError),
}

/// Entrada del historial de comandos (append-only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandRecord {
    /// Id asignado al aceptar el envío.
    pub id: CommandId,
    /// Tipo de comando ([`Command::kind`]).
    pub kind: &'static str,
    /// Argumentos ([`Command::args`]).
    pub args: Payload,
    /// Instante de simulación del envío.
    pub submitted_at: SimTime,
    /// Índice (desde 0) del paso que lo aplicó: se envió antes de ese paso. Es la clave para
    /// reproducir la partida.
    pub step: u64,
    /// Instante de simulación en que se aplicó.
    pub applied_at: SimTime,
    /// Resultado.
    pub outcome: CommandOutcome,
}

impl CommandRecord {
    pub(crate) fn new(
        queued: Queued,
        step: u64,
        applied_at: SimTime,
        outcome: CommandOutcome,
    ) -> Self {
        Self {
            id: queued.id,
            kind: queued.kind,
            args: queued.args,
            submitted_at: queued.submitted_at,
            step,
            applied_at,
            outcome,
        }
    }
}

/// Comando aceptado a la espera de su paso.
#[derive(Debug)]
pub(crate) struct Queued {
    pub(crate) id: CommandId,
    pub(crate) kind: &'static str,
    pub(crate) args: Payload,
    pub(crate) submitted_at: SimTime,
    pub(crate) command: Box<dyn Command>,
}

/// Invariantes de la cola y el historial de comandos (ver el módulo `invariants`).
pub(crate) const INVARIANTS: &[FnInvariant] = &[
    FnInvariant::new("commands.ids_increasing", check_ids_increasing),
    FnInvariant::new("commands.next_id_above", check_next_id_above),
    FnInvariant::new("commands.history_ordered", check_history_ordered),
    FnInvariant::new("commands.pending_ordered", check_pending_ordered),
];

/// Ids de todos los comandos conocidos en orden de proceso: historial y luego cola.
fn ids_in_order(queue: &CommandQueue) -> impl Iterator<Item = u32> + '_ {
    queue
        .history
        .iter()
        .map(|record| record.id.raw())
        .chain(queue.pending.iter().map(|queued| queued.id.raw()))
}

/// Los ids son estrictamente crecientes en el orden de proceso (historial, luego cola): ninguno
/// se repite ni se reordena.
fn check_ids_increasing(sim: &Sim) -> Result<(), Violation> {
    let mut previous: Option<u32> = None;
    for id in ids_in_order(sim.commands()) {
        if let Some(before) = previous.filter(|&before| id <= before) {
            return Err(Violation::new(
                "commands.ids_increasing",
                Some(entity_command(id)),
                format!(
                    "el comando {} aparece después del {} en el orden de proceso: los ids deben ser estrictamente crecientes",
                    entity_command(id),
                    entity_command(before)
                ),
            ));
        }
        previous = Some(id);
    }
    Ok(())
}

/// `next_id` es mayor que todo id ya asignado (así nunca se reutiliza uno).
fn check_next_id_above(sim: &Sim) -> Result<(), Violation> {
    let next_id = sim.commands().next_id;
    if let Some(id) = ids_in_order(sim.commands()).find(|&id| id >= next_id) {
        return Err(Violation::new(
            "commands.next_id_above",
            Some("commands.next_id".to_owned()),
            format!(
                "el próximo id de comando es {next_id}, pero {} ya fue asignado",
                entity_command(id)
            ),
        ));
    }
    Ok(())
}

/// El historial está en orden de aplicación: `step` y `applied_at` no decrecen, y cada comando
/// se aplicó en o después de su envío.
fn check_history_ordered(sim: &Sim) -> Result<(), Violation> {
    let mut previous: Option<&CommandRecord> = None;
    for record in &sim.commands().history {
        let entity = Some(entity_command(record.id.raw()));
        if record.applied_at < record.submitted_at {
            return Err(Violation::new(
                "commands.history_ordered",
                entity,
                format!(
                    "se aplicó a los {} s, antes de su envío ({} s)",
                    record.applied_at.as_seconds(),
                    record.submitted_at.as_seconds()
                ),
            ));
        }
        if let Some(before) = previous
            && (record.step < before.step || record.applied_at < before.applied_at)
        {
            return Err(Violation::new(
                "commands.history_ordered",
                entity,
                format!(
                    "el historial no está en orden de aplicación: paso {} a los {} s después de {} (paso {}, {} s)",
                    record.step,
                    record.applied_at.as_seconds(),
                    entity_command(before.id.raw()),
                    before.step,
                    before.applied_at.as_seconds()
                ),
            ));
        }
        previous = Some(record);
    }
    Ok(())
}

/// La cola está en orden de envío y nada se envió en el futuro.
fn check_pending_ordered(sim: &Sim) -> Result<(), Violation> {
    let mut previous: Option<SimTime> = None;
    for queued in &sim.commands().pending {
        let entity = Some(entity_command(queued.id.raw()));
        if queued.submitted_at > sim.time() {
            return Err(Violation::new(
                "commands.pending_ordered",
                entity,
                format!(
                    "se envió a los {} s, pero el tiempo actual es {} s",
                    queued.submitted_at.as_seconds(),
                    sim.time().as_seconds()
                ),
            ));
        }
        if previous.is_some_and(|before| queued.submitted_at < before) {
            return Err(Violation::new(
                "commands.pending_ordered",
                entity,
                "la cola no está en orden de envío (instante de envío decreciente)",
            ));
        }
        previous = Some(queued.submitted_at);
    }
    Ok(())
}

/// Cola FIFO de comandos pendientes e historial de los procesados.
#[derive(Debug, Default)]
pub(crate) struct CommandQueue {
    pending: VecDeque<Queued>,
    history: Vec<CommandRecord>,
    next_id: u32,
}

impl CommandQueue {
    /// `(próximo id, pendientes, historial)` para el hash de estado. Desestructura **sin `..`**:
    /// un campo nuevo obliga a decidir si se hashea.
    pub(crate) fn hash_parts(&self) -> (u32, &VecDeque<Queued>, &[CommandRecord]) {
        let Self {
            pending,
            history,
            next_id,
        } = self;
        (*next_id, pending, history)
    }

    /// `(próximo id, pendientes, historial)` para el guardado. Desestructura **sin `..`**: un
    /// campo nuevo obliga a decidir si se guarda.
    pub(crate) fn save_parts(&self) -> (u32, &VecDeque<Queued>, &[CommandRecord]) {
        let Self {
            pending,
            history,
            next_id,
        } = self;
        (*next_id, pending, history)
    }

    /// Reconstruye la cola desde un guardado (sin validar: lo hacen los invariantes al cargar).
    pub(crate) fn from_save_parts(
        next_id: u32,
        pending: VecDeque<Queued>,
        history: Vec<CommandRecord>,
    ) -> Self {
        Self {
            pending,
            history,
            next_id,
        }
    }

    /// Arma la entrada de cola de un comando ya validado y reserva su id.
    pub(crate) fn prepare(
        &mut self,
        command: Box<dyn Command>,
        submitted_at: SimTime,
    ) -> Result<Queued, CommandError> {
        let id = CommandId::from_raw(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(CommandError::IdsExhausted)?;
        Ok(Queued {
            id,
            kind: command.kind(),
            args: command.args(),
            submitted_at,
            command,
        })
    }

    pub(crate) fn push(&mut self, queued: Queued) {
        self.pending.push_back(queued);
    }

    /// Entrega los pendientes, en orden de envío, y deja la cola vacía.
    pub(crate) fn take_pending(&mut self) -> VecDeque<Queued> {
        std::mem::take(&mut self.pending)
    }

    pub(crate) fn record(&mut self, record: CommandRecord) {
        self.history.push(record);
    }

    pub(crate) fn pending_len(&self) -> usize {
        self.pending.len()
    }

    pub(crate) fn history(&self) -> &[CommandRecord] {
        &self.history
    }

    /// Cola, historial y próximo id (escritura): para corromperlos en los tests de invariantes.
    #[cfg(test)]
    pub(crate) fn parts_for_test(
        &mut self,
    ) -> (&mut VecDeque<Queued>, &mut Vec<CommandRecord>, &mut u32) {
        (&mut self.pending, &mut self.history, &mut self.next_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{Event, EventKind, Interrupt, Value};
    use crate::scenario::Scenario;
    use crate::time::Increment;

    fn new_sim(seed: u64) -> Sim {
        Sim::new(Scenario::default(), seed).unwrap()
    }

    /// Un avance mínimo: aplica los pendientes.
    fn tick(sim: &mut Sim) {
        sim.advance(Increment::S5).unwrap();
    }

    // Comandos de PRUEBA (no son comandos del juego). El «estado» observable son los eventos.

    fn marker_exists(sim: &Sim, n: u64) -> bool {
        sim.events()
            .iter()
            .any(|e| e.payload.get("n") == Some(&Value::UInt(n)))
    }

    fn emit_marker(sim: &mut Sim, n: u64) -> Result<(), CommandError> {
        let time = sim.time();
        sim.events_mut().emit(
            time,
            EventKind::ResearchCompleted,
            None,
            Interrupt::None,
            Payload::new().with("n", Value::UInt(n)),
        )?;
        Ok(())
    }

    /// Siempre válido: emite un evento con `n`.
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
            emit_marker(sim, self.0)
        }
    }

    /// Válido sólo si todavía no existe un marcador `n`: dos envíos iguales pasan la validación
    /// del envío, pero el segundo falla al aplicarse.
    #[derive(Debug)]
    struct MarkOnce(u64);

    impl MarkOnce {
        fn check(&self, sim: &Sim) -> Result<(), CommandError> {
            if marker_exists(sim, self.0) {
                Err(CommandError::NotAllowedNow {
                    reason: "reason.already_done",
                })
            } else {
                Ok(())
            }
        }
    }

    impl Command for MarkOnce {
        fn kind(&self) -> &'static str {
            "test.mark_once"
        }
        fn args(&self) -> Payload {
            Payload::new().with("n", Value::UInt(self.0))
        }
        fn validate(&self, sim: &Sim) -> Result<(), CommandError> {
            self.check(sim)
        }
        fn apply(&self, sim: &mut Sim) -> Result<(), CommandError> {
            self.check(sim)?; // comprobar ANTES de mutar
            emit_marker(sim, self.0)
        }
    }

    #[derive(Debug)]
    struct AlwaysInvalid;

    impl Command for AlwaysInvalid {
        fn kind(&self) -> &'static str {
            "test.always_invalid"
        }
        fn args(&self) -> Payload {
            Payload::new()
        }
        fn validate(&self, _: &Sim) -> Result<(), CommandError> {
            Err(CommandError::InvalidTarget {
                target: "body.inexistente".into(),
            })
        }
        fn apply(&self, _: &mut Sim) -> Result<(), CommandError> {
            Err(CommandError::IdsExhausted) // inalcanzable: nunca se encola
        }
    }

    /// Al aplicarse, envía otro comando (que debe esperar al paso siguiente).
    #[derive(Debug)]
    struct Spawner(u64);

    impl Command for Spawner {
        fn kind(&self) -> &'static str {
            "test.spawner"
        }
        fn args(&self) -> Payload {
            Payload::new().with("n", Value::UInt(self.0))
        }
        fn validate(&self, _: &Sim) -> Result<(), CommandError> {
            Ok(())
        }
        fn apply(&self, sim: &mut Sim) -> Result<(), CommandError> {
            sim.submit_command(Box::new(Mark(self.0)))?;
            Ok(())
        }
    }

    fn markers(sim: &Sim) -> Vec<u64> {
        sim.events()
            .iter()
            .filter_map(|e| match e.payload.get("n") {
                Some(Value::UInt(n)) => Some(*n),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn comando_invalido_devuelve_error_y_no_deja_rastro() {
        let mut sim = new_sim(1);
        sim.submit_command(Box::new(Mark(7))).unwrap();
        let events_before: Vec<Event> = sim.events().iter().cloned().collect();

        let err = sim.submit_command(Box::new(AlwaysInvalid)).unwrap_err();
        assert_eq!(
            err,
            CommandError::InvalidTarget {
                target: "body.inexistente".into()
            }
        );
        assert_eq!(err.loc_key(), "command.error.invalid_target");
        assert_eq!(sim.pending_command_count(), 1);
        assert_eq!(sim.command_history().len(), 0);
        assert!(sim.events().iter().cloned().eq(events_before));
        assert_eq!(sim.steps(), 0);

        // No consumió un id: el siguiente envío válido recibe el id 1.
        let id = sim.submit_command(Box::new(Mark(8))).unwrap();
        assert_eq!(id, CommandId::from_raw(1));
    }

    #[test]
    fn enviar_no_aplica_se_aplica_al_comienzo_del_siguiente_paso_en_orden() {
        let mut sim = new_sim(1);
        for n in [3, 1, 2] {
            sim.submit_command(Box::new(Mark(n))).unwrap();
        }
        assert_eq!(sim.pending_command_count(), 3);
        assert!(markers(&sim).is_empty(), "no se aplica al enviar");

        tick(&mut sim);
        assert_eq!(markers(&sim), [3, 1, 2], "orden de envío, no de valor");
        assert_eq!(sim.pending_command_count(), 0);
        let ids: Vec<u32> = sim.command_history().iter().map(|r| r.id.raw()).collect();
        assert_eq!(ids, [0, 1, 2]);
        assert!(
            sim.command_history()
                .iter()
                .all(|r| r.outcome == CommandOutcome::Applied && r.step == 0)
        );
        // Un paso vacío no agrega nada.
        tick(&mut sim);
        assert_eq!(sim.command_history().len(), 3);
        assert_eq!(sim.steps(), 2);
    }

    #[test]
    fn un_comando_enviado_durante_apply_espera_al_paso_siguiente() {
        let mut sim = new_sim(1);
        sim.submit_command(Box::new(Spawner(5))).unwrap();
        tick(&mut sim);
        assert_eq!(markers(&sim).len(), 0);
        assert_eq!(sim.pending_command_count(), 1);
        tick(&mut sim);
        assert_eq!(markers(&sim), [5]);
        let steps: Vec<u64> = sim.command_history().iter().map(|r| r.step).collect();
        assert_eq!(steps, [0, 1]);
    }

    #[test]
    fn fallo_al_aplicar_se_registra_no_muta_y_no_corta_la_cola() {
        let mut sim = new_sim(1);
        // Los tres validan en el envío (todavía no existe ningún marcador).
        sim.submit_command(Box::new(MarkOnce(1))).unwrap();
        sim.submit_command(Box::new(Mark(2))).unwrap();
        sim.submit_command(Box::new(MarkOnce(1))).unwrap();
        sim.submit_command(Box::new(Mark(3))).unwrap();

        tick(&mut sim);
        assert_eq!(markers(&sim), [1, 2, 3], "el fallido no emitió nada");
        let history = sim.command_history();
        assert_eq!(history.len(), 4, "el fallido también queda registrado");
        assert_eq!(history[0].outcome, CommandOutcome::Applied);
        assert_eq!(
            history[2].outcome,
            CommandOutcome::Failed(CommandError::NotAllowedNow {
                reason: "reason.already_done"
            })
        );
        assert_eq!(history[3].outcome, CommandOutcome::Applied);

        // Y una vez aplicado, un envío nuevo ya se rechaza en la validación.
        assert!(sim.submit_command(Box::new(MarkOnce(1))).is_err());
    }

    #[test]
    fn el_historial_registra_kind_args_y_tiempos() {
        let mut sim = new_sim(1);
        sim.submit_command(Box::new(Mark(9))).unwrap();
        sim.advance(Increment::H1).unwrap(); // paso 0: aplica Mark(9) al comienzo (t = 0)
        sim.submit_command(Box::new(Mark(10))).unwrap();
        tick(&mut sim); // paso 1: aplica Mark(10) con t = 3600
        let h = sim.command_history();
        assert_eq!(h[0].kind, "test.mark");
        assert_eq!(h[0].args.get("n"), Some(&Value::UInt(9)));
        assert_eq!(h[0].submitted_at, SimTime::ZERO);
        assert_eq!(
            h[0].applied_at,
            SimTime::ZERO,
            "se aplica al comienzo del paso"
        );
        assert_eq!((h[0].step, h[1].step), (0, 1));
        assert_eq!(h[1].submitted_at, SimTime::from_seconds(3600));
        assert_eq!(h[1].applied_at, SimTime::from_seconds(3600));
    }

    /// Reconstruye un comando de prueba desde su registro (lo que hará un registro de comandos).
    fn rebuild(record: &CommandRecord) -> Box<dyn Command> {
        let n = match record.args.get("n") {
            Some(Value::UInt(n)) => *n,
            _ => unreachable!("los comandos de prueba siempre llevan `n`"),
        };
        match record.kind {
            "test.mark" => Box::new(Mark(n)),
            "test.mark_once" => Box::new(MarkOnce(n)),
            "test.spawner" => Box::new(Spawner(n)),
            other => unreachable!("kind desconocido: {other}"),
        }
    }

    #[test]
    fn el_historial_permite_reproducir_la_partida() {
        // Partida en vivo: comandos entre pasos, con avance de tiempo, un fallo al aplicar y
        // un comando que envía otro. Cada paso es un `advance` con su incremento.
        let increments = [Increment::D1, Increment::H1, Increment::S5, Increment::D5];
        let mut live = new_sim(99);
        live.submit_command(Box::new(Mark(1))).unwrap();
        live.submit_command(Box::new(MarkOnce(2))).unwrap();
        live.submit_command(Box::new(MarkOnce(2))).unwrap(); // fallará al aplicar
        live.advance(increments[0]).unwrap();
        live.submit_command(Box::new(Mark(3))).unwrap();
        assert!(live.submit_command(Box::new(AlwaysInvalid)).is_err());
        live.advance(increments[1]).unwrap();
        live.advance(increments[2]).unwrap();
        live.submit_command(Box::new(Mark(4))).unwrap();
        live.advance(increments[3]).unwrap();

        // Reproducción: misma semilla, mismos incrementos, y cada registro reenviado antes
        // del paso en que se aplicó.
        let mut replay = new_sim(live.seed());
        let history = live.command_history().to_vec();
        for (step, increment) in (0..live.steps()).zip(increments) {
            for record in history.iter().filter(|r| r.step == step) {
                replay.submit_command(rebuild(record)).unwrap();
            }
            replay.advance(increment).unwrap();
        }

        assert_eq!(replay.steps(), live.steps());
        assert_eq!(replay.time(), live.time());
        assert_eq!(replay.command_history(), live.command_history());
        assert!(replay.events().iter().eq(live.events().iter()));
        assert_eq!(markers(&replay), [1, 2, 3, 4]);
    }

    #[test]
    fn claves_de_localizacion_estables_no_vacias_y_unicas() {
        let all = [
            CommandError::InvalidTarget { target: "x".into() },
            CommandError::InvalidArgument { argument: "x" },
            CommandError::NotAllowedNow { reason: "x" },
            CommandError::EventLog(EventError::LogFull),
            CommandError::IdsExhausted,
        ];
        // Si se agrega una variante, esta `match` deja de compilar y obliga a actualizar `all`.
        for e in &all {
            match e {
                CommandError::InvalidTarget { .. }
                | CommandError::InvalidArgument { .. }
                | CommandError::NotAllowedNow { .. }
                | CommandError::EventLog(_)
                | CommandError::IdsExhausted => {}
            }
        }
        let mut keys: Vec<&str> = all.iter().map(CommandError::loc_key).collect();
        for key in &keys {
            assert!(key.starts_with("command.error."), "{key}");
            assert!(
                key.bytes()
                    .all(|b| b.is_ascii_lowercase() || b == b'_' || b == b'.')
            );
            assert!(key.len() > "command.error.".len());
        }
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), all.len());
        for e in &all {
            assert_ne!(e.to_string(), "");
        }
    }

    #[test]
    fn el_error_de_eventos_se_convierte() {
        let e: CommandError = EventError::LogFull.into();
        assert_eq!(e.loc_key(), "command.error.event_log");
    }
}
