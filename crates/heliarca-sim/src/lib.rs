//! Núcleo de simulación de Heliarca. No depende de Godot.
//!
//! # API pública
//!
//! La **única** forma de usar el núcleo desde fuera (la CLI, la extensión de Godot, las
//! pruebas de integración) es [`Sim`]:
//!
//! | Paso | Llamada |
//! |---|---|
//! | Crear | [`Sim::new`]`(`[`Scenario`]`, semilla)` |
//! | Actuar | [`Sim::submit_command`]: valida y encola un [`Command`]; se aplica en el siguiente avance |
//! | Avanzar | [`Sim::advance`]`(`[`Increment`]`)`: aplica los comandos, avanza por tramos y se detiene en la primera interrupción; devuelve un [`AdvanceResult`] |
//! | Leer eventos | [`Sim::poll_events`] (novedades) / [`Sim::events`] (todo el registro) |
//! | Consultar | [`Sim::time`], [`Sim::date`], [`Sim::seed`], [`Sim::params`], [`Sim::steps`], [`Sim::command_history`]... |
//!
//! ```
//! use heliarca_sim::{Command, CommandError, Increment, Payload, Scenario, Sim};
//!
//! #[derive(Debug)]
//! struct Nada;
//!
//! impl Command for Nada {
//!     fn kind(&self) -> &'static str { "ejemplo.nada" }
//!     fn args(&self) -> Payload { Payload::new() }
//!     fn validate(&self, _sim: &Sim) -> Result<(), CommandError> { Ok(()) }
//!     fn apply(&self, _sim: &mut Sim) -> Result<(), CommandError> { Ok(()) }
//! }
//!
//! let mut sim = Sim::new(Scenario::default(), 42);   // crear
//! sim.submit_command(Box::new(Nada))?;                // enviar un comando
//! let result = sim.advance(Increment::D1)?;           // avanzar: lo aplica y pasa un día
//! assert!(result.completed());                        // (no hubo interrupciones)
//! println!("{}", sim.date()?);                         // consultar: «Año 1, día 2, 00:00:00»
//! for event in sim.poll_events() {                    // leer los eventos nuevos
//!     println!("{:?} a los {} s", event.kind, event.time.as_seconds());
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Ver [`Sim`] para el detalle (qué ve un comando durante el paso, qué pasa si un `apply`
//! entra en pánico, orden de los eventos).
//!
//! # Textos para el jugador
//!
//! El código sólo maneja claves; el texto sale de `data/loc/es.json` con [`Loc`]. Una clave que
//! falta se muestra como `¡clave!` (nunca falla):
//!
//! ```
//! use heliarca_sim::{CommandError, Loc};
//!
//! let loc = Loc::from_texts(
//!     [("command.error.ids_exhausted".to_owned(), "Sin órdenes disponibles.".to_owned())].into(),
//! );
//! assert_eq!(loc.command_error(&CommandError::IdsExhausted), "Sin órdenes disponibles.");
//! assert_eq!(loc.get("no.existe"), "¡no.existe!");
//! ```
//!
//! # Qué es interno
//!
//! Los módulos de implementación (cola de comandos, planificador, RNG, arenas, registro de
//! eventos...) son **privados** del crate: la CLI y la extensión de Godot no pueden alcanzarlos,
//! y lo que necesitan se reexporta explícitamente desde aquí. Es la regla «CLI y gdext sólo usan
//! la API pública» (doc 21 §2), hecha cumplir por el compilador. Los siguientes ejemplos no
//! compilan por error de privacidad (`E0603`):
//!
//! ```compile_fail,E0603
//! use heliarca_sim::command::CommandQueue;
//! ```
//!
//! ```compile_fail,E0603
//! use heliarca_sim::schedule::Scheduler;
//! ```
//!
//! ```compile_fail,E0603
//! use heliarca_sim::rng::Rng;
//! ```
//!
//! Tampoco se puede tocar el estado interno de un `Sim` (sus campos son privados):
//!
//! ```compile_fail,E0616
//! let sim = heliarca_sim::Sim::new(heliarca_sim::Scenario::default(), 1);
//! let _ = sim.scheduler;
//! ```
//!
//! Las excepciones son [`ids`] y [`data`], que siguen siendo módulos públicos: los ids tipados
//! ([`ids::BodyId`], [`ids::ShipId`]...), su registro de nombres y la macro [`define_id!`] se
//! usan por la primera ruta, y el cargador de `data/` ([`data::DataLoader`]) por la segunda
//! (lo usan la CLI y los tests de integración sin pasar por el estado interno de la simulación).

// Un bloque `unsafe` aquí es un error de compilación (más estricto que `deny`: no admite `#[allow]`).
#![forbid(unsafe_code)]

// Módulos internos: privados, y se reexporta explícitamente lo que forma la API pública.
// `arena`, `rng` y `schedule` todavía no los usa nada fuera de sus tests (los consumen el
// estado del mundo de M0.3+ y los sistemas): `allow(dead_code)` hasta entonces.
#[allow(dead_code)]
mod arena;
mod command;
pub mod data;
mod events;
pub mod ids;
mod loc;
mod params;
mod registry;
#[allow(dead_code)]
mod rng;
mod scenario;
#[allow(dead_code)]
mod schedule;
mod sim;
mod time;

pub use command::{Command, CommandError, CommandId, CommandOutcome, CommandRecord};
pub use events::{
    CausalChain, Event, EventError, EventId, EventKind, EventLog, Interrupt, Payload, Value,
};
pub use loc::{LOC_DOMAINS, Loc, is_valid_key};
pub use params::{Params, ParamsError};
pub use scenario::Scenario;
pub use sim::{AdvanceError, AdvanceResult, Sim};
pub use time::{
    Calendar, CalendarDate, CalendarError, DAYS_PER_YEAR, FIRST_DAY, FIRST_YEAR, HOURS_PER_DAY,
    Increment, MINUTES_PER_HOUR, SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE,
    SECONDS_PER_YEAR, SimTime,
};

/// Versión del núcleo de simulación (la del crate).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Devuelve la versión del núcleo de simulación.
#[must_use]
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_version_no_esta_vacia() {
        assert_ne!(version(), "");
    }
}
