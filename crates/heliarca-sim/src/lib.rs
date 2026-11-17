//! Núcleo de simulación de Heliarca. No depende de Godot.

/// Versión del núcleo de simulación (la del crate).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Devuelve la versión del núcleo de simulación.
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_version_no_esta_vacia() {
        assert!(!version().is_empty());
    }
}
