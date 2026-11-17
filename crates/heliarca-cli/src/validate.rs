//! Subcomando `validate-data` (#40): valida los archivos de `--data-dir` y junta **todos** los errores.
//!
//! # Etapas y política de errores
//!
//! Se informa cada problema una sola vez y sin abortar en el primero. Por cada archivo de
//! [`DATA_FILES`] (en ese orden) se aplica, como mucho, **una** de estas fuentes:
//!
//! 1. **Lectura y sintaxis** (cargador): archivo faltante, ilegible, UTF-8 inválido o JSON
//!    malformado. Si el archivo falla acá, no se puede validar nada más de él.
//! 2. **Esquema JSON**: el archivo legible se valida contra el esquema que `heliarca_sim::schema`
//!    genera en memoria (no contra `data/schema/*.json`, que podrían estar desactualizados; el CI
//!    garantiza que coinciden). Los errores llevan la ruta JSON dentro del archivo
//!    (`bodies.json: /3/orbit/period_days: ...`). Si el esquema encuentra problemas, los errores de
//!    modelo del cargador para ese archivo (tipo equivocado, campo faltante o desconocido) se
//!    omiten, porque dirían lo mismo con otro formato. Se listan hasta 100 errores por archivo.
//! 3. **Modelo del cargador**: sólo para archivos que pasaron el esquema; atrapa lo que el esquema
//!    no expresa (p. ej. `3.0` en un campo entero). Formato `bodies.json: body.luna: orbit.x: ...`.
//!
//! Después, **sólo si no hubo ningún error en las etapas anteriores** (los datos están completos),
//! corre el **registro** de ids y referencias (formato, duplicados, referencias inexistentes, ciclos
//! de `parent`). Con datos incompletos daría falsos «no existe», así que se espera a que los
//! archivos estén sanos.

use std::fs;
use std::path::Path;

use heliarca_sim::data::{DATA_FILES, DataError, DataLoader, Registry};
use heliarca_sim::schema::schema_for_data_file;
use serde_json::Value;

/// Largo máximo (en caracteres) de un mensaje de esquema; los valores enormes se recortan.
const MAX_MESSAGE_CHARS: usize = 300;

/// Máximo de errores de esquema que se listan por archivo; el resto se resume en una línea
/// (un archivo del tipo equivocado podría generar cientos de miles).
const MAX_SCHEMA_ERRORS_PER_FILE: usize = 100;

/// Resultado de validar un directorio de datos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// Cantidad de archivos de datos que se validaron.
    pub files_checked: usize,
    /// Errores, uno por línea, en orden determinista (el de [`DATA_FILES`]; luego el del registro).
    pub errors: Vec<String>,
}

impl Report {
    /// Cantidad de archivos distintos con errores (los errores del directorio no cuentan).
    #[must_use]
    pub fn files_with_errors(&self) -> usize {
        let mut files: Vec<&str> = self.errors.iter().filter_map(|e| file_of(e)).collect();
        files.sort_unstable();
        files.dedup();
        files.len()
    }

    /// Línea de resumen para stdout.
    #[must_use]
    pub fn summary(&self) -> String {
        if self.errors.is_empty() {
            return format!("OK: {} archivos validados", self.files_checked);
        }
        let n = self.errors.len();
        let m = self.files_with_errors();
        let errores = if n == 1 { "error" } else { "errores" };
        match m {
            0 => format!("{n} {errores}"),
            1 => format!("{n} {errores} en 1 archivo"),
            _ => format!("{n} {errores} en {m} archivos"),
        }
    }
}

/// Archivo (de [`DATA_FILES`]) con el que empieza una línea de error, si lo hay.
fn file_of(line: &str) -> Option<&'static str> {
    DATA_FILES
        .into_iter()
        .find(|file| line.strip_prefix(file).is_some_and(|r| r.starts_with(": ")))
}

/// Archivo al que pertenece un error del cargador (`None` para los del directorio).
fn data_error_file(error: &DataError) -> Option<&'static str> {
    match error {
        DataError::FileMissing { file }
        | DataError::Unreadable { file, .. }
        | DataError::InvalidUtf8 { file, .. }
        | DataError::Syntax { file, .. }
        | DataError::WrongRoot { file, .. }
        | DataError::Schema { file, .. } => Some(file),
        DataError::DirectoryMissing { .. } | DataError::NotADirectory { .. } => None,
    }
}

/// `true` si es un error de modelo (cubierto también por el esquema), no de lectura o sintaxis.
fn is_model_error(error: &DataError) -> bool {
    matches!(
        error,
        DataError::Schema { .. } | DataError::WrongRoot { .. }
    )
}

/// Recorta un mensaje largo a [`MAX_MESSAGE_CHARS`] caracteres.
fn clip(message: &str) -> String {
    if message.chars().count() <= MAX_MESSAGE_CHARS {
        return message.to_owned();
    }
    let mut out: String = message.chars().take(MAX_MESSAGE_CHARS).collect();
    out.push('…');
    out
}

/// Errores de esquema de `file` (ordenados), o un error técnico si no se pudo validar.
fn schema_errors(dir: &Path, file: &'static str) -> Vec<String> {
    let Some(schema) = schema_for_data_file(file) else {
        return vec![format!("{file}: no hay esquema para este archivo")];
    };
    let validator = match jsonschema::validator_for(schema.as_value()) {
        Ok(validator) => validator,
        Err(error) => return vec![format!("{file}: el esquema generado no es válido: {error}")],
    };
    // El cargador ya pudo leer y parsear el archivo; si falla acá es una carrera con el disco.
    let instance = match fs::read(dir.join(file))
        .map_err(|e| e.to_string())
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).map_err(|e| e.to_string()))
    {
        Ok(instance) => instance,
        Err(error) => return vec![format!("{file}: no se pudo releer: {error}")],
    };
    let mut errors: Vec<String> = validator
        .iter_errors(&instance)
        .map(|e| {
            let path = e.instance_path().to_string();
            let path = if path.is_empty() { "/" } else { &path };
            format!("{file}: {path}: {}", clip(&e.to_string()))
        })
        .collect();
    errors.sort();
    if errors.len() > MAX_SCHEMA_ERRORS_PER_FILE {
        let more = errors.len() - MAX_SCHEMA_ERRORS_PER_FILE;
        errors.truncate(MAX_SCHEMA_ERRORS_PER_FILE);
        errors.push(format!("{file}: ... y {more} errores de esquema más"));
    }
    errors
}

/// Valida `dir` (ver el módulo) y devuelve todos los errores encontrados.
#[must_use]
pub fn validate(dir: &Path) -> Report {
    let files_checked = DATA_FILES.len();
    let (data, loader_errors) = match DataLoader::load_dir(dir) {
        Ok(data) => (Some(data), Vec::new()),
        Err(errors) => (None, errors.0),
    };
    // Error del directorio: no hay archivos que mirar.
    if loader_errors.iter().any(|e| data_error_file(e).is_none()) {
        return Report {
            files_checked,
            errors: loader_errors.iter().map(ToString::to_string).collect(),
        };
    }

    let mut errors = Vec::new();
    for file in DATA_FILES {
        let of_file: Vec<&DataError> = loader_errors
            .iter()
            .filter(|e| data_error_file(e) == Some(file))
            .collect();
        // 1. Lectura/sintaxis: el archivo no se puede seguir validando.
        if of_file.iter().any(|e| !is_model_error(e)) {
            errors.extend(
                of_file
                    .iter()
                    .filter(|e| !is_model_error(e))
                    .map(ToString::to_string),
            );
            continue;
        }
        // 2. Esquema; si encuentra algo, el cargador diría lo mismo y se omite.
        let by_schema = schema_errors(dir, file);
        if by_schema.is_empty() {
            // 3. Lo que el esquema no expresa.
            errors.extend(of_file.iter().map(ToString::to_string));
        } else {
            errors.extend(by_schema);
        }
    }

    // Registro: sólo con todos los datos sanos.
    if errors.is_empty()
        && let Some(data) = data
        && let Err(registry_errors) = Registry::build(&data)
    {
        errors.extend(registry_errors.0.iter().map(ToString::to_string));
    }
    Report {
        files_checked,
        errors,
    }
}
