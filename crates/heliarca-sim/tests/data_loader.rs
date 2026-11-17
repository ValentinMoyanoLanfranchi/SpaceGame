//! Tests de integración del cargador de datos (`heliarca_sim::data`).
//!
//! Cada caso arma un directorio temporal copiando el `data/` real del repo y pisando un archivo
//! con un fixture de `tests/fixtures/`, así sólo falla lo que el caso quiere probar.

use std::fs;
use std::path::{Path, PathBuf};

use heliarca_sim::data::{DATA_FILES, DataError, DataLoadErrors, DataLoader};

fn repo_data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture {}: {e}", path.display()))
}

/// Directorio temporal con copia de `data/`; se borra al soltarse.
struct Sandbox(PathBuf);

impl Sandbox {
    fn new(caso: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("heliarca-data-{}-{caso}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for file in DATA_FILES {
            let dest = dir.join(file);
            fs::create_dir_all(dest.parent().expect("tiene padre")).expect("crear dir");
            fs::copy(repo_data().join(file), &dest).expect("copiar dato");
        }
        Self(dir)
    }

    fn put(&self, file: &str, contents: impl AsRef<[u8]>) -> &Self {
        fs::write(self.0.join(file), contents).expect("escribir");
        self
    }

    fn load(&self) -> Result<heliarca_sim::data::GameData, DataLoadErrors> {
        DataLoader::load_dir(&self.0)
    }

    fn errors(&self) -> Vec<DataError> {
        self.load().expect_err("se esperaban errores").0
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn texts(errors: &[DataError]) -> Vec<String> {
    errors.iter().map(ToString::to_string).collect()
}

#[test]
fn el_data_real_del_repo_carga_ok() {
    let data = DataLoader::load_dir(repo_data()).expect("data/ debe cargar");
    assert!(data.bodies.is_empty() && data.goods.is_empty());
    assert!(!data.loc_es.is_empty());
}

#[test]
fn un_cuerpo_valido_se_carga_con_todos_sus_campos() {
    let sb = Sandbox::new("valido");
    sb.put("bodies.json", fixture("bodies_valid.json"));
    let data = sb.load().expect("debe cargar");
    assert_eq!(data.bodies.len(), 2);
    let luna = &data.bodies[0];
    assert_eq!(luna.id, "body.luna");
    assert_eq!(luna.parent.as_deref(), Some("body.tierra"));
    assert_eq!(luna.orbit.as_ref().map(|o| o.period_days), Some(27.3));
    assert_eq!(luna.deposits.len(), 2);
    assert_eq!(luna.deposits[1].good, "good.hierro");
    assert!(data.bodies[1].orbit.is_none());
}

#[test]
fn json_malformado_informa_archivo_y_linea() {
    let errors = Sandbox::new("malformado")
        .put("bodies.json", fixture("bodies_malformed.json"))
        .errors();
    assert_eq!(errors.len(), 1);
    match &errors[0] {
        DataError::Syntax {
            file, line, column, ..
        } => {
            assert_eq!(*file, "bodies.json");
            assert_eq!(*line, 4);
            assert!(*column > 0);
        }
        otro => panic!("se esperaba Syntax, llegó {otro:?}"),
    }
    assert!(
        errors[0]
            .to_string()
            .starts_with("bodies.json: JSON malformado en la línea 4, columna ")
    );
}

#[test]
fn tipo_equivocado_informa_archivo_id_y_campo() {
    let errors = Sandbox::new("tipo")
        .put("bodies.json", fixture("bodies_wrong_type.json"))
        .errors();
    assert_eq!(errors.len(), 1);
    assert!(
        errors[0].to_string().starts_with(
            "bodies.json: body.luna: orbit.period_days: invalid type: string \"27.3\""
        ),
        "{}",
        errors[0]
    );
}

#[test]
fn campo_desconocido_es_un_error() {
    let errors = Sandbox::new("desconocido")
        .put("bodies.json", fixture("bodies_unknown_field.json"))
        .errors();
    assert_eq!(errors.len(), 1);
    let text = errors[0].to_string();
    assert!(text.starts_with("bodies.json: body.luna: "), "{text}");
    assert!(text.contains("unknown field `habitabilty`"), "{text}");
}

#[test]
fn campo_faltante_es_un_error() {
    let errors = Sandbox::new("faltante")
        .put("bodies.json", fixture("bodies_missing_field.json"))
        .errors();
    assert_eq!(errors.len(), 1);
    let text = errors[0].to_string();
    assert!(text.starts_with("bodies.json: body.luna: "), "{text}");
    assert!(text.contains("missing field `kind`"), "{text}");
}

#[test]
fn se_acumulan_los_errores_de_varias_entradas_y_archivos() {
    let sb = Sandbox::new("acumula");
    sb.put("bodies.json", fixture("bodies_two_bad.json"))
        .put("goods.json", fixture("goods_missing_id.json"))
        .put("events.json", "[")
        .put("loc/es.json", fixture("loc_bad_value.json"))
        .put("techs.json", fixture("root_not_list.json"));
    let t = texts(&sb.errors());
    assert_eq!(t.len(), 6, "{t:#?}");
    // Orden fijo: el de DATA_FILES, y dentro de un archivo el de las entradas.
    assert!(
        t[0].starts_with("bodies.json: body.luna: radius_km: invalid type: string \"grande\""),
        "{t:#?}"
    );
    assert!(t[1].starts_with("bodies.json: [2]: "), "{t:#?}");
    assert!(t[1].contains("missing field `id`"), "{t:#?}");
    assert!(
        t[2].starts_with("goods.json: [0]: ") && t[2].contains("missing field `id`"),
        "{t:#?}"
    );
    assert_eq!(t[3], "techs.json: la raíz debe ser una lista", "{t:#?}");
    assert!(
        t[4].starts_with("events.json: JSON malformado en la línea 1"),
        "{t:#?}"
    );
    assert!(
        t[5].starts_with("loc/es.json: txt.hola: invalid type: integer `5`"),
        "{t:#?}"
    );
}

#[test]
fn display_de_los_errores_lista_cada_uno() {
    let errs = Sandbox::new("display")
        .put("goods.json", "[")
        .load()
        .expect_err("error");
    let text = errs.to_string();
    assert!(
        text.starts_with("1 error(es) al cargar los datos\n  - goods.json: JSON malformado"),
        "{text}"
    );
}

#[test]
fn la_raiz_con_tipo_equivocado_se_informa() {
    let sb = Sandbox::new("raiz");
    sb.put("scenario_earth.json", fixture("root_not_object.json"));
    assert_eq!(
        texts(&sb.errors()),
        ["scenario_earth.json: la raíz debe ser un objeto"]
    );
}

#[test]
fn archivo_vacio_de_cero_bytes_es_error_de_sintaxis() {
    let sb = Sandbox::new("cero-bytes");
    sb.put("recipes.json", "");
    let errors = sb.errors();
    assert!(
        matches!(
            &errors[..],
            [DataError::Syntax {
                file: "recipes.json",
                line: 1,
                ..
            }]
        ),
        "{errors:?}"
    );
}

#[test]
fn archivo_no_utf8_es_error_tipado() {
    let sb = Sandbox::new("utf8");
    sb.put("designs.json", [b'[', b'"', 0xFF, 0xFE, b'"', b']']);
    let errors = sb.errors();
    assert_eq!(
        errors,
        [DataError::InvalidUtf8 {
            file: "designs.json",
            valid_up_to: 2
        }]
    );
}

#[test]
fn un_bom_utf8_se_informa_como_json_malformado() {
    let sb = Sandbox::new("bom");
    sb.put("designs.json", b"\xEF\xBB\xBF[]");
    let errors = sb.errors();
    assert!(
        matches!(
            &errors[..],
            [DataError::Syntax {
                file: "designs.json",
                line: 1,
                ..
            }]
        ),
        "{errors:?}"
    );
}

#[test]
fn archivo_faltante_es_error_tipado() {
    let sb = Sandbox::new("faltante-archivo");
    fs::remove_file(sb.0.join("companies.json")).expect("borrar");
    assert_eq!(
        sb.errors(),
        [DataError::FileMissing {
            file: "companies.json"
        }]
    );
}

#[test]
fn archivo_ilegible_es_error_tipado() {
    // Un directorio donde se espera el archivo: leerlo falla en todas las plataformas.
    let sb = Sandbox::new("ilegible");
    fs::remove_file(sb.0.join("facilities.json")).expect("borrar");
    fs::create_dir(sb.0.join("facilities.json")).expect("crear dir");
    let errors = sb.errors();
    assert!(
        matches!(
            &errors[..],
            [DataError::Unreadable {
                file: "facilities.json",
                ..
            }]
        ),
        "{errors:?}"
    );
}

#[test]
fn directorio_inexistente_es_error_tipado() {
    let path = std::env::temp_dir().join(format!("heliarca-no-existe-{}", std::process::id()));
    let errs = DataLoader::load_dir(&path).expect_err("debe fallar");
    assert_eq!(errs.0, [DataError::DirectoryMissing { path }]);
}

#[test]
fn una_ruta_que_es_un_archivo_no_es_un_directorio() {
    let path = repo_data().join("bodies.json");
    let errs = DataLoader::load_dir(&path).expect_err("debe fallar");
    assert_eq!(errs.0, [DataError::NotADirectory { path }]);
}

#[test]
fn los_archivos_extra_en_data_se_ignoran() {
    let sb = Sandbox::new("extra");
    sb.put("notas.json", "esto ni siquiera es JSON")
        .put("loc/en.json", "{");
    assert!(sb.load().is_ok());
}

#[test]
fn la_carga_es_determinista() {
    let sb = Sandbox::new("determinista");
    sb.put("bodies.json", fixture("bodies_valid.json")).put(
        "scenario_earth.json",
        r#"{ "b": 1, "a": [1, 2], "c": { "z": null } }"#,
    );
    let a = sb.load().expect("carga");
    let b = sb.load().expect("carga");
    assert_eq!(a, b);
    let claves: Vec<_> = a.scenario_earth.keys().map(String::as_str).collect();
    assert_eq!(claves, ["a", "b", "c"]);
}

// ---- params.json (parámetros globales, #36) ----

#[test]
fn el_params_json_real_coincide_con_el_gdd() {
    let data = DataLoader::load_dir(repo_data()).expect("data/ debe cargar");
    let p = data.params;
    assert_eq!(
        (p.days_per_week(), p.days_per_month(), p.days_per_year()),
        (7, 30, 365)
    );
    assert_eq!(p, heliarca_sim::Params::default());
}

#[test]
fn params_json_con_valores_propios_se_carga() {
    let sb = Sandbox::new("params_ok");
    sb.put(
        "params.json",
        r#"{"days_per_week": 5, "days_per_month": 10, "days_per_year": 100}"#,
    );
    let data = sb.load().expect("debe cargar");
    assert_eq!(data.params, heliarca_sim::Params::new(5, 10, 100).unwrap());
    let scenario = heliarca_sim::Scenario::from_game_data(&data);
    assert_eq!(
        heliarca_sim::Sim::new(scenario, 1)
            .params()
            .days_per_month(),
        10
    );
}

#[test]
fn params_falta_un_obligatorio() {
    let sb = Sandbox::new("params_falta");
    sb.put(
        "params.json",
        r#"{"days_per_week": 7, "days_per_year": 365}"#,
    );
    assert_eq!(
        texts(&sb.errors()),
        ["params.json: days_per_month: falta el parámetro obligatorio"]
    );
}

#[test]
fn params_tipo_equivocado_campo_desconocido_y_valores_invalidos() {
    let casos = [
        (
            r#"{"days_per_week": 7, "days_per_month": "30", "days_per_year": 365}"#,
            "params.json: days_per_month: se esperaba un entero no negativo",
        ),
        (
            r#"{"days_per_week": 7, "days_per_month": 30.5, "days_per_year": 365}"#,
            "params.json: days_per_month: se esperaba un entero no negativo",
        ),
        (
            r#"{"days_per_week": 7, "days_per_month": -3, "days_per_year": 365}"#,
            "params.json: days_per_month: se esperaba un entero no negativo",
        ),
        (
            r#"{"days_per_week": 7, "days_per_month": 30, "days_per_year": 365, "days_per_mes": 1}"#,
            "params.json: days_per_mes: campo desconocido",
        ),
        (
            r#"{"days_per_week": 7, "days_per_month": 0, "days_per_year": 365}"#,
            "params.json: days_per_month: fuera de rango",
        ),
        (
            r#"{"days_per_week": 40, "days_per_month": 30, "days_per_year": 365}"#,
            "params.json: days_per_week: 40 no puede superar",
        ),
        (
            r#"{"days_per_week": 7, "days_per_month": 30, "days_per_year": 20}"#,
            "params.json: days_per_month: 30 no puede superar",
        ),
    ];
    for (i, (json, esperado)) in casos.into_iter().enumerate() {
        let sb = Sandbox::new(&format!("params_malo{i}"));
        sb.put("params.json", json);
        let errors = texts(&sb.errors());
        assert!(
            errors.iter().any(|e| e.starts_with(esperado)),
            "{json}: {errors:?}"
        );
    }
}

#[test]
fn params_malformado_raiz_incorrecta_o_ausente() {
    let sb = Sandbox::new("params_raiz");
    sb.put("params.json", "[]");
    assert_eq!(
        texts(&sb.errors()),
        ["params.json: la raíz debe ser un objeto"]
    );
    sb.put("params.json", "{");
    assert!(texts(&sb.errors())[0].starts_with("params.json: JSON malformado"));
    fs::remove_file(sb.0.join("params.json")).unwrap();
    assert_eq!(texts(&sb.errors()), ["params.json: falta el archivo"]);
}

#[test]
fn los_errores_de_params_se_acumulan_con_los_de_otros_archivos() {
    let sb = Sandbox::new("params_acumula");
    sb.put("params.json", "{}");
    sb.put("bodies.json", fixture("bodies_wrong_type.json"));
    let errors = texts(&sb.errors());
    assert!(
        errors.iter().any(|e| e.starts_with("bodies.json")),
        "{errors:?}"
    );
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.starts_with("params.json"))
            .count(),
        3
    );
}
