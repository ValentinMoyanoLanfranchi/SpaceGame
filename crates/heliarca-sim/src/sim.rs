//! Esqueleto mínimo del estado raíz [`Sim`] (doc 21 §2).
//!
//! **Provisional: esqueleto mínimo; #25 (`M0.2.3.4`) lo completa.** Existe ahora sólo porque
//! [`Command`](crate::command::Command) necesita un `Sim` sobre el que validar y aplicar. Reúne
//! lo que ya hay (semilla y RNG, planificador/tiempo, bus de eventos, cola e historial de
//! comandos) y **no** contiene estado del mundo: cuerpos, naves, escenarios y la API pública
//! definitiva (`Sim::new(scenario, seed)`, `advance`, `poll_events`, consultas) son de #25.
//!
//! El «paso» ([`Sim::step`]) es provisional y mínimo: sólo aplica los comandos pendientes. El
//! avance real del tiempo con interrupciones es de #26/#27.

use crate::command::{
    Command, CommandError, CommandId, CommandOutcome, CommandQueue, CommandRecord,
};
use crate::events::EventLog;
use crate::rng::Rng;
use crate::schedule::Scheduler;
use crate::time::SimTime;

/// Estado raíz de la simulación (esqueleto mínimo; ver la documentación del módulo).
#[derive(Debug)]
pub struct Sim {
    rng: Rng,
    scheduler: Scheduler,
    events: EventLog,
    commands: CommandQueue,
    /// Pasos ejecutados hasta ahora (clave para reproducir un historial de comandos).
    steps: u64,
}

impl Sim {
    /// Crea una simulación vacía a partir de una semilla, en [`SimTime::ZERO`].
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Rng::from_seed(seed),
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

    /// Pasos ejecutados con [`Sim::step`].
    #[must_use]
    pub const fn steps(&self) -> u64 {
        self.steps
    }

    /// Registro de eventos (lectura).
    #[must_use]
    pub const fn events(&self) -> &EventLog {
        &self.events
    }

    /// Registro de eventos (escritura): para los comandos al aplicarse.
    #[cfg_attr(not(test), allow(dead_code))] // Lo usarán los comandos reales; hoy sólo los tests.
    pub(crate) fn events_mut(&mut self) -> &mut EventLog {
        &mut self.events
    }

    /// Planificador (escritura): para los tests; #25/#26 definen el avance público.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn scheduler_mut(&mut self) -> &mut Scheduler {
        &mut self.scheduler
    }

    /// Valida un comando y, si es válido, lo encola. **No lo aplica**: se aplicará al comienzo
    /// del siguiente [`Sim::step`], en orden de envío (FIFO).
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

    /// Historial append-only de los comandos ya procesados, en orden de aplicación.
    #[must_use]
    pub fn command_history(&self) -> &[CommandRecord] {
        self.commands.history()
    }

    /// Paso mínimo **provisional**: aplica los comandos pendientes y cuenta el paso. El avance
    /// real del tiempo (con interrupciones) es de #26/#27, que lo reemplazarán o extenderán.
    pub fn step(&mut self) {
        self.apply_pending_commands();
    }

    /// Aplica, en orden de envío, los comandos encolados antes de este paso.
    ///
    /// Se vacía la cola con [`std::mem::take`] *antes* de aplicar: así `apply(&self, &mut Sim)`
    /// recibe el `Sim` completo sin préstamos pendientes sobre la cola, y un comando que envíe
    /// otro durante su `apply` lo deja para el paso siguiente (nunca se reentra).
    ///
    /// Si un `apply` falla se registra el error en el historial y se **continúa** con el resto.
    pub(crate) fn apply_pending_commands(&mut self) {
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
