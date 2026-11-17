//! Scripts de comandos (#42): archivo JSON versionado con los comandos a enviar y cuándo.
//!
//! # Formato (versión 1)
//!
//! ```json
//! {
//!   "version": 1,
//!   "commands": [
//!     { "at": 0,   "command": { "kind": "debug.mark", "args": { "label": "inicio" } } },
//!     { "at": 365, "command": { "kind": "debug.mark", "args": { "label": "un año" } } }
//!   ]
//! }
//! ```
//!
//! - La raíz es un **objeto** con `version` (entero; hoy sólo `1`) y `commands` (lista, puede ser
//!   vacía). Un objeto raíz, y no directamente la lista, permite cambiar el formato más adelante
//!   sin ambigüedad: una versión desconocida es un error claro y no una mala interpretación.
//! - `at`: **días simulados desde el inicio** (entero ≥ 0; `0` es el comienzo de la corrida). Es
//!   la unidad natural para quien escribe el script y coincide con la granularidad del calendario;
//!   un día son [`SECONDS_PER_DAY`] segundos de simulación. El comando se **envía** cuando el
//!   tiempo de simulación llega a `at` días y se **aplica al comienzo del paso siguiente**.
//! - `command.kind` es el nombre del comando (ver `CommandRegistry`); `command.args` es un objeto
//!   (opcional; por omisión `{}`) con los argumentos. Valores admitidos: texto, booleano y
//!   enteros (los no negativos son `UInt`, los negativos `Int`); sin decimales, `null`, listas
//!   ni objetos anidados, porque el estado de la simulación no usa flotantes.
//! - Las entradas van **ordenadas por `at`**; con el mismo `at` se envían en el orden del archivo.
//! - Campos desconocidos en cualquier nivel son error (un error de tipeo no se ignora).
//!
//! # Errores y posiciones
//!
//! Las entradas se numeran **desde 1** (`entrada 3` es la tercera de `commands`). Todo el script
//! se lee, valida y construye **antes** de correr la simulación, de modo que un error de forma
//! (kind desconocido, argumentos, orden, rango) no deja una corrida a medias. Cada error indica la
//! entrada y el campo (`command.args.label`); los de archivo o JSON, el archivo y la línea y la
//! columna.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use heliarca_sim::{Command, CommandRegistry, Payload, SECONDS_PER_DAY, Value};
use serde_json::{Map, Value as Json};

/// Única versión del formato que entiende esta CLI.
pub const SUPPORTED_VERSION: u64 = 1;

/// Tamaño máximo del archivo de script (16 MiB): evita leer archivos enormes por error.
pub const MAX_SCRIPT_BYTES: u64 = 16 * 1024 * 1024;

/// Error de un script, con la posición de la entrada y el campo cuando corresponde.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptError {
    /// Entrada (desde 1) en la que ocurrió; `None` si es del archivo o de la raíz.
    pub entry: Option<usize>,
    /// Campo dentro de la entrada o de la raíz (`at`, `command.args.label`, `version`...).
    pub field: Option<String>,
    /// Qué pasó.
    pub message: String,
}

impl ScriptError {
    fn root(field: Option<&str>, message: impl Into<String>) -> Self {
        Self {
            entry: None,
            field: field.map(str::to_owned),
            message: message.into(),
        }
    }

    fn at_entry(entry: usize, field: Option<&str>, message: impl Into<String>) -> Self {
        Self {
            entry: Some(entry),
            field: field.map(str::to_owned),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(entry) = self.entry {
            write!(f, "entrada {entry}: ")?;
        }
        if let Some(field) = &self.field {
            write!(f, "campo {field}: ")?;
        }
        f.write_str(&self.message)
    }
}

/// Una entrada ya validada y con su comando construido.
#[derive(Debug)]
pub struct Entry {
    /// Día de simulación (desde el inicio) en que se envía.
    pub at_days: u64,
    /// El comando construido por el registro.
    pub command: Box<dyn Command>,
}

/// Un script completo: entradas ordenadas por `at`.
#[derive(Debug)]
pub struct Script {
    /// Entradas en orden de envío.
    pub entries: Vec<Entry>,
}

impl Script {
    /// Lee y parsea el archivo `path` (ver el módulo).
    ///
    /// # Errors
    /// [`ScriptError`] si el archivo no se puede leer, no es UTF-8 ni JSON, o no cumple el formato.
    pub fn load(path: &Path, registry: &CommandRegistry) -> Result<Self, ScriptError> {
        let file = File::open(path)
            .map_err(|error| ScriptError::root(None, format!("no se pudo leer: {error}")))?;
        let mut bytes = Vec::new();
        file.take(MAX_SCRIPT_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| ScriptError::root(None, format!("no se pudo leer: {error}")))?;
        if bytes.len() as u64 > MAX_SCRIPT_BYTES {
            return Err(ScriptError::root(
                None,
                format!("el archivo supera el máximo de {MAX_SCRIPT_BYTES} bytes"),
            ));
        }
        let text = String::from_utf8(bytes).map_err(|error| {
            ScriptError::root(
                None,
                format!(
                    "no es UTF-8 válido (byte {})",
                    error.utf8_error().valid_up_to()
                ),
            )
        })?;
        Self::parse(&text, registry)
    }

    /// Parsea el texto de un script (ver el módulo).
    ///
    /// # Errors
    /// [`ScriptError`] con la entrada y el campo del primer problema.
    pub fn parse(text: &str, registry: &CommandRegistry) -> Result<Self, ScriptError> {
        // Los editores de Windows suelen anteponer un BOM; no es parte del JSON.
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let root: Json = serde_json::from_str(text).map_err(|error| {
            // El `Display` de serde ya termina en «at line L column C»: se reemplaza por nuestro prefijo.
            let detail = error.to_string();
            let detail = detail
                .rsplit_once(" at line ")
                .map_or(detail.as_str(), |(head, _)| head);
            ScriptError::root(
                None,
                format!(
                    "JSON malformado en la línea {}, columna {}: {detail}",
                    error.line(),
                    error.column()
                ),
            )
        })?;
        let Json::Object(root) = root else {
            return Err(ScriptError::root(
                None,
                "la raíz debe ser un objeto {\"version\": 1, \"commands\": [...]}",
            ));
        };
        check_unknown(&root, &["version", "commands"], |key| {
            ScriptError::root(Some(key), "campo desconocido")
        })?;
        check_version(root.get("version"))?;
        let Some(commands) = root.get("commands") else {
            return Err(ScriptError::root(Some("commands"), "falta el campo"));
        };
        let Json::Array(commands) = commands else {
            return Err(ScriptError::root(Some("commands"), "debe ser una lista"));
        };

        let mut entries: Vec<Entry> = Vec::with_capacity(commands.len());
        for (index, raw) in commands.iter().enumerate() {
            let entry = parse_entry(index + 1, raw, registry)?;
            if let Some(previous) = entries.last()
                && entry.at_days < previous.at_days
            {
                return Err(ScriptError::at_entry(
                    index + 1,
                    Some("at"),
                    format!(
                        "fuera de orden: {} es anterior a {} de la entrada {}",
                        entry.at_days, previous.at_days, index
                    ),
                ));
            }
            entries.push(entry);
        }
        Ok(Self { entries })
    }

    /// Comprueba que ningún comando quede fuera de la corrida: `at` debe ser **menor** que
    /// `total_days`, porque un comando enviado en el último instante nunca llegaría a aplicarse.
    ///
    /// # Errors
    /// [`ScriptError`] en la primera entrada fuera de rango.
    pub fn check_range(&self, total_days: u64) -> Result<(), ScriptError> {
        for (index, entry) in self.entries.iter().enumerate() {
            if entry.at_days >= total_days {
                let message = if total_days == 0 {
                    format!(
                        "{} está fuera de la corrida: no simula ningún día (--years 0)",
                        entry.at_days
                    )
                } else {
                    format!(
                        "{} está fuera de la corrida: debe ser menor que {total_days} (los días simulados)",
                        entry.at_days
                    )
                };
                return Err(ScriptError::at_entry(index + 1, Some("at"), message));
            }
        }
        Ok(())
    }
}

/// Segundos de simulación de `days` días; `None` si desborda.
pub fn days_to_seconds(days: u64) -> Option<u64> {
    days.checked_mul(SECONDS_PER_DAY)
}

fn check_unknown(
    map: &Map<String, Json>,
    allowed: &[&str],
    error: impl Fn(&str) -> ScriptError,
) -> Result<(), ScriptError> {
    match map.keys().find(|key| !allowed.contains(&key.as_str())) {
        Some(key) => Err(error(key)),
        None => Ok(()),
    }
}

fn check_version(version: Option<&Json>) -> Result<(), ScriptError> {
    match version {
        None => Err(ScriptError::root(
            Some("version"),
            format!("falta el campo (la versión soportada es {SUPPORTED_VERSION})"),
        )),
        Some(Json::Number(n)) if n.as_u64() == Some(SUPPORTED_VERSION) => Ok(()),
        Some(other) => Err(ScriptError::root(
            Some("version"),
            format!(
                "versión no soportada: {other} (esta CLI sólo entiende la versión {SUPPORTED_VERSION})"
            ),
        )),
    }
}

fn parse_entry(
    number: usize,
    raw: &Json,
    registry: &CommandRegistry,
) -> Result<Entry, ScriptError> {
    let fail = |field: &str, message: String| ScriptError::at_entry(number, Some(field), message);
    let Json::Object(entry) = raw else {
        return Err(ScriptError::at_entry(
            number,
            None,
            "debe ser un objeto {\"at\": ..., \"command\": {...}}",
        ));
    };
    check_unknown(entry, &["at", "command"], |key| {
        fail(key, "campo desconocido".into())
    })?;

    let at_days = match entry.get("at") {
        None => return Err(fail("at", "falta el campo".into())),
        Some(Json::Number(n)) => n.as_u64().ok_or_else(|| {
            fail(
                "at",
                if n.as_i64().is_some() {
                    format!("{n} es negativo: debe ser un entero de días desde el inicio (≥ 0)")
                } else {
                    format!("{n} no es un entero de días válido")
                },
            )
        })?,
        Some(_) => {
            return Err(fail(
                "at",
                "debe ser un entero de días desde el inicio".into(),
            ));
        }
    };

    let Some(command) = entry.get("command") else {
        return Err(fail("command", "falta el campo".into()));
    };
    let Json::Object(command) = command else {
        return Err(fail("command", "debe ser un objeto".into()));
    };
    check_unknown(command, &["kind", "args"], |key| {
        fail(&format!("command.{key}"), "campo desconocido".into())
    })?;
    let kind = match command.get("kind") {
        None => return Err(fail("command.kind", "falta el campo".into())),
        Some(Json::String(kind)) => kind,
        Some(_) => return Err(fail("command.kind", "debe ser texto".into())),
    };
    let args = match command.get("args") {
        None => Payload::new(),
        Some(Json::Object(args)) => to_payload(args)
            .map_err(|(key, message)| fail(&format!("command.args.{key}"), message))?,
        Some(_) => return Err(fail("command.args", "debe ser un objeto".into())),
    };

    let built = registry.build(kind, &args).map_err(|error| {
        use heliarca_sim::CommandBuildError as E;
        match &error {
            E::UnknownKind { .. } => {
                let known: Vec<_> = registry.kinds().collect();
                fail(
                    "command.kind",
                    format!("{error} (conocidos: {})", known.join(", ")),
                )
            }
            E::MissingArgument { argument } => {
                fail("command.args", format!("{error} ({argument})"))
            }
            E::UnexpectedArgument { argument } => {
                fail(&format!("command.args.{argument}"), error.to_string())
            }
            E::InvalidArgument { argument, .. } => {
                fail(&format!("command.args.{argument}"), error.to_string())
            }
            _ => fail("command", error.to_string()),
        }
    })?;
    Ok(Entry {
        at_days,
        command: built,
    })
}

/// JSON → [`Payload`]: texto, booleano y enteros. `Err((clave, motivo))` si algún valor no entra.
fn to_payload(args: &Map<String, Json>) -> Result<Payload, (String, String)> {
    let mut payload = Payload::new();
    for (key, value) in args {
        let value = match value {
            Json::String(text) => Value::Text(text.clone()),
            Json::Bool(flag) => Value::Bool(*flag),
            Json::Number(n) => match (n.as_u64(), n.as_i64()) {
                (Some(unsigned), _) => Value::UInt(unsigned),
                (None, Some(signed)) => Value::Int(signed),
                (None, None) => {
                    return Err((
                        key.clone(),
                        format!("{n} no es un entero (no hay decimales)"),
                    ));
                }
            },
            Json::Null | Json::Array(_) | Json::Object(_) => {
                return Err((
                    key.clone(),
                    "sólo se admiten texto, booleanos y enteros".into(),
                ));
            }
        };
        payload = payload.with(key.clone(), value);
    }
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<Script, ScriptError> {
        Script::parse(text, &CommandRegistry::builtin())
    }

    fn err(text: &str) -> String {
        parse(text).unwrap_err().to_string()
    }

    fn entry(at: &str, label: &str) -> String {
        format!(
            r#"{{"at": {at}, "command": {{"kind": "debug.mark", "args": {{"label": "{label}"}}}}}}"#
        )
    }

    #[test]
    fn parsea_un_script_valido_y_conserva_el_orden() {
        let text = format!(
            r#"{{"version": 1, "commands": [{}, {}, {}]}}"#,
            entry("0", "a"),
            entry("5", "b"),
            entry("5", "c")
        );
        let script = parse(&text).unwrap();
        let ats: Vec<u64> = script.entries.iter().map(|e| e.at_days).collect();
        assert_eq!(ats, [0, 5, 5]);
        assert_eq!(script.entries[2].command.kind(), "debug.mark");
        assert!(
            parse(r#"{"version": 1, "commands": []}"#)
                .unwrap()
                .entries
                .is_empty()
        );
        // `args` es opcional pero este comando lo necesita: el error es del comando.
        assert!(
            err(r#"{"version":1,"commands":[{"at":0,"command":{"kind":"debug.mark"}}]}"#)
                .contains("entrada 1: campo command.args: falta el argumento «label»")
        );
        // BOM inicial tolerado.
        assert!(parse("\u{feff}{\"version\":1,\"commands\":[]}").is_ok());
    }

    #[test]
    fn los_errores_indican_entrada_y_campo() {
        let cases: [(&str, &[&str]); 12] = [
            ("[]", &["la raíz debe ser un objeto"]),
            (r#"{"commands": []}"#, &["campo version", "falta"]),
            (
                r#"{"version": 2, "commands": []}"#,
                &["campo version", "no soportada: 2"],
            ),
            (
                r#"{"version": "1", "commands": []}"#,
                &["campo version", "no soportada"],
            ),
            (r#"{"version": 1}"#, &["campo commands", "falta"]),
            (
                r#"{"version": 1, "commands": {}}"#,
                &["campo commands", "lista"],
            ),
            (
                r#"{"version": 1, "commands": [], "x": 1}"#,
                &["campo x", "desconocido"],
            ),
            (
                r#"{"version": 1, "commands": [3]}"#,
                &["entrada 1:", "objeto"],
            ),
            (
                r#"{"version": 1, "commands": [{"at": 0, "command": {"kind": "x.y"}, "z": 1}]}"#,
                &["entrada 1: campo z", "desconocido"],
            ),
            (
                r#"{"version": 1, "commands": [{"at": 0, "command": {"kind": "x.y"}}]}"#,
                &[
                    "entrada 1: campo command.kind",
                    "comando desconocido: x.y",
                    "debug.mark",
                ],
            ),
            (
                r#"{"version": 1, "commands": [{"at": -1, "command": {"kind": "x.y"}}]}"#,
                &["entrada 1: campo at", "negativo"],
            ),
            (
                r#"{"version": 1, "commands": [{"at": 1.5, "command": {"kind": "x.y"}}]}"#,
                &["entrada 1: campo at", "no es un entero"],
            ),
        ];
        for (text, parts) in cases {
            let message = err(text);
            for part in parts {
                assert!(
                    message.contains(part),
                    "{text}\n  -> {message}\n  falta {part:?}"
                );
            }
        }
    }

    #[test]
    fn la_entrada_con_el_problema_es_la_que_se_informa() {
        let text = format!(
            r#"{{"version": 1, "commands": [{}, {}, {{"at": 2, "command": {{"kind": "debug.mark", "args": {{"label": 7}}}}}}]}}"#,
            entry("0", "a"),
            entry("1", "b")
        );
        let message = err(&text);
        assert!(
            message.starts_with("entrada 3: campo command.args.label"),
            "{message}"
        );
        let text = format!(
            r#"{{"version": 1, "commands": [{}, {}]}}"#,
            entry("5", "a"),
            entry("4", "b")
        );
        let message = err(&text);
        assert!(
            message.starts_with("entrada 2: campo at: fuera de orden"),
            "{message}"
        );
        let wrong_args = [
            (r#"{"label": "a", "otro": 1}"#, "command.args.otro"),
            (r#"{"label": null}"#, "command.args.label"),
            (r#"{"label": 1.5}"#, "command.args.label"),
            (r#"{"label": ""}"#, "command.args.label"),
            ("{}", "command.args"),
            ("[]", "command.args"),
        ];
        for (args, field) in wrong_args {
            let text = format!(
                r#"{{"version":1,"commands":[{{"at":0,"command":{{"kind":"debug.mark","args":{args}}}}}]}}"#
            );
            let message = err(&text);
            assert!(
                message.starts_with(&format!("entrada 1: campo {field}")),
                "{args}: {message}"
            );
        }
    }

    #[test]
    fn json_malformado_da_linea_y_columna() {
        let message = err("{\n  \"version\": 1,\n  \"commands\": [\n");
        assert!(
            message.contains("JSON malformado en la línea 4"),
            "{message}"
        );
        assert!(err("").contains("JSON malformado"));
    }

    #[test]
    fn el_rango_se_comprueba_contra_los_dias_simulados() {
        let text = format!(
            r#"{{"version": 1, "commands": [{}, {}]}}"#,
            entry("0", "a"),
            entry("10", "b")
        );
        let script = parse(&text).unwrap();
        assert!(script.check_range(11).is_ok());
        let message = script.check_range(10).unwrap_err().to_string();
        assert!(
            message.starts_with("entrada 2: campo at: 10 está fuera"),
            "{message}"
        );
        assert!(
            script
                .check_range(0)
                .unwrap_err()
                .to_string()
                .starts_with("entrada 1")
        );
        // Un `at` enorme no desborda: sólo queda fuera de rango.
        let huge = format!(
            r#"{{"version":1,"commands":[{}]}}"#,
            entry("18446744073709551615", "a")
        );
        assert!(parse(&huge).unwrap().check_range(365).is_err());
        assert_eq!(days_to_seconds(u64::MAX), None);
        assert_eq!(days_to_seconds(2), Some(172_800));
    }
}
