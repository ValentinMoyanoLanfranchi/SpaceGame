//! Ejecutor sin interfaz de Heliarca.
//!
//! Sólo usa la API pública de `heliarca-sim` (`Sim`, `Scenario`...). La CLI real (subcomandos,
//! argumentos, semilla) es la tarea #39.

use heliarca_sim::{Scenario, Sim};

/// Semilla provisional: la CLI real (#39) la recibirá por argumento.
const DEFAULT_SEED: u64 = 0;

/// Texto que imprime el ejecutable al arrancar: versión del núcleo y fecha inicial de la partida.
fn banner() -> String {
    let sim = Sim::new(Scenario::default(), DEFAULT_SEED);
    let date = sim
        .date()
        .map_or_else(|error| error.to_string(), |date| date.to_string());
    format!("heliarca-cli (sim {}) - {date}", heliarca_sim::version())
}

fn main() {
    println!("{}", banner());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_banner_incluye_la_version_de_sim() {
        assert!(banner().contains(heliarca_sim::version()));
    }

    #[test]
    fn el_banner_incluye_la_fecha_inicial() {
        assert!(banner().ends_with("Año 1, día 1, 00:00:00"));
    }
}
