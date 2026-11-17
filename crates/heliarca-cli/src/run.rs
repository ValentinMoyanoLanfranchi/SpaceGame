//! Subcomando `run` (#41): carga los datos, avanza N años y resume el resultado.
//!
//! # Decisiones
//!
//! - **Datos**: se cargan con [`DataLoader::load_validated`] (archivos + registro de ids y
//!   referencias), la misma política completa que `validate-data`: una corrida sobre datos
//!   inconsistentes no tiene sentido. Si falla, se imprimen los errores del cargador y se sale
//!   con 1. El escenario sale de [`Scenario::from_game_data`].
//! - **Segundos por año**: [`SECONDS_PER_YEAR`] (365 días), que es el año del calendario que
//!   muestra [`Sim::date`] hoy; así `--years 40` termina en «Año 41, día 1, 00:00:00». Cuando
//!   el calendario use `params.days_per_year` (issue #439) hay que cambiar [`total_seconds`].
//! - **Bucle**: se avanza con el incremento elegido y, si el resto es menor, con el mayor
//!   incremento que entre (D30 → D5 → D1 → H1 → S5): nunca se pasa del total. Si `advance`
//!   devuelve menos de lo pedido (interrupción) se sigue desde donde quedó, porque la CLI no
//!   tiene a quién preguntarle; si no avanza nada (`advanced == 0`) se corta con error en vez de
//!   girar para siempre.
//! - **Límite**: `--years` va de 0 a [`MAX_YEARS`] (validado por clap: error de uso, código 2).
//!   `--years 0` es válido: 0 pasos.
//! - **Duración real**: ver [`crate::timing`]; mide sólo el bucle de avance (sin la carga de
//!   datos) y no influye en la simulación ni en el hash.

use std::io::Write;
use std::path::Path;

use clap::ValueEnum;
use heliarca_sim::data::{DataLoader, ValidatedLoadError};
use heliarca_sim::{Increment, SECONDS_PER_YEAR, Scenario, Sim};

use crate::CliError;
use crate::timing::{Stopwatch, format_duration};

/// Cantidad máxima de años de `--years`: acota el tiempo de una corrida y evita desbordes.
pub const MAX_YEARS: u32 = 10_000;

/// Años por defecto: el horizonte del criterio de rendimiento (doc 21 §1).
pub const DEFAULT_YEARS: u32 = 40;

/// Incremento elegible por la línea de comandos (`--increment`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum IncrementArg {
    /// 5 segundos.
    S5,
    /// 1 hora.
    H1,
    /// 1 día.
    D1,
    /// 5 días.
    D5,
    /// 30 días.
    D30,
}

impl IncrementArg {
    /// El incremento de la simulación.
    #[must_use]
    pub const fn increment(self) -> Increment {
        match self {
            Self::S5 => Increment::S5,
            Self::H1 => Increment::H1,
            Self::D1 => Increment::D1,
            Self::D5 => Increment::D5,
            Self::D30 => Increment::D30,
        }
    }

    /// Nombre en la línea de comandos.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::S5 => "s5",
            Self::H1 => "h1",
            Self::D1 => "d1",
            Self::D5 => "d5",
            Self::D30 => "d30",
        }
    }
}

/// Segundos de simulación de `years` años; `None` si desborda.
fn total_seconds(years: u32) -> Option<u64> {
    SECONDS_PER_YEAR.checked_mul(u64::from(years))
}

/// Próximo incremento: el elegido si entra en `remaining`; si no, el mayor que entre.
/// `None` sólo si `remaining` es menor que el incremento más chico (S5).
fn next_increment(chosen: Increment, remaining: u64) -> Option<Increment> {
    if chosen.seconds() <= remaining {
        return Some(chosen);
    }
    Increment::ALL
        .into_iter()
        .rev()
        .find(|increment| increment.seconds() <= remaining.min(chosen.seconds()))
}

/// Avanza `sim` exactamente `total` segundos usando `chosen` (ver el módulo).
fn advance_total(sim: &mut Sim, chosen: Increment, total: u64) -> Result<(), CliError> {
    let mut remaining = total;
    while remaining > 0 {
        let Some(increment) = next_increment(chosen, remaining) else {
            return Err(CliError::Sim(format!(
                "el total no es múltiplo del incremento más chico: quedan {remaining} s"
            )));
        };
        let result = sim
            .advance(increment)
            .map_err(|error| CliError::Sim(error.to_string()))?;
        if result.advanced == 0 {
            return Err(CliError::Sim(format!(
                "la simulación no avanza (interrumpida con {remaining} s restantes)"
            )));
        }
        remaining = remaining.saturating_sub(result.advanced);
    }
    Ok(())
}

/// Líneas de error de una carga fallida, una por error.
fn load_error_lines(error: &ValidatedLoadError) -> Vec<String> {
    match error {
        ValidatedLoadError::Load(errors) => errors.0.iter().map(ToString::to_string).collect(),
        ValidatedLoadError::Registry(errors) => errors.0.iter().map(ToString::to_string).collect(),
    }
}

/// Ejecuta `run` y escribe el resumen en `out`.
///
/// # Errors
///
/// [`CliError::Data`] si los datos no cargan o no validan; [`CliError::Sim`]
/// si la simulación falla (desborde, falta de progreso, fecha fuera de rango).
pub fn run(
    data_dir: &Path,
    seed: u64,
    years: u32,
    increment: IncrementArg,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    let validated = DataLoader::load_validated(data_dir)
        .map_err(|error| CliError::Data(load_error_lines(&error)))?;
    let total = total_seconds(years)
        .ok_or_else(|| CliError::Sim("el total de años desborda el tiempo de simulación".into()))?;
    let mut sim = Sim::new(Scenario::from_game_data(&validated.data), seed);

    let watch = Stopwatch::start();
    advance_total(&mut sim, increment.increment(), total)?;
    let elapsed = watch.elapsed_millis();

    let date = sim
        .date()
        .map_err(|error| CliError::Sim(error.to_string()))?;
    // Si stdout está cerrado no hay a quién avisar.
    let _ = write!(
        out,
        "Años simulados: {years}\n\
         Semilla: {seed}\n\
         Incremento: {}\n\
         Pasos: {}\n\
         Fecha final: {date}\n\
         Eventos: {}\n\
         Hash de estado: {}\n\
         Duración real: {}\n",
        increment.name(),
        sim.steps(),
        sim.events().len(),
        sim.state_hash(),
        format_duration(elapsed, sim.steps()),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_ano_son_365_dias() {
        assert_eq!(total_seconds(1), Some(365 * 86_400));
        assert_eq!(total_seconds(0), Some(0));
        assert!(total_seconds(MAX_YEARS).is_some());
    }

    #[test]
    fn el_incremento_se_achica_para_no_pasarse() {
        let d30 = Increment::D30;
        assert_eq!(next_increment(d30, 40 * 86_400), Some(Increment::D30));
        assert_eq!(next_increment(d30, 29 * 86_400), Some(Increment::D5));
        assert_eq!(next_increment(d30, 4 * 86_400), Some(Increment::D1));
        assert_eq!(next_increment(d30, 3_600), Some(Increment::H1));
        assert_eq!(next_increment(d30, 5), Some(Increment::S5));
        assert_eq!(next_increment(d30, 4), None);
        // Nunca sube por encima del elegido.
        assert_eq!(next_increment(Increment::H1, 86_400), Some(Increment::H1));
    }

    #[test]
    fn el_bucle_cubre_justo_el_total() {
        let mut sim = Sim::new(Scenario::default(), 1);
        let total = 400 * 86_400;
        advance_total(&mut sim, Increment::D30, total).unwrap();
        assert_eq!(sim.time().as_seconds(), total);
        // 13 x D30 (390 d) + 2 x D5 (10 d) = 15 pasos.
        assert_eq!(sim.steps(), 15);
    }
}
