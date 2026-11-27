//! Puente con Godot (gdext): punto de entrada de la extensión `heliarca.gdextension`.
//!
//! Sin lógica de juego: sólo usa la API pública de `heliarca-sim` (los internos del núcleo son
//! privados del crate). Las clases expuestas a Godot (`SimHandle`) están en el módulo privado `handle`.

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

mod handle {
    //! Clase `SimHandle`: el único estado propio del crate es el `Sim` que posee. Toda regla,
    //! cálculo o texto de juego vive en `heliarca-sim`; acá sólo se envuelve su API pública.

    use godot::prelude::*;
    use heliarca_sim::{CalendarError, Scenario, Sim};

    /// Semilla por defecto de un `SimHandle` recién creado.
    const DEFAULT_SEED: u64 = 0;

    /// Texto de saludo con la versión del núcleo y su fecha actual. Función pura: no depende de
    /// Godot, así se prueba con `cargo test`.
    pub(super) fn hello_text(sim: &Sim) -> String {
        let date = match sim.date() {
            Ok(date) => date.to_string(),
            Err(error) => format!("fecha no disponible ({error:?})"),
        };
        format!("Heliarca (sim {}): {date}", super::sim_version())
    }

    /// Simulación del escenario por defecto con la semilla dada.
    fn new_sim(seed: u64) -> Result<Sim, CalendarError> {
        Sim::new(Scenario::default(), seed)
    }

    /// Manejador de la simulación para `GDScript`. Base `RefCounted`: no necesita estar en el árbol
    /// de escenas y Godot lo libera solo cuando nadie lo referencia.
    #[derive(GodotClass)]
    #[class(base = RefCounted)]
    pub struct SimHandle {
        /// `Err` si el escenario no se pudo crear: se informa en el saludo, sin entrar en pánico.
        sim: Result<Sim, CalendarError>,
    }

    #[godot_api]
    impl IRefCounted for SimHandle {
        fn init(_base: Base<RefCounted>) -> Self {
            Self {
                sim: new_sim(DEFAULT_SEED),
            }
        }
    }

    #[godot_api]
    impl SimHandle {
        /// Reinicia la simulación con otra semilla (los enteros negativos se reinterpretan como
        /// `u64`).
        #[func]
        fn set_seed(&mut self, seed: i64) {
            self.sim = new_sim(seed.cast_unsigned());
        }

        /// Texto de saludo que viene del núcleo (ver `hello_text`).
        #[func]
        fn hello(&self) -> GString {
            let text = match &self.sim {
                Ok(sim) => hello_text(sim),
                Err(error) => format!("Heliarca: simulación no disponible ({error})"),
            };
            GString::from(&text)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn hello_incluye_version_y_fecha_inicial() {
            let sim = new_sim(DEFAULT_SEED).unwrap();
            assert_eq!(
                hello_text(&sim),
                format!(
                    "Heliarca (sim {}): Año 1, día 1, 00:00:00",
                    heliarca_sim::version()
                )
            );
        }

        #[test]
        fn hello_no_depende_de_la_semilla() {
            let a = hello_text(&new_sim(1).unwrap());
            let b = hello_text(&new_sim(99).unwrap());
            assert_eq!(a, b);
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
