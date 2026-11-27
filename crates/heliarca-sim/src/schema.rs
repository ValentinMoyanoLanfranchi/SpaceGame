//! Esquemas JSON (JSON Schema, draft 2020-12) de los archivos de `data/` (M0.3.1.3, #33).
//!
//! # Para qué sirven
//!
//! Los modelos de [`crate::data`] derivan [`schemars::JsonSchema`]: de ellos sale **un esquema por
//! archivo de datos**, que se versiona en `data/schema/` para que los editores validen y
//! autocompleten mientras se escriben los datos, y para que el CI detecte cuando un modelo cambia
//! sin que se actualice el esquema (ver `tests/schemas.rs`).
//!
//! # Nombres
//!
//! [`generate_all`] devuelve un esquema por cada archivo de [`DATA_FILES`], con la ruta relativa a
//! `data/schema/` que resulta de cambiar `.json` por `.schema.json`: `bodies.json` ->
//! `bodies.schema.json`, `loc/es.json` -> `loc/es.schema.json`.
//!
//! # Determinismo
//!
//! La salida es byte a byte reproducible en cualquier plataforma: las claves de cada objeto se
//! ordenan alfabéticamente (no se depende del orden interno de `serde_json`, que cambia si otro
//! crate activa `preserve_order`), se indenta con 2 espacios, los saltos de línea son `\n` y hay un
//! salto de línea final. No incluye rutas ni versiones.
//!
//! # Alcance
//!
//! El esquema describe la **forma** (tipos, campos requeridos y desconocidos) y, en `bodies.json`,
//! los rangos y la regla del padre de [`Body::validate`]; no las reglas que valida el registro de
//! ids (formato de ids, unicidad, referencias cruzadas).
//! Un caso conocido de diferencia con el cargador: para un campo entero (`detect_difficulty`) el
//! esquema acepta `3.0`, que `serde_json` rechaza al deserializar a `u8`.

use std::collections::BTreeMap;

use schemars::{JsonSchema, Schema, schema_for};
use serde::ser::{Serialize, SerializeMap, Serializer};
use serde_json::Value;

use crate::data::{Body, DATA_FILES, Entry, Localization, ScenarioEarth};

/// Ruta del esquema (relativa a `data/schema/`) de un archivo de datos (relativo a `data/`):
/// cambia la extensión `.json` por `.schema.json`.
#[must_use]
pub fn schema_path(data_file: &str) -> String {
    let stem = data_file.strip_suffix(".json").unwrap_or(data_file);
    format!("{stem}.schema.json")
}

/// Esquema del tipo `T` (la raíz del archivo).
fn root<T: JsonSchema>() -> Schema {
    schema_for!(T)
}

/// El esquema del archivo de datos `data_file`, o `None` si no es uno de [`DATA_FILES`].
#[must_use]
pub fn schema_for_data_file(data_file: &str) -> Option<Schema> {
    Some(match data_file {
        "bodies.json" => root::<Vec<Body>>(),
        "goods.json" | "recipes.json" | "facilities.json" | "techs.json" | "designs.json"
        | "companies.json" | "events.json" => root::<Vec<Entry>>(),
        "scenario_earth.json" => root::<ScenarioEarth>(),
        "loc/es.json" => root::<Localization>(),
        "params.json" => crate::params::json_schema(),
        _ => return None,
    })
}

/// Serializa un `Value` con las claves de los objetos en orden alfabético.
struct Sorted<'a>(&'a Value);

impl Serialize for Sorted<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            Value::Object(map) => {
                let mut entries: Vec<(&String, &Value)> = map.iter().collect();
                entries.sort_by(|a, b| a.0.cmp(b.0));
                let mut out = serializer.serialize_map(Some(entries.len()))?;
                for (key, value) in entries {
                    out.serialize_entry(key, &Sorted(value))?;
                }
                out.end()
            }
            Value::Array(items) => serializer.collect_seq(items.iter().map(Sorted)),
            other => other.serialize(serializer),
        }
    }
}

/// Texto canónico de un esquema: claves ordenadas, indentado con 2 espacios, LF y salto final.
///
/// # Errors
///
/// Sólo si `serde_json` no puede serializar el valor (no ocurre con esquemas generados).
pub fn to_canonical_json(schema: &Schema) -> Result<String, serde_json::Error> {
    let mut text = serde_json::to_string_pretty(&Sorted(schema.as_value()))?;
    text.push('\n');
    Ok(text)
}

/// Genera el esquema de cada archivo de [`DATA_FILES`]: ruta relativa a `data/schema/` (ver
/// [`schema_path`]) -> contenido del archivo.
///
/// # Errors
///
/// Ver [`to_canonical_json`].
pub fn generate_all() -> Result<BTreeMap<String, String>, serde_json::Error> {
    let mut out = BTreeMap::new();
    for file in DATA_FILES {
        if let Some(schema) = schema_for_data_file(file) {
            out.insert(schema_path(file), to_canonical_json(&schema)?);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hay_un_esquema_por_archivo_de_datos() {
        let all = generate_all().expect("genera");
        assert_eq!(all.len(), DATA_FILES.len());
        for file in DATA_FILES {
            assert!(all.contains_key(&schema_path(file)), "falta {file}");
        }
    }

    #[test]
    fn rutas_de_esquema() {
        assert_eq!(schema_path("bodies.json"), "bodies.schema.json");
        assert_eq!(schema_path("loc/es.json"), "loc/es.schema.json");
    }

    #[test]
    fn archivo_desconocido_no_tiene_esquema() {
        assert!(schema_for_data_file("otro.json").is_none());
    }

    #[test]
    fn salida_canonica_lf_y_salto_final() {
        for (name, text) in generate_all().expect("genera") {
            assert!(text.ends_with("}\n"), "{name}");
            assert!(!text.contains('\r'), "{name}");
        }
    }

    #[test]
    fn la_generacion_es_estable() {
        assert_eq!(generate_all().expect("1"), generate_all().expect("2"));
    }
}
