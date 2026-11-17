//! Puente con Godot (gdext). La integración con Godot se agrega más adelante.

/// Versión del núcleo de simulación expuesta al lado Godot.
#[must_use]
pub fn sim_version() -> &'static str {
    heliarca_sim::version()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expone_la_version_de_sim() {
        assert_eq!(sim_version(), heliarca_sim::version());
    }
}
