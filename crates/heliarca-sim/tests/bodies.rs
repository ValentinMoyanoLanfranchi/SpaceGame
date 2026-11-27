//! Tests del modelo `Body` (#67): esquema, regla del padre, rangos y doc 22.
//!
//! Cada caso se prueba contra el cargador **y** contra el esquema generado: deben coincidir.

use std::fs;
use std::path::{Path, PathBuf};

use heliarca_sim::data::{
    Body, BodyError, BodyKind, DATA_FILES, DataLoadErrors, DataLoader, Orbit, OrbitKind,
    OrbitRadius,
};
use heliarca_sim::schema::generate_all;
use serde_json::{Value, json};

const KINDS: [&str; 6] = [
    "star",
    "planet",
    "dwarf_planet",
    "moon",
    "asteroid",
    "gas_giant",
];

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Carga el `data/` real con `bodies.json` reemplazado por `bodies`.
fn load(name: &str, bodies: &str) -> Result<Vec<Body>, DataLoadErrors> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("bodies")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    for file in DATA_FILES {
        let dest = dir.join(file);
        fs::create_dir_all(dest.parent().expect("tiene padre")).expect("crear dir");
        fs::copy(repo().join("data").join(file), dest).expect("copiar dato");
    }
    fs::write(dir.join("bodies.json"), bodies).expect("escribir");
    DataLoader::load_dir(&dir).map(|data| data.bodies)
}

fn schema_accepts(instance: &Value) -> bool {
    let text = &generate_all().expect("genera")["bodies.schema.json"];
    let schema: Value = serde_json::from_str(text).expect("JSON");
    jsonschema::validator_for(&schema)
        .expect("esquema válido")
        .is_valid(instance)
}

/// Una luna válida con todos los campos.
fn luna() -> Value {
    json!({
        "id": "body.luna", "kind": "moon", "parent": "body.tierra",
        "orbit": {"type": "circular", "radius_km": 384_400, "period_days": 27.3},
        "radius_km": 1737, "mass_kg": 7.35e22, "detect_difficulty": 0, "habitability": 0.3,
        "deposits": [{"good": "good.hierro", "reserve_mt": 200_000, "access": 0.7}]
    })
}

/// Aplica `value` en `pointer` sobre [`luna`]; `Value::Null` en un campo opcional = ausente.
fn luna_with(pointer: &str, value: Value) -> Value {
    let mut body = luna();
    *body.pointer_mut(pointer).expect("campo existente") = value;
    body
}

/// El cargador y el esquema rechazan `body`; devuelve los errores del cargador.
fn rejected(name: &str, body: &Value) -> Vec<String> {
    let list = json!([body]);
    assert!(!schema_accepts(&list), "{name}: el esquema acepta {body}");
    let errors = load(name, &list.to_string()).expect_err(name);
    errors.errors().iter().map(ToString::to_string).collect()
}

fn accepted(name: &str, body: &Value) -> Body {
    let list = json!([body]);
    assert!(schema_accepts(&list), "{name}: el esquema rechaza {body}");
    load(name, &list.to_string()).expect(name).remove(0)
}

#[test]
fn los_seis_tipos_de_cuerpo_son_validos_y_no_hay_otros() {
    for kind in KINDS {
        accepted(kind, &luna_with("/kind", json!(kind)));
    }
    for kind in ["jump_point", "Moon", "comet", ""] {
        rejected("tipo-desconocido", &luna_with("/kind", json!(kind)));
    }
}

#[test]
fn sin_parent_solo_es_valido_para_star() {
    for (i, kind) in KINDS.into_iter().enumerate() {
        let mut absent = luna_with("/kind", json!(kind));
        absent.as_object_mut().expect("objeto").remove("parent");
        let null = luna_with("/kind", json!(kind));
        let mut null = null;
        null["parent"] = Value::Null;
        for (form, body) in [("ausente", absent), ("null", null)] {
            let name = format!("sin-padre-{i}-{form}");
            if kind == "star" {
                assert_eq!(accepted(&name, &body).parent, None);
            } else {
                assert_eq!(
                    rejected(&name, &body),
                    [
                        "bodies.json: body.luna: parent: falta: sólo un cuerpo de tipo `star` \
                      puede no tener padre"
                    ]
                );
            }
        }
    }
}

#[test]
fn el_radio_orbital_va_en_ua_o_en_km_pero_exactamente_uno() {
    let orbit = |fields: Value| {
        let mut orbit = json!({"type": "circular", "period_days": 365});
        for (key, value) in fields.as_object().expect("objeto") {
            orbit[key] = value.clone();
        }
        luna_with("/orbit", orbit)
    };
    let au = accepted("ua", &orbit(json!({"radius_au": 1.524})));
    assert_eq!(au.orbit.map(|o| o.radius), Some(OrbitRadius::Au(1.524)));
    let km = accepted("km", &orbit(json!({"radius_km": 9376, "radius_au": null})));
    assert_eq!(km.orbit.map(|o| o.radius), Some(OrbitRadius::Km(9376.0)));

    let none = rejected("sin-radio", &orbit(json!({})));
    assert!(none[0].contains("falta el radio orbital"), "{none:?}");
    let both = rejected(
        "dos-radios",
        &orbit(json!({"radius_au": 1, "radius_km": 1})),
    );
    assert!(both[0].contains("sólo uno"), "{both:?}");
}

#[test]
fn los_limites_validos_se_aceptan() {
    for (i, (pointer, value)) in [
        ("/detect_difficulty", json!(0)),
        ("/detect_difficulty", json!(10)),
        ("/habitability", json!(0)),
        ("/habitability", json!(1)),
        ("/habitability", Value::Null),
        ("/deposits/0/access", json!(0)),
        ("/deposits/0/access", json!(1)),
        ("/mass_kg", json!(1.989e30)),
    ]
    .into_iter()
    .enumerate()
    {
        accepted(&format!("limite-{i}"), &luna_with(pointer, value));
    }
}

#[test]
fn los_valores_fuera_de_rango_se_rechazan_con_archivo_id_y_campo() {
    for (i, (pointer, value, field)) in [
        ("/detect_difficulty", json!(11), "detect_difficulty"),
        ("/habitability", json!(-0.01), "habitability"),
        ("/habitability", json!(1.01), "habitability"),
        ("/radius_km", json!(0), "radius_km"),
        ("/radius_km", json!(-1), "radius_km"),
        ("/mass_kg", json!(0), "mass_kg"),
        ("/mass_kg", json!(-7.35e22), "mass_kg"),
        ("/orbit/radius_km", json!(0), "orbit.radius_km"),
        ("/orbit/period_days", json!(0), "orbit.period_days"),
        ("/deposits/0/reserve_mt", json!(0), "deposits[0].reserve_mt"),
        ("/deposits/0/access", json!(1.5), "deposits[0].access"),
        ("/deposits/0/access", json!(-0.5), "deposits[0].access"),
    ]
    .into_iter()
    .enumerate()
    {
        let errors = rejected(&format!("rango-{i}"), &luna_with(pointer, value));
        assert_eq!(errors.len(), 1, "{errors:?}");
        let prefix = format!("bodies.json: body.luna: {field}: fuera de rango: ");
        assert!(errors[0].starts_with(&prefix), "{errors:?}");
    }
}

#[test]
fn mass_kg_es_obligatorio() {
    let mut body = luna();
    body.as_object_mut().expect("objeto").remove("mass_kg");
    let errors = rejected("sin-masa", &body);
    assert!(errors[0].contains("mass_kg"), "{errors:?}");
}

/// JSON no puede escribir NaN ni infinito: se prueban armando el `Body` en código.
#[test]
fn validate_exige_valores_finitos_y_junta_todos_los_errores() {
    let mut body = accepted("finitos", &luna());
    assert_eq!(body.validate(), Ok(()));
    assert_eq!(body.kind, BodyKind::Moon);
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        body.parent = None;
        body.radius_km = bad;
        body.mass_kg = bad;
        body.habitability = Some(bad);
        body.orbit = Some(Orbit {
            kind: OrbitKind::Circular,
            radius: OrbitRadius::Au(bad),
            period_days: bad,
        });
        body.deposits[0].reserve_mt = bad;
        body.deposits[0].access = bad;
        let errors = body.validate().expect_err("no es finito");
        let fields: Vec<&str> = errors.iter().map(BodyError::field).collect();
        assert_eq!(
            fields,
            [
                "parent",
                "orbit.radius_au",
                "orbit.period_days",
                "radius_km",
                "mass_kg",
                "habitability",
                "deposits[0].reserve_mt",
                "deposits[0].access",
            ],
            "{bad}"
        );
    }
}

fn doc22() -> String {
    fs::read_to_string(repo().join("Documentacion/22_datos_iniciales_mvp.md"))
        .expect("leer doc 22")
        .replace("\r\n", "\n")
}

#[test]
fn el_ejemplo_de_cuerpo_del_doc_22_carga_con_mass_kg() {
    let doc = doc22();
    let (_, rest) = doc.split_once("Ejemplo (cuerpo):").expect("hay ejemplo");
    let (_, rest) = rest.split_once("```json\n").expect("abre el bloque");
    let (example, _) = rest.split_once("```").expect("cierra el bloque");
    let body: Value = serde_json::from_str(example).expect("el ejemplo es JSON");
    assert!(accepted("doc-22", &body).mass_kg > 0.0);
}

#[test]
fn la_tabla_de_cuerpos_del_doc_22_da_la_masa_de_los_21() {
    let doc = doc22();
    let (_, rest) = doc.split_once("### 1.1 ").expect("sección 1.1");
    let (section, _) = rest.split_once("### 1.2 ").expect("sección 1.2");
    let mut rows = section.lines().filter(|l| l.starts_with('|'));
    let header: Vec<&str> = rows.next().expect("encabezado").split('|').collect();
    let column = header
        .iter()
        .position(|c| c.trim() == "Masa (kg)")
        .expect("columna de masa");
    let masses: Vec<f64> = rows
        .skip(1)
        .map(|row| {
            let cell = row.split('|').nth(column).expect("celda").trim();
            cell.replace(',', ".").parse().expect("masa numérica")
        })
        .collect();
    assert_eq!(masses.len(), 21);
    assert!(masses.iter().all(|m| m.is_finite() && *m > 0.0));
}
