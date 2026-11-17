//! Núcleo de simulación de Heliarca. No depende de Godot.

// Un bloque `unsafe` aquí es un error de compilación (más estricto que `deny`: no admite `#[allow]`).
#![forbid(unsafe_code)]

pub mod arena;
pub mod ids;
pub mod rng;
pub mod schedule;
pub mod time;

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
