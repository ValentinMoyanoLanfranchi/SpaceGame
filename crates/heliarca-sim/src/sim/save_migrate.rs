//! Migraciones del formato de guardado (issue #52, doc 21 §2).
//!
//! Un guardado viejo se **migra** antes de deserializarlo al modelo actual: cada paso `vN → vN+1`
//! transforma el JSON ([`serde_json::Value`]) y se encadenan hasta [`SAVE_FORMAT_VERSION`]. Así el
//! modelo actual (`SaveFile`) sólo conoce **un** formato y las versiones viejas viven acá, como
//! funciones chicas e independientes.
//!
//! # Reglas de la versión
//!
//! - `format_version` ausente: [`SaveError::MissingFormatVersion`].
//! - `format_version` que no es un entero sin signo (texto, negativo, decimal, `null`...):
//!   [`SaveError::InvalidFormatVersion`].
//! - Mayor que la actual: [`SaveError::UnsupportedSaveVersion`] (este código no sabe leer guardados
//!   de un juego más nuevo; no se intenta adivinar).
//! - Menor o igual: se aplican las migraciones que falten. Un paso que no puede transformar el
//!   JSON (campo faltante, tipo equivocado) devuelve [`SaveError::Migration`], nunca un pánico.
//!
//! # Formato v0 (el «histórico»)
//!
//! Es el formato anterior al v1 actual. Difiere de v1 en cuatro cosas (ver [`v0_to_v1`]):
//!
//! | v0 | v1 |
//! |---|---|
//! | `rng: {"seed": .., "state": ".."}` | `seed` y `rng_state` sueltos en la raíz |
//! | `time_s` | `time` |
//! | `state_hash`: texto hexadecimal (32 dígitos) | `state_hash: {"format_version": 3, "value": ".."}` |
//! | sin `data_fingerprint` | `data_fingerprint: null` (sin huella: la carga no la verifica) |
//!
//! El resto de las claves (`calendar`, `params`, `steps`, `events`, `commands`) es igual. El hash
//! de un guardado v0 es, por definición, el del formato de hash **3**: la migración escribe ese
//! `3` como literal (no `state_hash::FORMAT_VERSION`), de modo que si el hash cambia de versión
//! en el futuro el guardado v0 deja de verificarse en lugar de compararse con otro algoritmo.
//!
//! # Política del hash al cargar
//!
//! El `state_hash` guardado se **verifica sólo si su `format_version` es el del hash actual**
//! (`state_hash::FORMAT_VERSION`): hashes de algoritmos distintos no se pueden comparar. Si la
//! versión del hash es otra, se **recalcula y no se compara** (el `Sim` cargado ya tiene el hash
//! nuevo, y al volver a guardar queda escrito). Es lo más simple y no impide abrir guardados
//! viejos; el costo, documentado, es que un guardado de otra versión del hash no detecta
//! alteraciones por esa vía. Siguen protegiéndolo la validación estructural (`deny_unknown_fields`,
//! tipos), los valores válidos y los invariantes de [`Sim::check_invariants`](super::Sim::check_invariants).
//! Quien cambie el formato del hash debe ser consciente de ello (ver CONTRIBUTING).
//!
//! # Agregar una migración
//!
//! 1. Escribí `fn v1_to_v2(save: &mut Map<String, Json>) -> Result<(), String>` que transforme el
//!    objeto raíz de v1 en el de v2 (usá [`take`] y los demás ayudantes: devuelven `Err` con un
//!    motivo, sin `unwrap`). **No** toques `format_version`: lo actualiza el encadenado.
//! 2. Agregala al **final** de [`MIGRATIONS`] con `from: 1`.
//! 3. Subí [`SAVE_FORMAT_VERSION`] a 2. El test `la_tabla_tiene_una_migracion_por_version` falla
//!    mientras la tabla no tenga exactamente una entrada por cada versión anterior a la actual.
//! 4. Dejá un fixture `v1` en `tests/fixtures/` y un test que lo cargue con el hash esperado.

use serde_json::{Map, Value as Json};

use super::SaveError;
use super::save::SAVE_FORMAT_VERSION;

/// Un paso `vFROM → vFROM+1`: transforma el objeto raíz del guardado. El motivo del error es el
/// texto de [`SaveError::Migration`].
type MigrationFn = fn(&mut Map<String, Json>) -> Result<(), String>;

/// Un paso de la tabla de migraciones.
struct Migration {
    /// Versión que **recibe** (produce `from + 1`).
    from: u32,
    apply: MigrationFn,
}

/// Tabla de migraciones, ordenada y contigua: la entrada `i` tiene `from == i`, y la tabla tiene
/// una entrada por cada versión anterior a [`SAVE_FORMAT_VERSION`].
const MIGRATIONS: &[Migration] = &[Migration {
    from: 0,
    apply: v0_to_v1,
}];

/// Lleva `value` a [`SAVE_FORMAT_VERSION`] aplicando las migraciones que falten.
pub(super) fn migrate_to_current(value: Json) -> Result<Json, SaveError> {
    migrate_chain(MIGRATIONS, SAVE_FORMAT_VERSION, value)
}

/// El encadenado, con la tabla y la versión destino como parámetros (para probarlo con tablas de
/// varios pasos).
fn migrate_chain(table: &[Migration], current: u32, mut value: Json) -> Result<Json, SaveError> {
    let Json::Object(save) = &mut value else {
        return Err(SaveError::Malformed(
            "el guardado debe ser un objeto JSON".into(),
        ));
    };
    let found = match save.get("format_version") {
        None => return Err(SaveError::MissingFormatVersion),
        Some(version) => version
            .as_u64()
            .ok_or_else(|| SaveError::InvalidFormatVersion {
                found: version.to_string(),
            })?,
    };
    let Ok(mut version) = u32::try_from(found) else {
        return Err(unsupported(found, current));
    };
    if version > current {
        return Err(unsupported(found, current));
    }
    while version < current {
        let step = table
            .iter()
            .find(|step| step.from == version)
            .ok_or_else(|| SaveError::Migration {
                from: version,
                reason: "no hay una migración registrada para esta versión".into(),
            })?;
        (step.apply)(save).map_err(|reason| SaveError::Migration {
            from: version,
            reason,
        })?;
        version += 1;
        save.insert("format_version".into(), Json::from(version));
    }
    Ok(value)
}

fn unsupported(found: u64, supported: u32) -> SaveError {
    SaveError::UnsupportedSaveVersion { found, supported }
}

// ---- Ayudantes para escribir migraciones (todos devuelven `Err`, nunca entran en pánico) ----

/// Saca la clave `key` del objeto; error si no está.
fn take(map: &mut Map<String, Json>, key: &str) -> Result<Json, String> {
    map.remove(key)
        .ok_or_else(|| format!("falta la clave `{key}`"))
}

/// Saca la clave `key` y exige que sea un objeto.
fn take_object(map: &mut Map<String, Json>, key: &str) -> Result<Map<String, Json>, String> {
    match take(map, key)? {
        Json::Object(inner) => Ok(inner),
        other => Err(format!("`{key}` debe ser un objeto, no {}", kind(&other))),
    }
}

/// Saca la clave `key` y exige que sea un texto.
fn take_string(map: &mut Map<String, Json>, key: &str) -> Result<String, String> {
    match take(map, key)? {
        Json::String(text) => Ok(text),
        other => Err(format!("`{key}` debe ser un texto, no {}", kind(&other))),
    }
}

/// Inserta `key`; error si ya existía (un guardado de esa versión no la tiene: está dañado).
fn insert_new(map: &mut Map<String, Json>, key: &str, value: Json) -> Result<(), String> {
    if map.insert(key.to_owned(), value).is_some() {
        return Err(format!("`{key}` no existe en esta versión del formato"));
    }
    Ok(())
}

fn kind(value: &Json) -> &'static str {
    match value {
        Json::Null => "null",
        Json::Bool(_) => "un booleano",
        Json::Number(_) => "un número",
        Json::String(_) => "un texto",
        Json::Array(_) => "una lista",
        Json::Object(_) => "un objeto",
    }
}

/// Versión del formato del hash que llevan, por definición, los guardados v0 (ver el módulo).
const V0_HASH_FORMAT_VERSION: u32 = 3;

/// v0 → v1: ver la tabla «Formato v0» del módulo.
fn v0_to_v1(save: &mut Map<String, Json>) -> Result<(), String> {
    // `rng: {seed, state}` → `seed` y `rng_state` en la raíz.
    let mut rng = take_object(save, "rng")?;
    let seed = take(&mut rng, "seed").map_err(|e| format!("en `rng`: {e}"))?;
    let state = take(&mut rng, "state").map_err(|e| format!("en `rng`: {e}"))?;
    if let Some(extra) = rng.keys().next() {
        return Err(format!("clave desconocida `{extra}` en `rng`"));
    }
    insert_new(save, "seed", seed)?;
    insert_new(save, "rng_state", state)?;
    // `time_s` → `time`.
    let time = take(save, "time_s")?;
    insert_new(save, "time", time)?;
    // `state_hash` (texto) → `{format_version, value}`.
    let hash = take_string(save, "state_hash")?;
    let mut wrapped = Map::new();
    wrapped.insert("format_version".into(), Json::from(V0_HASH_FORMAT_VERSION));
    wrapped.insert("value".into(), Json::String(hash));
    insert_new(save, "state_hash", Json::Object(wrapped))?;
    // v0 no guardaba la huella de `data/`: sin huella, la carga no la verifica.
    insert_new(save, "data_fingerprint", Json::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(text: &str) -> Json {
        serde_json::from_str(text).unwrap()
    }

    #[test]
    fn la_tabla_tiene_una_migracion_por_version() {
        // Falla si se sube `SAVE_FORMAT_VERSION` sin agregar su migración (o al revés).
        assert_eq!(
            u32::try_from(MIGRATIONS.len()).unwrap(),
            SAVE_FORMAT_VERSION,
            "falta (o sobra) una migración en MIGRATIONS para SAVE_FORMAT_VERSION"
        );
    }

    #[test]
    fn la_tabla_es_contigua_desde_cero() {
        for (index, step) in MIGRATIONS.iter().enumerate() {
            assert_eq!(usize::try_from(step.from).unwrap(), index);
        }
    }

    #[test]
    fn la_version_actual_pasa_sin_cambios() {
        let v = obj(r#"{"format_version": 1, "x": 1}"#);
        assert_eq!(migrate_to_current(v.clone()).unwrap(), v);
    }

    // ---- Cadena de varios pasos con una tabla de prueba (v0 → v1 → v2 → v3) ----

    fn add_a(save: &mut Map<String, Json>) -> Result<(), String> {
        insert_new(save, "a", Json::from(1))
    }

    fn rename_a_to_b(save: &mut Map<String, Json>) -> Result<(), String> {
        let a = take(save, "a")?;
        insert_new(save, "b", a)
    }

    fn double_b(save: &mut Map<String, Json>) -> Result<(), String> {
        let b = take(save, "b")?;
        let n = b.as_i64().ok_or("`b` debe ser entero")?;
        insert_new(save, "b", Json::from(n * 2))
    }

    const TEST_TABLE: &[Migration] = &[
        Migration {
            from: 0,
            apply: add_a,
        },
        Migration {
            from: 1,
            apply: rename_a_to_b,
        },
        Migration {
            from: 2,
            apply: double_b,
        },
    ];

    #[test]
    fn encadena_todos_los_pasos_en_orden() {
        let out = migrate_chain(TEST_TABLE, 3, obj(r#"{"format_version": 0}"#)).unwrap();
        assert_eq!(out, obj(r#"{"format_version": 3, "b": 2}"#));
        // Desde un punto intermedio aplica sólo los que faltan.
        let out = migrate_chain(TEST_TABLE, 3, obj(r#"{"format_version": 2, "b": 5}"#)).unwrap();
        assert_eq!(out, obj(r#"{"format_version": 3, "b": 10}"#));
        // La misma tabla con un destino menor se detiene antes.
        let out = migrate_chain(TEST_TABLE, 1, obj(r#"{"format_version": 0}"#)).unwrap();
        assert_eq!(out, obj(r#"{"format_version": 1, "a": 1}"#));
    }

    #[test]
    fn un_paso_que_falla_informa_su_version_de_origen() {
        // Falla el paso 1 → 2 (falta `a`): el error dice `from: 1`.
        let err = migrate_chain(TEST_TABLE, 3, obj(r#"{"format_version": 1}"#)).unwrap_err();
        assert!(matches!(err, SaveError::Migration { from: 1, .. }), "{err}");
        // Tipo equivocado en el paso 2 → 3.
        let err =
            migrate_chain(TEST_TABLE, 3, obj(r#"{"format_version": 2, "b": "x"}"#)).unwrap_err();
        assert!(matches!(err, SaveError::Migration { from: 2, .. }), "{err}");
    }

    #[test]
    fn una_tabla_con_un_hueco_es_un_error_y_no_un_panico() {
        let err = migrate_chain(&TEST_TABLE[1..], 3, obj(r#"{"format_version": 0}"#)).unwrap_err();
        assert!(matches!(err, SaveError::Migration { from: 0, .. }), "{err}");
    }

    #[test]
    fn versiones_futuras_ausentes_e_invalidas() {
        for (text, found) in [
            (r#"{"format_version": 4}"#, 4),
            (r#"{"format_version": 4294967296}"#, 4_294_967_296),
            (r#"{"format_version": 18446744073709551615}"#, u64::MAX),
        ] {
            assert_eq!(
                migrate_chain(TEST_TABLE, 3, obj(text)).unwrap_err(),
                SaveError::UnsupportedSaveVersion {
                    found,
                    supported: 3
                },
                "{text}"
            );
        }
        assert_eq!(
            migrate_chain(TEST_TABLE, 3, obj("{}")).unwrap_err(),
            SaveError::MissingFormatVersion
        );
        for bad in [
            "\"1\"",
            "-1",
            "1.5",
            "null",
            "true",
            "[]",
            "18446744073709551616",
        ] {
            let v = obj(&format!(r#"{{"format_version": {bad}}}"#));
            assert!(
                matches!(
                    migrate_chain(TEST_TABLE, 3, v),
                    Err(SaveError::InvalidFormatVersion { .. })
                ),
                "{bad}"
            );
        }
        assert!(matches!(
            migrate_chain(TEST_TABLE, 3, obj("[1]")),
            Err(SaveError::Malformed(_))
        ));
    }

    #[test]
    fn v0_a_v1_no_entra_en_panico_con_json_raro() {
        let weird = [
            Json::Null,
            Json::from(1),
            Json::from("x"),
            Json::Array(vec![]),
            Json::Object(Map::new()),
        ];
        for key in ["rng", "time_s", "state_hash", "seed", "data_fingerprint"] {
            for value in &weird {
                let mut save = Map::new();
                save.insert(key.to_owned(), value.clone());
                // Sólo importa que devuelva (casi siempre `Err`) y no entre en pánico.
                let _ = v0_to_v1(&mut save);
            }
        }
    }
}
