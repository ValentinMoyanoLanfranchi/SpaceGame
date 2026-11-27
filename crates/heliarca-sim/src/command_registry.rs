//! Registro de comandos: reconstruye un [`Command`] a partir de su `kind` y sus argumentos.
//!
//! El historial ([`crate::CommandRecord`]) guarda cada comando como `kind` + `args` ([`Payload`],
//! datos planos). Los scripts de la CLI (#42) y, más adelante, los guardados y el *autoplayer*
//! necesitan el camino inverso: dado un nombre y unos argumentos, construir el comando. Eso hace
//! [`CommandRegistry::build`]; así el historial se puede reinyectar en un `Sim` nuevo.
//!
//! # Construir no es validar contra el estado
//!
//! `build` sólo comprueba la **forma** de los argumentos (faltan, sobran, tipo o rango). Que el
//! comando tenga sentido en el estado actual lo decide después [`Command::validate`], al enviarlo
//! con [`crate::Sim::submit_command`].
//!
//! # Comandos incluidos
//!
//! Hoy el único es `debug.mark`: **infraestructura de pruebas y diagnóstico, no mecánica del
//! juego**. Con un argumento de texto `label` deja un evento [`EventKind::DebugMark`] (sin
//! interrupción) en el registro de eventos. Sirve para tener algo real que aplicar en scripts y
//! pruebas de reproducción: altera el estado hasheado (un evento y una entrada de historial), pero
//! no tiene ningún efecto sobre el mundo. Los comandos del juego los define el diseño del MVP y
//! se agregarán a [`CommandRegistry::builtin`].
//!
//! ```
//! use heliarca_sim::{CommandRegistry, Payload, Scenario, Sim, Value, Increment};
//!
//! let registry = CommandRegistry::builtin();
//! let args = Payload::new().with("label", Value::Text("inicio".into()));
//! let mut sim = Sim::new(Scenario::default(), 1)?;
//! sim.submit_command(registry.build("debug.mark", &args)?)?;
//! sim.advance(Increment::D1)?;
//! assert_eq!(sim.events().len(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::collections::BTreeMap;

use thiserror::Error;

use crate::command::{Command, CommandError};
use crate::events::{EventKind, Interrupt, Payload, Value};
use crate::sim::Sim;

/// Largo máximo, en caracteres, del `label` de `debug.mark`.
pub const MAX_MARK_LABEL_CHARS: usize = 64;

/// Textos `&'static str` que pueden aparecer dentro de un [`CommandError`] (nombres de argumento
/// de `InvalidArgument` y claves de motivo de `NotAllowedNow`). Un [`CommandError`] guardado en el
/// historial no se puede reconstruir con un texto arbitrario (los campos son `&'static str`), así
/// que el guardado sólo admite los de esta lista. **Al agregar un comando que devuelva
/// `InvalidArgument`/`NotAllowedNow` con un texto nuevo, agregalo acá.**
const KNOWN_STATIC_TEXTS: &[&str] = &["label"];

/// El `&'static str` de [`KNOWN_STATIC_TEXTS`] igual a `text`, si lo hay.
pub(crate) fn known_static_text(text: &str) -> Option<&'static str> {
    KNOWN_STATIC_TEXTS
        .iter()
        .copied()
        .find(|known| *known == text)
}

/// Error al construir un comando desde su `kind` y sus argumentos.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum CommandBuildError {
    /// No hay ningún comando con ese `kind`.
    #[error("comando desconocido: {kind}")]
    UnknownKind {
        /// El `kind` pedido.
        kind: String,
    },
    /// Falta un argumento obligatorio.
    #[error("falta el argumento «{argument}»")]
    MissingArgument {
        /// Nombre del argumento.
        argument: &'static str,
    },
    /// Sobra un argumento que el comando no define.
    #[error("argumento desconocido: «{argument}»")]
    UnexpectedArgument {
        /// Nombre del argumento sobrante.
        argument: String,
    },
    /// Un argumento tiene el tipo equivocado o está fuera de rango.
    #[error("argumento «{argument}» inválido: {reason}")]
    InvalidArgument {
        /// Nombre del argumento.
        argument: &'static str,
        /// Motivo, en texto técnico.
        reason: &'static str,
    },
}

type Builder = fn(&Payload) -> Result<Box<dyn Command>, CommandBuildError>;

/// Catálogo de comandos conocidos: `kind` → constructor desde argumentos.
#[derive(Debug, Clone)]
pub struct CommandRegistry {
    builders: BTreeMap<&'static str, Builder>,
}

impl CommandRegistry {
    /// El catálogo con todos los comandos incluidos en el núcleo.
    #[must_use]
    pub fn builtin() -> Self {
        let mut builders: BTreeMap<&'static str, Builder> = BTreeMap::new();
        builders.insert(DebugMark::KIND, DebugMark::build);
        Self { builders }
    }

    /// Los `kind` conocidos, en orden alfabético.
    pub fn kinds(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.builders.keys().copied()
    }

    /// Construye el comando `kind` con `args`. Es la inversa de [`Command::kind`] +
    /// [`Command::args`]: `build(c.kind(), &c.args())` reconstruye `c`.
    ///
    /// # Errors
    /// [`CommandBuildError`] si el `kind` es desconocido o los argumentos no tienen la forma
    /// esperada (faltan, sobran, tipo o rango).
    pub fn build(&self, kind: &str, args: &Payload) -> Result<Box<dyn Command>, CommandBuildError> {
        let builder = self
            .builders
            .get(kind)
            .ok_or_else(|| CommandBuildError::UnknownKind { kind: kind.into() })?;
        builder(args)
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::builtin()
    }
}

/// `debug.mark`: deja un evento de diagnóstico con un texto (ver el módulo).
#[derive(Debug)]
struct DebugMark {
    label: String,
}

impl DebugMark {
    const KIND: &'static str = "debug.mark";

    fn build(args: &Payload) -> Result<Box<dyn Command>, CommandBuildError> {
        if let Some((extra, _)) = args.iter().find(|(key, _)| *key != "label") {
            return Err(CommandBuildError::UnexpectedArgument {
                argument: extra.into(),
            });
        }
        let label = match args.get("label") {
            None => return Err(CommandBuildError::MissingArgument { argument: "label" }),
            Some(Value::Text(label)) => label,
            Some(_) => {
                return Err(CommandBuildError::InvalidArgument {
                    argument: "label",
                    reason: "debe ser texto",
                });
            }
        };
        let chars = label.chars().count();
        if chars == 0 || chars > MAX_MARK_LABEL_CHARS || label.chars().any(char::is_control) {
            return Err(CommandBuildError::InvalidArgument {
                argument: "label",
                reason: "debe tener de 1 a 64 caracteres, sin caracteres de control",
            });
        }
        Ok(Box::new(Self {
            label: label.clone(),
        }))
    }
}

impl Command for DebugMark {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn args(&self) -> Payload {
        Payload::new().with("label", Value::Text(self.label.clone()))
    }

    fn validate(&self, _sim: &Sim) -> Result<(), CommandError> {
        Ok(())
    }

    fn apply(&self, sim: &mut Sim) -> Result<(), CommandError> {
        let time = sim.time();
        sim.events_mut().emit(
            time,
            EventKind::DebugMark,
            None,
            Interrupt::None,
            self.args(),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn label(text: &str) -> Payload {
        Payload::new().with("label", Value::Text(text.into()))
    }

    #[test]
    fn construye_y_reconstruye_desde_kind_y_args() {
        let registry = CommandRegistry::builtin();
        let command = registry.build("debug.mark", &label("hola")).unwrap();
        assert_eq!(command.kind(), "debug.mark");
        assert_eq!(command.args(), label("hola"));
        let again = registry.build(command.kind(), &command.args()).unwrap();
        assert_eq!(again.args(), command.args());
        assert_eq!(registry.kinds().collect::<Vec<_>>(), ["debug.mark"]);
    }

    #[test]
    fn rechaza_kind_desconocido_y_argumentos_mal_formados() {
        let registry = CommandRegistry::builtin();
        let err = |kind: &str, args: &Payload| registry.build(kind, args).unwrap_err();
        assert_eq!(
            err("nada.nada", &Payload::new()),
            CommandBuildError::UnknownKind {
                kind: "nada.nada".into()
            }
        );
        assert_eq!(
            err("debug.mark", &Payload::new()),
            CommandBuildError::MissingArgument { argument: "label" }
        );
        assert_eq!(
            err("debug.mark", &label("x").with("otro", Value::Bool(true))),
            CommandBuildError::UnexpectedArgument {
                argument: "otro".into()
            }
        );
        let wrong = Payload::new().with("label", Value::UInt(3));
        assert!(matches!(
            err("debug.mark", &wrong),
            CommandBuildError::InvalidArgument {
                argument: "label",
                ..
            }
        ));
        for bad in ["", "a\nb", &"x".repeat(MAX_MARK_LABEL_CHARS + 1)] {
            assert!(matches!(
                err("debug.mark", &label(bad)),
                CommandBuildError::InvalidArgument { .. }
            ));
        }
        // Justo en el límite (en caracteres, no en bytes).
        assert!(
            registry
                .build("debug.mark", &label(&"ñ".repeat(MAX_MARK_LABEL_CHARS)))
                .is_ok()
        );
    }
}
