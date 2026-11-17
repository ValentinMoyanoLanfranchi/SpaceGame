//! Medición de la duración real de una corrida (reloj de pared).
//!
//! Es el **único** lugar de la CLI que toca `std::time::Instant`. El `clippy.toml` de la raíz
//! prohíbe `Instant`/`SystemTime` porque el reloj del sistema no es determinista y no debe
//! influir en la simulación (el tiempo de juego sale de `SimTime`). Acá es legítimo porque se
//! mide **desde afuera** de la simulación: el valor sólo se imprime (presentación y línea base
//! de rendimiento, doc 21 §1); nunca se le pasa a `heliarca-sim` ni entra al estado ni al hash.
//! La excepción está acotada a este módulo y no se relaja la regla para `heliarca-sim` ni para
//! `heliarca-godot`.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "medir la duración real de la CLI es presentación; no entra a la simulación ni al hash"
)]

use std::time::Instant;

/// Cronómetro de reloj de pared.
#[derive(Debug, Clone, Copy)]
pub struct Stopwatch(Instant);

impl Stopwatch {
    /// Arranca el cronómetro.
    #[must_use]
    pub fn start() -> Self {
        Self(Instant::now())
    }

    /// Milisegundos transcurridos desde [`Stopwatch::start`] (saturando en `u64::MAX`).
    #[must_use]
    pub fn elapsed_millis(&self) -> u64 {
        u64::try_from(self.0.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
}

/// Formatea milisegundos como `<segundos>,<mmm>` (coma decimal, tres decimales, sin redondeo
/// dependiente de la configuración regional): `1234` -> `1,234`.
#[must_use]
pub fn format_seconds(millis: u64) -> String {
    format!("{},{:03}", millis / 1000, millis % 1000)
}

/// Duración para la salida: `<s>,<mmm> s` y, si el tiempo es mayor que 0 ms, ` (<n> pasos/s)`
/// (pasos por segundo, entero y sin separador de miles; saturado, a partir de milisegundos).
#[must_use]
pub fn format_duration(millis: u64, steps: u64) -> String {
    let base = format!("{} s", format_seconds(millis));
    if millis == 0 {
        return base;
    }
    let per_second = u128::from(steps) * 1000 / u128::from(millis);
    format!(
        "{base} ({} pasos/s)",
        u64::try_from(per_second).unwrap_or(u64::MAX)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatea_con_coma_y_tres_decimales() {
        assert_eq!(format_seconds(0), "0,000");
        assert_eq!(format_seconds(7), "0,007");
        assert_eq!(format_seconds(1234), "1,234");
        assert_eq!(format_seconds(61_000), "61,000");
    }

    #[test]
    fn la_duracion_agrega_pasos_por_segundo_si_hay_tiempo() {
        assert_eq!(format_duration(0, 14_600), "0,000 s");
        assert_eq!(format_duration(2, 14_600), "0,002 s (7300000 pasos/s)");
    }

    #[test]
    fn el_cronometro_no_retrocede() {
        let watch = Stopwatch::start();
        assert!(watch.elapsed_millis() < 60_000);
    }
}
