//! Tests de integración de los esquemas JSON (`heliarca_sim::schema`, #33).
//!
//! - Los esquemas versionados en `data/schema/` coinciden con los generados desde los modelos
//!   (si no, falla con las instrucciones para regenerarlos).
//! - Los archivos de `data/` validan contra su esquema.
//! - El esquema y el cargador (`DataLoader`) están de acuerdo sobre qué es válido.
//!
//! Para regenerar los esquemas: `UPDATE_SCHEMAS=1 cargo test -p heliarca-sim --test schemas`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use heliarca_sim::data::{DATA_FILES, DataLoader};
use heliarca_sim::schema::{generate_all, schema_path};
use jsonschema::Validator;
use serde_json::{Value, json};

const REGENERAR: &str = "UPDATE_SCHEMAS=1 cargo test -p heliarca-sim --test schemas";

fn repo_data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn schema_dir() -> PathBuf {
    repo_data().join("schema")
}

/// Normaliza los saltos de línea: un checkout con `autocrlf` no debe romper la comparación.
fn lf(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// Archivos `*.json` bajo `dir`, como rutas relativas con `/`, ordenadas.
fn json_files(dir: &Path) -> BTreeSet<String> {
    fn walk(dir: &Path, prefix: &str, out: &mut BTreeSet<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let rel = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            let path = entry.path();
            if path.is_dir() {
                walk(&path, &rel, out);
            } else if path.extension().is_some_and(|e| e == "json") {
                out.insert(rel);
            }
        }
    }
    let mut out = BTreeSet::new();
    walk(dir, "", &mut out);
    out
}

#[test]
fn los_esquemas_versionados_coinciden_con_los_generados() {
    let generated = generate_all().expect("generar los esquemas");
    let dir = schema_dir();

    if std::env::var_os("UPDATE_SCHEMAS").is_some() {
        for (name, text) in &generated {
            let path = dir.join(name);
            fs::create_dir_all(path.parent().expect("tiene padre")).expect("crear directorio");
            fs::write(&path, text).expect("escribir el esquema");
        }
        for stale in json_files(&dir) {
            if !generated.contains_key(&stale) {
                fs::remove_file(dir.join(&stale)).expect("borrar esquema huérfano");
            }
        }
        return;
    }

    let mut problemas = Vec::new();
    for (name, expected) in &generated {
        match fs::read_to_string(dir.join(name)) {
            Ok(actual) if lf(&actual) == *expected => {}
            Ok(_) => problemas.push(format!("data/schema/{name}: desactualizado")),
            Err(e) => problemas.push(format!("data/schema/{name}: no se pudo leer ({e})")),
        }
    }
    for extra in json_files(&dir) {
        if !generated.contains_key(&extra) {
            problemas.push(format!(
                "data/schema/{extra}: no corresponde a ningún archivo de datos"
            ));
        }
    }
    assert!(
        problemas.is_empty(),
        "Los esquemas de data/schema/ no coinciden con los modelos de heliarca_sim::data:\n  - {}\n\
         Regeneralos con:\n    {REGENERAR}\ny commiteá el resultado.",
        problemas.join("\n  - ")
    );
}

/// Validador del esquema del archivo de datos `file`, construido desde el esquema generado.
fn validator(file: &str) -> Validator {
    let all = generate_all().expect("generar los esquemas");
    let text = all.get(&schema_path(file)).expect("hay esquema");
    let schema: Value = serde_json::from_str(text).expect("el esquema es JSON");
    jsonschema::validator_for(&schema).expect("el esquema es válido")
}

fn parse(text: &str) -> Value {
    serde_json::from_str(text).expect("JSON de prueba")
}

#[test]
fn los_esquemas_versionados_son_esquemas_validos() {
    for file in DATA_FILES {
        let path = schema_dir().join(schema_path(file));
        let text = fs::read_to_string(&path).expect("leer esquema versionado");
        let schema = parse(&text);
        assert!(
            jsonschema::meta::is_valid(&schema),
            "{}: no es un esquema válido",
            path.display()
        );
    }
}

#[test]
fn los_archivos_de_data_validan_contra_su_esquema() {
    for file in DATA_FILES {
        let text = fs::read_to_string(repo_data().join(file)).expect("leer dato");
        let instance = parse(&text);
        let errores: Vec<String> = validator(file)
            .iter_errors(&instance)
            .map(|e| format!("{}: {e}", e.instance_path()))
            .collect();
        assert!(
            errores.is_empty(),
            "data/{file} no valida contra su esquema:\n  - {}",
            errores.join("\n  - ")
        );
    }
}

#[test]
fn el_directorio_de_esquemas_no_molesta_al_cargador() {
    assert!(schema_dir().is_dir(), "falta data/schema/");
    if let Err(e) = DataLoader::load_dir(repo_data()) {
        panic!("{e}");
    }
}

#[test]
fn un_body_con_tipo_equivocado_no_valida() {
    let v = validator("bodies.json");
    let bueno =
        json!([{"id": "body.luna", "kind": "moon", "radius_km": 1737, "detect_difficulty": 0}]);
    assert!(v.is_valid(&bueno));
    for (caso, malo) in [
        (
            "radius_km texto",
            json!([{"id": "body.luna", "kind": "moon", "radius_km": "grande", "detect_difficulty": 0}]),
        ),
        (
            "id numérico",
            json!([{"id": 7, "kind": "moon", "radius_km": 1, "detect_difficulty": 0}]),
        ),
        (
            "dificultad fuera de u8",
            json!([{"id": "b", "kind": "moon", "radius_km": 1, "detect_difficulty": 300}]),
        ),
        (
            "dificultad negativa",
            json!([{"id": "b", "kind": "moon", "radius_km": 1, "detect_difficulty": -1}]),
        ),
        (
            "órbita elíptica",
            json!([{"id": "b", "kind": "moon", "radius_km": 1, "detect_difficulty": 0,
            "orbit": {"type": "elliptical", "radius_km": 1, "period_days": 1}}]),
        ),
        (
            "yacimientos no es lista",
            json!([{"id": "b", "kind": "moon", "radius_km": 1, "detect_difficulty": 0, "deposits": {}}]),
        ),
        ("raíz no es lista", json!({})),
    ] {
        assert!(!v.is_valid(&malo), "{caso}: no debería validar");
    }
}

/// Casos de consistencia: (archivo de datos, contenido). El cargador y el esquema deben coincidir.
fn casos() -> Vec<(&'static str, String)> {
    let fixture = |name: &str| {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture {}: {e}", path.display()))
    };
    let inline = |text: &str| text.to_owned();
    vec![
        ("bodies.json", fixture("bodies_valid.json")),
        ("bodies.json", fixture("bodies_wrong_type.json")),
        ("bodies.json", fixture("bodies_unknown_field.json")),
        ("bodies.json", fixture("bodies_missing_field.json")),
        ("bodies.json", fixture("bodies_two_bad.json")),
        ("goods.json", fixture("goods_missing_id.json")),
        (
            "goods.json",
            inline(r#"[{"id": "good.hierro", "nombre": "Hierro", "extra": {"a": [1, 2]}}]"#),
        ),
        ("loc/es.json", fixture("loc_bad_value.json")),
        ("loc/es.json", inline(r#"{"txt.hola": "Hola"}"#)),
        ("techs.json", fixture("root_not_list.json")),
        ("scenario_earth.json", fixture("root_not_object.json")),
        ("scenario_earth.json", inline(r#"{"inicio": {"dia": 1}}"#)),
        ("bodies.json", inline("[]")),
        (
            "params.json",
            inline(r#"{"days_per_week": 7, "days_per_month": 30, "days_per_year": 365}"#),
        ),
        (
            "params.json",
            inline(r#"{"days_per_week": 1, "days_per_month": 1, "days_per_year": 65535}"#),
        ),
        (
            "params.json",
            inline(r#"{"days_per_week": 1, "days_per_month": 1, "days_per_year": 65536}"#),
        ),
        (
            "params.json",
            inline(r#"{"days_per_week": 7, "days_per_year": 365}"#),
        ),
        (
            "params.json",
            inline(
                r#"{"days_per_week": 7, "days_per_month": 30, "days_per_year": 365, "days_per_mes": 1}"#,
            ),
        ),
        (
            "params.json",
            inline(r#"{"days_per_week": 7, "days_per_month": "30", "days_per_year": 365}"#),
        ),
        (
            "params.json",
            inline(r#"{"days_per_week": 7, "days_per_month": 0, "days_per_year": 365}"#),
        ),
        (
            "params.json",
            inline(r#"{"days_per_week": 7, "days_per_month": -1, "days_per_year": 365}"#),
        ),
        (
            "params.json",
            inline(r#"{"days_per_week": 7, "days_per_month": 30, "days_per_year": 1000001}"#),
        ),
        (
            "params.json",
            inline(r#"{"days_per_week": 7, "days_per_month": 30.5, "days_per_year": 365}"#),
        ),
        ("params.json", inline("[]")),
        (
            "bodies.json",
            inline(r#"[{"id": "b", "kind": "k", "radius_km": 1, "detect_difficulty": 256}]"#),
        ),
        (
            "bodies.json",
            inline(
                r#"[{"id": "b", "kind": "k", "radius_km": 1, "detect_difficulty": 1, "habitability": null, "parent": null, "orbit": null}]"#,
            ),
        ),
        (
            "bodies.json",
            inline(
                r#"[{"id": "b", "kind": "k", "radius_km": 1, "detect_difficulty": 1, "orbit": {"type": "circular", "radius_km": 1, "period_days": 2, "e": 0}}]"#,
            ),
        ),
    ]
}

#[test]
fn el_esquema_y_el_cargador_coinciden() {
    for (i, (file, contents)) in casos().into_iter().enumerate() {
        let dir = std::env::temp_dir().join(format!("heliarca-schema-{}-{i}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for f in DATA_FILES {
            let dest = dir.join(f);
            fs::create_dir_all(dest.parent().expect("tiene padre")).expect("crear dir");
            fs::copy(repo_data().join(f), &dest).expect("copiar dato");
        }
        fs::write(dir.join(file), &contents).expect("escribir caso");
        let carga_ok = DataLoader::load_dir(&dir).is_ok();
        let _ = fs::remove_dir_all(&dir);

        let valida = validator(file).is_valid(&parse(&contents));
        assert_eq!(
            carga_ok,
            valida,
            "caso {i} ({file}): el cargador {} pero el esquema {}\n{contents}",
            if carga_ok { "acepta" } else { "rechaza" },
            if valida { "acepta" } else { "rechaza" },
        );
    }
}

#[test]
fn los_limites_del_esquema_de_params_salen_de_params() {
    let schema: Value =
        serde_json::from_str(&generate_all().unwrap()[&schema_path("params.json")]).unwrap();
    let max = i64::try_from(heliarca_sim::Params::MAX_DAYS).unwrap();
    let max_year = i64::try_from(heliarca_sim::Params::MAX_DAYS_PER_YEAR).unwrap();
    let props = schema["properties"].as_object().expect("propiedades");
    assert_eq!(props.len(), 3);
    for (campo, def) in props {
        assert_eq!(def["minimum"], 1, "{campo}");
        let esperado = if campo == "days_per_year" {
            max_year
        } else {
            max
        };
        assert_eq!(def["maximum"], esperado, "{campo}");
    }
    assert_eq!(schema["required"].as_array().unwrap().len(), 3);
    assert_eq!(schema["additionalProperties"], false);
}

/// Divergencia conocida y deliberada: `days_per_week <= days_per_month <= days_per_year` es
/// validación semántica del cargador; JSON Schema no compara campos entre sí.
#[test]
fn el_orden_semana_mes_anio_lo_valida_solo_el_cargador() {
    let malo = r#"{"days_per_week": 40, "days_per_month": 30, "days_per_year": 365}"#;
    assert!(validator("params.json").is_valid(&parse(malo)));
    let dir = std::env::temp_dir().join(format!("heliarca-schema-orden-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    for f in DATA_FILES {
        let dest = dir.join(f);
        fs::create_dir_all(dest.parent().expect("tiene padre")).expect("crear dir");
        fs::copy(repo_data().join(f), &dest).expect("copiar dato");
    }
    fs::write(dir.join("params.json"), malo).expect("escribir");
    let carga = DataLoader::load_dir(&dir);
    let _ = fs::remove_dir_all(&dir);
    assert!(carga.is_err());
}
