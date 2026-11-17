//! Cargador de los datos del juego (`data/`, doc 22 §11).
//!
//! # Dónde vive y por qué
//!
//! Es un módulo de `heliarca-sim` (y no de un crate aparte ni de `heliarca-godot`) porque la
//! simulación es la que consume los datos (doc 21 §2: "dirigida por datos") y debe poder cargarlos
//! sin Godot: la CLI, los tests y la extensión usan el mismo cargador. Sólo hace E/S de lectura de
//! archivos; no usa reloj ni colecciones no deterministas.
//!
//! # Qué hace
//!
//! [`DataLoader::load_dir`] lee, en un orden fijo (el de [`DATA_FILES`], el de doc 22 §11, no el del
//! sistema de archivos), los archivos de datos y devuelve un [`GameData`]. Si algo falla devuelve
//! [`DataLoadErrors`] con **todos** los errores de **todos** los archivos (no sólo el primero), nunca
//! un `panic`.
//!
//! - Un archivo `[]` / `{}` (lista / objeto vacío) es válido. Un archivo de 0 bytes **no**: no es
//!   JSON y se informa como error de sintaxis en la línea 1.
//! - El archivo debe ser UTF-8 sin BOM.
//! - Los archivos de `data/` que no están en la lista (p. ej. `data/schema/`, `README`) se
//!   **ignoran**: el cargador sólo lee lo que conoce. Un archivo conocido que falta es un error.
//!
//! # Formato de los errores
//!
//! Cada [`DataError`] lleva el archivo y, cuando corresponde, el id de la entrada y la ruta del campo:
//! `bodies.json: body.luna: orbit.period_days: invalid type: string "x", expected f64`. El JSON
//! malformado informa línea y columna. Las entradas sin `id` legible se identifican por su posición
//! (`[3]`). Los errores semánticos (tipo equivocado, campo faltante o desconocido) se informan por
//! entrada, así que una sola corrida lista los de todas las entradas.
//!
//! # Modelos
//!
//! Doc 22 §11 sólo define el esquema de `bodies.json` (con un ejemplo); de los demás archivos sólo da
//! el nombre. Por eso [`Body`] es tipado y estricto (`deny_unknown_fields`: un campo mal escrito es
//! un error) y el resto usa modelos mínimos y extensibles ([`Entry`]: sólo `id` + campos libres)
//! que las tareas de datos posteriores reemplazarán por tipos propios. Los ids son `String`: su
//! formato y unicidad, y las referencias cruzadas, los valida el registro de ids (#32), no el
//! cargador. Las listas conservan el orden del archivo; los mapas son `BTreeMap`.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;

// Registro de ids y validación de referencias (#32): vive en `registry.rs` y se expone aquí,
// junto al cargador, que es la ruta pública de los datos.
pub use crate::registry::{
    BodyRegistry, CompanyId, FacilityId, JUMP_POINT_PREFIX, RecipeId, Registry, RegistryError,
    RegistryErrors, SocialEventId, ValidatedData, ValidatedLoadError,
};

/// Archivos que lee el cargador, en el orden en que los lee (doc 22 §11), relativos a `data/`.
pub const DATA_FILES: [&str; 10] = [
    "bodies.json",
    "goods.json",
    "recipes.json",
    "facilities.json",
    "techs.json",
    "designs.json",
    "companies.json",
    "scenario_earth.json",
    "events.json",
    "loc/es.json",
];

/// Forma de órbita de un cuerpo. El MVP sólo usa órbitas circulares (doc 22 §0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrbitKind {
    /// Circular y coplanar (`e = 0`).
    Circular,
}

/// Órbita de un cuerpo (`orbit` en `bodies.json`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Orbit {
    /// Tipo de órbita (campo `type` en el JSON).
    #[serde(rename = "type")]
    pub kind: OrbitKind,
    /// Radio orbital, en km.
    pub radius_km: f64,
    /// Período, en días.
    pub period_days: f64,
}

/// Yacimiento de un cuerpo (`deposits[]` en `bodies.json`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Deposit {
    /// Id de texto del bien (`good.*`).
    pub good: String,
    /// Reserva, en megatoneladas.
    pub reserve_mt: f64,
    /// Facilidad de acceso (0–1).
    pub access: f64,
}

/// Cuerpo celeste (entrada de `bodies.json`, doc 22 §11).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Body {
    /// Id de texto (`body.*`).
    pub id: String,
    /// Tipo de cuerpo (`moon`, `planet`...). El doc no fija la lista: es texto libre por ahora.
    pub kind: String,
    /// Id del cuerpo padre; ausente para la estrella.
    #[serde(default)]
    pub parent: Option<String>,
    /// Órbita; ausente para la estrella.
    #[serde(default)]
    pub orbit: Option<Orbit>,
    /// Radio del cuerpo, en km.
    pub radius_km: f64,
    /// Dificultad de detección por telescopio (0–10).
    pub detect_difficulty: u8,
    /// Habitabilidad (0–1); ausente si no aplica.
    #[serde(default)]
    pub habitability: Option<f64>,
    /// Yacimientos.
    #[serde(default)]
    pub deposits: Vec<Deposit>,
}

/// Entrada mínima y extensible de los archivos cuyo esquema aún no está definido: sólo exige `id`
/// y conserva el resto de los campos tal cual (en un `BTreeMap`, orden estable).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Entry {
    /// Id de texto estable (`good.hierro`, `tech.pro_0_1`...).
    pub id: String,
    /// Campos adicionales sin interpretar.
    #[serde(flatten)]
    pub fields: BTreeMap<String, Value>,
}

/// Escenario inicial de la Tierra (`scenario_earth.json`): objeto libre hasta que se defina su esquema.
pub type ScenarioEarth = BTreeMap<String, Value>;

/// Textos en español (`loc/es.json`): id de texto -> texto.
pub type Localization = BTreeMap<String, String>;

/// Todos los datos del juego cargados desde `data/`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GameData {
    /// `bodies.json`.
    pub bodies: Vec<Body>,
    /// `goods.json`.
    pub goods: Vec<Entry>,
    /// `recipes.json`.
    pub recipes: Vec<Entry>,
    /// `facilities.json`.
    pub facilities: Vec<Entry>,
    /// `techs.json`.
    pub techs: Vec<Entry>,
    /// `designs.json`.
    pub designs: Vec<Entry>,
    /// `companies.json`.
    pub companies: Vec<Entry>,
    /// `scenario_earth.json`.
    pub scenario_earth: ScenarioEarth,
    /// `events.json`.
    pub events: Vec<Entry>,
    /// `loc/es.json`.
    pub loc_es: Localization,
}

/// Un error al cargar los datos. Su `Display` es texto técnico en español para logs y la CLI.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DataError {
    /// El directorio de datos no existe.
    #[error("{}: el directorio de datos no existe", .path.display())]
    DirectoryMissing {
        /// Ruta pedida.
        path: PathBuf,
    },
    /// La ruta existe pero no es un directorio.
    #[error("{}: no es un directorio", .path.display())]
    NotADirectory {
        /// Ruta pedida.
        path: PathBuf,
    },
    /// Falta un archivo de datos requerido.
    #[error("{file}: falta el archivo")]
    FileMissing {
        /// Archivo, relativo a `data/`.
        file: &'static str,
    },
    /// No se pudo leer el archivo (permisos, es un directorio, etc.).
    #[error("{file}: no se pudo leer: {message}")]
    Unreadable {
        /// Archivo, relativo a `data/`.
        file: &'static str,
        /// Mensaje del error de E/S.
        message: String,
    },
    /// El archivo no es UTF-8 válido.
    #[error("{file}: no es UTF-8 válido (primer byte inválido en el desplazamiento {valid_up_to})")]
    InvalidUtf8 {
        /// Archivo, relativo a `data/`.
        file: &'static str,
        /// Cantidad de bytes válidos antes del primer byte inválido.
        valid_up_to: usize,
    },
    /// JSON malformado.
    #[error("{file}: JSON malformado en la línea {line}, columna {column}: {message}")]
    Syntax {
        /// Archivo, relativo a `data/`.
        file: &'static str,
        /// Línea (desde 1).
        line: usize,
        /// Columna (desde 1).
        column: usize,
        /// Descripción del problema.
        message: String,
    },
    /// El JSON es válido pero la raíz no es del tipo esperado.
    #[error("{file}: la raíz debe ser {expected}")]
    WrongRoot {
        /// Archivo, relativo a `data/`.
        file: &'static str,
        /// Qué se esperaba (`una lista` / `un objeto`).
        expected: &'static str,
    },
    /// El contenido no cumple el modelo (tipo equivocado, campo faltante o desconocido...).
    #[error("{}", format_schema(.file, .entry.as_deref(), .field.as_deref(), .message))]
    Schema {
        /// Archivo, relativo a `data/`.
        file: &'static str,
        /// Id de la entrada (o su posición `[n]` si no tiene id legible); `None` si el error no es
        /// de una entrada de una lista.
        entry: Option<String>,
        /// Ruta del campo dentro de la entrada (`orbit.period_days`); `None` si es la entrada entera.
        field: Option<String>,
        /// Descripción del problema.
        message: String,
    },
}

fn format_schema(file: &str, entry: Option<&str>, field: Option<&str>, message: &str) -> String {
    let mut out = String::from(file);
    for parte in [entry, field].into_iter().flatten() {
        out.push_str(": ");
        out.push_str(parte);
    }
    out.push_str(": ");
    out.push_str(message);
    out
}

/// Todos los errores de una carga (nunca vacío), uno por línea en su `Display`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLoadErrors(pub Vec<DataError>);

impl DataLoadErrors {
    /// Los errores, en el orden en que se detectaron.
    #[must_use]
    pub fn errors(&self) -> &[DataError] {
        &self.0
    }
}

impl fmt::Display for DataLoadErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} error(es) al cargar los datos", self.0.len())?;
        for e in &self.0 {
            write!(f, "\n  - {e}")?;
        }
        Ok(())
    }
}

impl std::error::Error for DataLoadErrors {}

/// Cargador de `data/`.
#[derive(Debug, Clone, Copy)]
pub struct DataLoader;

impl DataLoader {
    /// Carga todos los archivos de datos de `dir`.
    ///
    /// # Errors
    ///
    /// Devuelve [`DataLoadErrors`] con todos los errores encontrados (directorio inexistente,
    /// archivos faltantes o ilegibles, UTF-8 inválido, JSON malformado o datos que no cumplen el
    /// modelo). Si el directorio no existe se informa sólo eso.
    pub fn load_dir(dir: impl AsRef<Path>) -> Result<GameData, DataLoadErrors> {
        let dir = dir.as_ref();
        if !dir.exists() {
            return Err(DataLoadErrors(vec![DataError::DirectoryMissing {
                path: dir.to_path_buf(),
            }]));
        }
        if !dir.is_dir() {
            return Err(DataLoadErrors(vec![DataError::NotADirectory {
                path: dir.to_path_buf(),
            }]));
        }

        let mut errors = Vec::new();
        let [
            bodies,
            goods,
            recipes,
            facilities,
            techs,
            designs,
            companies,
            scenario,
            events,
            loc,
        ] = DATA_FILES;
        let data = GameData {
            bodies: load_list(dir, bodies, &mut errors),
            goods: load_list(dir, goods, &mut errors),
            recipes: load_list(dir, recipes, &mut errors),
            facilities: load_list(dir, facilities, &mut errors),
            techs: load_list(dir, techs, &mut errors),
            designs: load_list(dir, designs, &mut errors),
            companies: load_list(dir, companies, &mut errors),
            scenario_earth: load_object(dir, scenario, &mut errors),
            events: load_list(dir, events, &mut errors),
            loc_es: load_object(dir, loc, &mut errors),
        };
        if errors.is_empty() {
            Ok(data)
        } else {
            Err(DataLoadErrors(errors))
        }
    }
}

/// Lee y parsea un archivo a un `Value`; los errores van a `errors`.
fn read_json(dir: &Path, file: &'static str, errors: &mut Vec<DataError>) -> Option<Value> {
    let bytes = match fs::read(dir.join(file)) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            errors.push(DataError::FileMissing { file });
            return None;
        }
        Err(e) => {
            errors.push(DataError::Unreadable {
                file,
                message: e.to_string(),
            });
            return None;
        }
    };
    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(e) => {
            errors.push(DataError::InvalidUtf8 {
                file,
                valid_up_to: e.utf8_error().valid_up_to(),
            });
            return None;
        }
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(value) => Some(value),
        Err(e) => {
            // serde_json agrega " at line L column C" al mensaje; ya lo informamos aparte.
            let full = e.to_string();
            let suffix = format!(" at line {} column {}", e.line(), e.column());
            let message = full.strip_suffix(&suffix).unwrap_or(&full).to_owned();
            errors.push(DataError::Syntax {
                file,
                line: e.line(),
                column: e.column(),
                message,
            });
            None
        }
    }
}

/// Deserializa `value` como `T` y, si falla, devuelve (ruta del campo, mensaje).
fn deserialize_with_path<T: DeserializeOwned>(value: Value) -> Result<T, (Option<String>, String)> {
    serde_path_to_error::deserialize::<_, T>(value).map_err(|e| {
        let path = e.path().to_string();
        let field = if path == "." { None } else { Some(path) };
        (field, e.inner().to_string())
    })
}

/// Carga un archivo cuya raíz es una lista de entradas; informa los errores de cada entrada.
fn load_list<T: DeserializeOwned>(
    dir: &Path,
    file: &'static str,
    errors: &mut Vec<DataError>,
) -> Vec<T> {
    let Some(root) = read_json(dir, file, errors) else {
        return Vec::new();
    };
    let Value::Array(items) = root else {
        errors.push(DataError::WrongRoot {
            file,
            expected: "una lista",
        });
        return Vec::new();
    };
    let mut out = Vec::with_capacity(items.len());
    for (index, item) in items.into_iter().enumerate() {
        let entry = match item.get("id").and_then(Value::as_str) {
            Some(id) => id.to_owned(),
            None => format!("[{index}]"),
        };
        match deserialize_with_path::<T>(item) {
            Ok(parsed) => out.push(parsed),
            Err((field, message)) => {
                errors.push(DataError::Schema {
                    file,
                    entry: Some(entry),
                    field,
                    message,
                });
            }
        }
    }
    out
}

/// Carga un archivo cuya raíz es un objeto, deserializado como `T`.
fn load_object<T: DeserializeOwned + Default>(
    dir: &Path,
    file: &'static str,
    errors: &mut Vec<DataError>,
) -> T {
    let Some(root) = read_json(dir, file, errors) else {
        return T::default();
    };
    if !root.is_object() {
        errors.push(DataError::WrongRoot {
            file,
            expected: "un objeto",
        });
        return T::default();
    }
    match deserialize_with_path::<T>(root) {
        Ok(parsed) => parsed,
        Err((field, message)) => {
            errors.push(DataError::Schema {
                file,
                entry: None,
                field,
                message,
            });
            T::default()
        }
    }
}
