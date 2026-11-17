//! Ejecutor sin interfaz de Heliarca.
//!
//! Sólo usa la API pública de `heliarca-sim` (`Sim`, `Scenario`...). Este archivo es el esqueleto
//! de la CLI (#39): argumentos, subcomandos y códigos de salida. `validate-data` (#40) vive en
//! [`validate`]; la lógica de `run` (#41) llega en su tarea y hasta entonces es un stub con un
//! error claro.
//!
//! Códigos de salida: `0` éxito (incluye `--help` y `--version`), `1` error de datos, de
//! simulación o funcionalidad todavía no implementada, `2` error de uso (argumentos inválidos).

mod validate;

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use heliarca_sim::{Scenario, Sim};

/// Semilla por defecto cuando no se pasa `--seed`.
const DEFAULT_SEED: u64 = 0;

/// Argumentos de la línea de comandos.
#[derive(Debug, Parser)]
#[command(
    name = "heliarca-cli",
    version,
    about = "Ejecutor sin interfaz de Heliarca"
)]
struct Cli {
    /// Semilla del generador aleatorio de la simulación.
    #[arg(long, global = true, value_name = "U64", default_value_t = DEFAULT_SEED)]
    seed: u64,

    /// Directorio con los datos del juego.
    #[arg(long, global = true, value_name = "RUTA", default_value = "data")]
    data_dir: PathBuf,

    /// Sin subcomando se imprime el banner con la versión del núcleo y la fecha inicial.
    #[command(subcommand)]
    command: Option<Command>,
}

/// Subcomandos disponibles.
#[derive(Debug, Subcommand, PartialEq, Eq)]
enum Command {
    /// Avanza la simulación N años (pendiente: #41).
    Run {
        /// Cantidad de años a simular.
        #[arg(long, value_name = "N")]
        years: Option<u32>,
    },
    /// Valida los archivos de datos de `--data-dir` e imprime todos los errores (código 1 si hay).
    ValidateData,
}

/// Errores de la CLI, cada uno con su código de salida.
#[derive(Debug, thiserror::Error)]
enum CliError {
    /// Argumentos inválidos, `--help` o `--version` (clap decide el código: 2 o 0).
    #[error("{0}")]
    Usage(clap::Error),
    /// Subcomando reconocido pero sin lógica todavía.
    #[error("`{command}` no está implementado todavía (ver #{issue})")]
    NotImplemented { command: &'static str, issue: u32 },
    /// `validate-data` encontró errores en los datos (se imprimen uno por línea).
    #[error("{}", .0.summary())]
    InvalidData(validate::Report),
}

impl CliError {
    /// Código de salida del proceso.
    fn exit_code(&self) -> u8 {
        match self {
            // clap: 0 para ayuda/versión, 2 para errores de uso.
            Self::Usage(error) => u8::try_from(error.exit_code()).unwrap_or(2),
            Self::NotImplemented { .. } | Self::InvalidData(_) => 1,
        }
    }
}

/// Texto que imprime el ejecutable sin subcomando: versión del núcleo y fecha inicial.
fn banner(seed: u64) -> String {
    let sim = Sim::new(Scenario::default(), seed);
    let date = sim
        .date()
        .map_or_else(|error| error.to_string(), |date| date.to_string());
    format!("heliarca-cli (sim {}) - {date}", heliarca_sim::version())
}

/// Ejecuta la CLI sobre `args` (incluido el nombre del programa) y escribe en `out`.
fn execute<I, T>(args: I, out: &mut dyn Write) -> Result<(), CliError>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = Cli::try_parse_from(args).map_err(CliError::Usage)?;
    match cli.command {
        None => {
            // Si stdout está cerrado no hay a quién avisar.
            let _ = writeln!(out, "{}", banner(cli.seed));
            Ok(())
        }
        Some(Command::Run { .. }) => Err(CliError::NotImplemented {
            command: "run",
            issue: 41,
        }),
        Some(Command::ValidateData) => {
            let report = validate::validate(&cli.data_dir);
            if report.errors.is_empty() {
                let _ = writeln!(out, "{}", report.summary());
                Ok(())
            } else {
                Err(CliError::InvalidData(report))
            }
        }
    }
}

/// Punto de entrada testeable: devuelve el código de salida y escribe en `out`/`err`.
fn run<'a, I, T>(args: I, out: &'a mut dyn Write, err: &'a mut dyn Write) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    match execute(args, out) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let code = error.exit_code();
            // La ayuda y la versión van a stdout; el resto a stderr.
            match &error {
                CliError::Usage(usage) => {
                    let target = if code == 0 { out } else { err };
                    let _ = write!(target, "{}", usage.render());
                }
                CliError::NotImplemented { .. } => {
                    let _ = writeln!(err, "error: {error}");
                }
                // Los errores van a stderr, uno por línea; el resumen, a stdout.
                CliError::InvalidData(report) => {
                    for line in &report.errors {
                        let _ = writeln!(err, "{line}");
                    }
                    let _ = writeln!(out, "{}", report.summary());
                }
            }
            ExitCode::from(code)
        }
    }
}

fn main() -> ExitCode {
    run(
        std::env::args_os(),
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Corre la CLI y devuelve (código, stdout, stderr).
    fn exec(args: &[&str]) -> (ExitCode, String, String) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(args.iter().copied(), &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    #[test]
    fn el_banner_incluye_la_version_de_sim() {
        assert!(banner(DEFAULT_SEED).contains(heliarca_sim::version()));
    }

    #[test]
    fn el_banner_incluye_la_fecha_inicial() {
        assert!(banner(DEFAULT_SEED).ends_with("Año 1, día 1, 00:00:00"));
    }

    #[test]
    fn sin_subcomando_imprime_el_banner_y_sale_con_0() {
        let (code, out, _) = exec(&["heliarca-cli"]);
        assert_eq!(code, ExitCode::SUCCESS);
        assert!(out.contains("heliarca-cli (sim "));
    }

    #[test]
    fn parsea_seed_y_data_dir_globales() {
        let cli = Cli::try_parse_from(["h", "--seed", "42", "--data-dir", "x", "run"]).unwrap();
        assert_eq!(cli.seed, 42);
        assert_eq!(cli.data_dir, PathBuf::from("x"));
        // Los globales también valen después del subcomando.
        let cli = Cli::try_parse_from(["h", "validate-data", "--seed", "7"]).unwrap();
        assert_eq!(cli.seed, 7);
        assert_eq!(cli.command, Some(Command::ValidateData));
    }

    #[test]
    fn los_valores_por_defecto_son_semilla_0_y_data() {
        let cli = Cli::try_parse_from(["h", "run"]).unwrap();
        assert_eq!(cli.seed, 0);
        assert_eq!(cli.data_dir, PathBuf::from("data"));
    }

    #[test]
    fn run_es_un_stub_que_sale_con_1_y_apunta_al_issue() {
        let (code, _, err) = exec(&["h", "run"]);
        assert_eq!(code, ExitCode::from(1));
        assert!(err.contains("#41"));
    }

    #[test]
    fn validate_data_con_directorio_inexistente_sale_con_1() {
        let (code, out, err) = exec(&["h", "validate-data", "--data-dir", "no/existe/nunca"]);
        assert_eq!(code, ExitCode::from(1));
        assert!(err.contains("el directorio de datos no existe"), "{err}");
        assert!(out.contains("1 error"), "{out}");
    }

    #[test]
    fn los_errores_de_uso_salen_con_2() {
        for args in [
            &["h", "nope"][..],
            &["h", "--seed", "abc", "run"],
            &["h", "--nope"],
        ] {
            let (code, out, err) = exec(args);
            assert_eq!(code, ExitCode::from(2), "{args:?}");
            assert!(out.is_empty() && !err.is_empty());
        }
    }

    #[test]
    fn help_y_version_salen_con_0_por_stdout() {
        let (code, out, err) = exec(&["h", "--help"]);
        assert_eq!(code, ExitCode::SUCCESS);
        assert!(out.contains("run") && out.contains("validate-data") && err.is_empty());
        let (code, out, _) = exec(&["h", "--version"]);
        assert_eq!(code, ExitCode::SUCCESS);
        assert!(out.contains("heliarca-cli"));
    }
}
