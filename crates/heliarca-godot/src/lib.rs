//! Puente con Godot (gdext): punto de entrada de la extensión `heliarca.gdextension`.
//!
//! Sin lógica de juego: sólo usa la API pública de `heliarca-sim` (los internos del núcleo son
//! privados del crate). Las clases expuestas a Godot (`SimHandle`) llegan en M0.7.1.3.

mod extension {
    //! Único lugar del crate con `unsafe`: `ExtensionLibrary` es un trait `unsafe` de gdext (el
    //! contrato es que el nivel de API del binario coincida con el de Godot). El workspace tiene
    //! `unsafe_code = "deny"`; el `allow` va en este módulo porque la expansión de `#[gdextension]`
    //! no respeta un `#[allow]` puesto sobre el `impl`.
    #![allow(unsafe_code)]

    use godot::prelude::*;

    /// Punto de entrada de la extensión. Godot lo localiza por el símbolo `gdext_rust_init`
    /// (`entry_symbol` de `godot/heliarca.gdextension`).
    struct HeliarcaExtension;

    #[gdextension]
    unsafe impl ExtensionLibrary for HeliarcaExtension {
        fn on_stage_init(stage: InitStage) {
            // Evidencia en consola de que la extensión cargó (se ve al abrir el proyecto).
            if stage == InitStage::Scene {
                godot_print!(
                    "Heliarca: extensión cargada (heliarca-sim {})",
                    super::sim_version()
                );
            }
        }
    }
}

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
