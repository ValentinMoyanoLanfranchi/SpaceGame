//! Subcomando `run` (#41): carga los datos, avanza N años y resume el resultado.
//!
//! # Decisiones
//!
//! - **Datos**: se cargan con [`DataLoader::load_validated`] (archivos + registro de ids y
//!   referencias), la misma política completa que `validate-data`: una corrida sobre datos
//!   inconsistentes no tiene sentido. Si falla, se imprimen los errores del cargador y se sale
//!   con 1. El escenario sale de [`Scenario::from_game_data`].
//! - **Segundos por año**: `days_per_year` días de los parámetros (`data/params.json`, o los del
//!   guardado en `load`), que es el año del calendario que muestra [`Sim::date`]; así `--years 40`
//!   termina en «Año 41, día 1, 00:00:00».
//! - **Bucle**: se avanza con el incremento elegido y, si el resto es menor, con el mayor
//!   incremento que entre (D30 → D5 → D1 → H1 → S5): nunca se pasa del total. Si `advance`
//!   devuelve menos de lo pedido (interrupción) se sigue desde donde quedó, porque la CLI no
//!   tiene a quién preguntarle; si no avanza nada (`advanced == 0`) se corta con error en vez de
//!   girar para siempre.
//! - **Límite**: `--years` va de 0 a [`MAX_YEARS`] (validado por clap: error de uso, código 2).
//!   `--years 0` es válido: 0 pasos.
//! - **Guardado (`--save`) y `load`**: ver [`finish`] y [`load`]. El guardado se escribe al final,
//!   con escritura atómica de `heliarca-sim`, pasando los datos para fijar su huella.
//! - **Duración real**: ver [`crate::timing`]; mide sólo el bucle de avance (sin la carga de
//!   datos) y no influye en la simulación ni en el hash.

use std::io::Write;
use std::path::Path;

use clap::ValueEnum;
use heliarca_sim::data::{DataLoader, GameData, ValidatedLoadError};
use heliarca_sim::ids::EntityId;
use heliarca_sim::{
    CommandOutcome, CommandRegistry, Increment, Params, SECONDS_PER_DAY, Scenario, Sim,
};

use crate::CliError;
use crate::script::{Script, days_to_seconds};
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

/// Segundos de simulación de `years` años de `params`; `None` si desborda.
fn total_seconds(years: u32, params: &Params) -> Option<u64> {
    SECONDS_PER_DAY
        .checked_mul(params.days_per_year())?
        .checked_mul(u64::from(years))
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

/// Entradas del script ya enviadas, para informar fallos al aplicarse.
struct ScriptRun {
    /// `kind` de cada entrada, en orden (el id de comando `n` es la entrada `n + 1`).
    kinds: Vec<&'static str>,
    /// Cuántos registros del historial ya se revisaron.
    checked: usize,
}

impl ScriptRun {
    /// Revisa los registros nuevos del historial: un comando que falló al aplicarse detiene la
    /// corrida con la entrada del script.
    fn check_history(&mut self, sim: &Sim, path: &Path) -> Result<(), CliError> {
        let history = sim.command_history();
        for record in &history[self.checked..] {
            if let CommandOutcome::Failed(error) = &record.outcome {
                let index = usize::try_from(record.id.raw()).unwrap_or(usize::MAX);
                let kind = self.kinds.get(index).copied().unwrap_or(record.kind);
                return Err(CliError::Script(format!(
                    "script {}: entrada {} ({kind}): falló al aplicarse en el paso {}: {error} [{}]",
                    path.display(),
                    index.saturating_add(1),
                    record.step,
                    error.loc_key()
                )));
            }
        }
        self.checked = history.len();
        Ok(())
    }
}

/// Avanza hasta `total` segundos enviando cada comando del script al llegar a su `at`.
///
/// Un comando se **envía** cuando el tiempo de simulación es exactamente `at` días y se **aplica
/// al comienzo del paso siguiente** (el primer `advance` después del envío), como todo comando.
/// Los comandos con el mismo `at` se envían juntos, en orden de archivo.
fn run_script(
    sim: &mut Sim,
    chosen: Increment,
    total: u64,
    script: Script,
    path: &Path,
) -> Result<(), CliError> {
    let mut state = ScriptRun {
        kinds: script.entries.iter().map(|e| e.command.kind()).collect(),
        checked: 0,
    };
    let fail = |index: usize, error: String| {
        CliError::Script(format!(
            "script {}: entrada {}: {error}",
            path.display(),
            index + 1
        ))
    };
    for (index, entry) in script.entries.into_iter().enumerate() {
        let at = days_to_seconds(entry.at_days)
            .ok_or_else(|| fail(index, "`at` desborda el tiempo de simulación".into()))?;
        let now = sim.time().as_seconds();
        if at > now {
            advance_total(sim, chosen, at - now)?;
            state.check_history(sim, path)?;
        }
        let kind = entry.command.kind();
        sim.submit_command(entry.command).map_err(|error| {
            fail(
                index,
                format!(
                    "el comando {kind} fue rechazado (día {}): {error} [{}]",
                    entry.at_days,
                    error.loc_key()
                ),
            )
        })?;
    }
    advance_total(sim, chosen, total - sim.time().as_seconds())?;
    state.check_history(sim, path)
}

/// Opciones de [`run`].
pub struct RunOptions<'a> {
    /// Directorio de datos.
    pub data_dir: &'a Path,
    /// Semilla.
    pub seed: u64,
    /// Años a simular.
    pub years: u32,
    /// Incremento elegido.
    pub increment: IncrementArg,
    /// Script de comandos, si se pasó `--script`.
    pub script: Option<&'a Path>,
    /// Archivo donde guardar el estado final, si se pasó `--save`.
    pub save: Option<&'a Path>,
}

/// Opciones de [`load`].
pub struct LoadOptions<'a> {
    /// Directorio de datos.
    pub data_dir: &'a Path,
    /// Guardado a cargar.
    pub file: &'a Path,
    /// Años a continuar.
    pub years: u32,
    /// Incremento elegido.
    pub increment: IncrementArg,
    /// Archivo donde volver a guardar el estado final, si se pasó `--save`.
    pub save: Option<&'a Path>,
}

/// Carga los datos con la política completa de `validate-data`.
fn load_data(data_dir: &Path) -> Result<GameData, CliError> {
    DataLoader::load_validated(data_dir)
        .map(|validated| validated.data)
        .map_err(|error| CliError::Data(load_error_lines(&error)))
}

/// Segundos totales de `years` años, o error si desborda.
fn checked_total(years: u32, params: &Params) -> Result<u64, CliError> {
    total_seconds(years, params)
        .ok_or_else(|| CliError::Sim("el total de años desborda el tiempo de simulación".into()))
}

/// Imprime las líneas finales comunes a `run` y `load` (desde `Pasos`; `commands` agrega
/// `Comandos aplicados`) y, si se pidió, guarda el estado.
///
/// **Orden:** primero se imprime el resumen y recién después se guarda; si el guardado falla, el
/// resumen (válido: la simulación terminó bien) ya salió por stdout y el error va a stderr con
/// código 1. La línea `Guardado: <ruta>` sólo aparece si el guardado tuvo éxito.
fn finish(
    out: &mut dyn Write,
    sim: &Sim,
    game_data: &GameData,
    elapsed: u64,
    show_commands: bool,
    save: Option<&Path>,
) -> Result<(), CliError> {
    let date = sim
        .date()
        .map_err(|error| CliError::Sim(error.to_string()))?;
    // Si stdout está cerrado no hay a quién avisar.
    let _ = writeln!(out, "Pasos: {}", sim.steps());
    if show_commands {
        // Si alguno hubiera fallado, la corrida ya habría terminado con error.
        let _ = writeln!(out, "Comandos aplicados: {}", sim.command_history().len());
    }
    let _ = write!(
        out,
        "Fecha final: {date}\n\
         Eventos: {}\n\
         Hash de estado: {}\n\
         Duración real: {}\n",
        sim.events().len(),
        sim.state_hash(),
        format_duration(elapsed, sim.steps()),
    );
    if let Some(path) = save {
        sim.save_to_file(path, Some(game_data)).map_err(|error| {
            CliError::Save(format!("no se pudo guardar en {}: {error}", path.display()))
        })?;
        let _ = writeln!(out, "Guardado: {}", path.display());
    }
    Ok(())
}

/// Ejecuta `run` y escribe el resumen en `out`.
///
/// # Errors
///
/// [`CliError::Data`] si los datos no cargan o no validan; [`CliError::Script`] si el script no
/// se lee, es inválido o un comando falla; [`CliError::Sim`] si la simulación falla (desborde,
/// falta de progreso, fecha fuera de rango); [`CliError::Save`] si falla `--save`.
pub fn run(options: &RunOptions<'_>, out: &mut dyn Write) -> Result<(), CliError> {
    let RunOptions {
        data_dir,
        seed,
        years,
        increment,
        script,
        save,
    } = *options;
    let data = load_data(data_dir)?;
    let total = checked_total(years, &data.params)?;
    // El script se lee, valida y construye completo ANTES de correr nada.
    let script = script
        .map(|path| {
            let loaded = Script::load(path, &CommandRegistry::builtin())
                .and_then(|script| script.check_range(total / SECONDS_PER_DAY).map(|()| script))
                .map_err(|error| CliError::Script(format!("script {}: {error}", path.display())))?;
            Ok::<_, CliError>((path, loaded))
        })
        .transpose()?;
    let mut sim = Sim::new(Scenario::from_game_data(&data), seed)
        .map_err(|error| CliError::Sim(error.to_string()))?;

    let script_info = script
        .as_ref()
        .map(|(path, script)| (path.display().to_string(), script.entries.len()));
    let watch = Stopwatch::start();
    match script {
        None => advance_total(&mut sim, increment.increment(), total)?,
        Some((path, script)) => run_script(&mut sim, increment.increment(), total, script, path)?,
    }
    let elapsed = watch.elapsed_millis();

    let _ = write!(out, "Años simulados: {years}\nSemilla: {seed}\n");
    let _ = writeln!(out, "Incremento: {}", increment.name());
    if let Some((path, count)) = &script_info {
        let noun = if *count == 1 { "comando" } else { "comandos" };
        let _ = writeln!(out, "Script: {path} ({count} {noun})");
    }
    finish(out, &sim, &data, elapsed, script_info.is_some(), save)
}

/// Ejecuta `load`: carga un guardado, continúa `years` años y escribe el mismo resumen que `run`.
///
/// La semilla sale del guardado (se informa en `Semilla`). `Años simulados` cuenta los años de
/// **esta** invocación y `Pasos`, los del guardado más los nuevos. Sin script: los `at` de un
/// script serían ambiguos (¿desde el inicio de la partida o desde lo cargado?), así que no se
/// soporta. Los datos se verifican contra la huella del guardado.
///
/// # Errors
///
/// [`CliError::Data`] si los datos no cargan; [`CliError::Save`] si el guardado no se puede leer
/// o no corresponde a los datos (o falla `--save`); [`CliError::Sim`] si la simulación falla.
pub fn load(options: &LoadOptions<'_>, out: &mut dyn Write) -> Result<(), CliError> {
    let LoadOptions {
        data_dir,
        file,
        years,
        increment,
        save,
    } = *options;
    let data = load_data(data_dir)?;
    let mut sim = Sim::load_from_file(file, Some(&data)).map_err(|error| {
        CliError::Save(format!("no se pudo cargar {}: {error}", file.display()))
    })?;
    let total = checked_total(years, sim.params())?;
    let origin = sim
        .date()
        .map_err(|error| CliError::Sim(error.to_string()))?;

    let watch = Stopwatch::start();
    advance_total(&mut sim, increment.increment(), total)?;
    let elapsed = watch.elapsed_millis();

    let _ = writeln!(out, "Origen: {} ({origin})", file.display());
    let _ = write!(out, "Años simulados: {years}\nSemilla: {}\n", sim.seed());
    let _ = writeln!(out, "Incremento: {}", increment.name());
    finish(out, &sim, &data, elapsed, false, save)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_ano_son_365_dias() {
        let gdd = Params::default();
        assert_eq!(total_seconds(1, &gdd), Some(365 * 86_400));
        assert_eq!(total_seconds(0, &gdd), Some(0));
        assert!(total_seconds(MAX_YEARS, &gdd).is_some());
    }

    #[test]
    fn el_ano_dura_lo_que_dicen_los_params() {
        let params = Params::new(7, 30, 360).unwrap();
        assert_eq!(total_seconds(2, &params), Some(720 * 86_400));
        let largo = Params::new(7, 30, Params::MAX_DAYS_PER_YEAR).unwrap();
        assert!(total_seconds(MAX_YEARS, &largo).is_some());
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
        let mut sim = Sim::new(Scenario::default(), 1).unwrap();
        let total = 400 * 86_400;
        advance_total(&mut sim, Increment::D30, total).unwrap();
        assert_eq!(sim.time().as_seconds(), total);
        // 13 x D30 (390 d) + 2 x D5 (10 d) = 15 pasos.
        assert_eq!(sim.steps(), 15);
    }
}
