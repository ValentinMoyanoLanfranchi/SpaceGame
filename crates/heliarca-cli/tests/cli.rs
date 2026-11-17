//! Pruebas de integración: corren el binario real y verifican salida y códigos de salida.

use heliarca_sim::data::DATA_FILES;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_heliarca-cli"))
        .args(args)
        .output()
        .expect("no se pudo lanzar heliarca-cli")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn help_lista_los_subcomandos_y_sale_con_0() {
    let output = cli(&["--help"]);
    assert_eq!(output.status.code(), Some(0));
    let out = text(&output.stdout);
    assert!(out.contains("run"), "{out}");
    assert!(out.contains("validate-data"), "{out}");
    assert!(
        out.contains("--seed") && out.contains("--data-dir"),
        "{out}"
    );
}

#[test]
fn version_sale_con_0() {
    let output = cli(&["--version"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(text(&output.stdout).contains("heliarca-cli"));
}

#[test]
fn los_errores_de_uso_salen_con_2() {
    for args in [
        &["subcomando-inexistente"][..],
        &["--seed", "abc", "run"],
        &["--opcion-desconocida"],
    ] {
        let output = cli(args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(!output.stderr.is_empty(), "{args:?}");
    }
}

#[test]
fn run_sin_implementar_sale_con_1() {
    let output = cli(&["run"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("no está implementado"));
}

// ---- validate-data ----

/// Archivos de datos, en el orden del cargador.
const FILES: [&str; DATA_FILES.len()] = DATA_FILES;

const TIERRA: &str =
    r#"{"id":"body.tierra","kind":"planet","radius_km":6371.0,"detect_difficulty":0}"#;
const LUNA: &str = r#"{"id":"body.luna","kind":"moon","parent":"body.tierra","orbit":{"type":"circular","radius_km":384400.0,"period_days":27.32},"radius_km":1737.4,"detect_difficulty":1}"#;

fn real_data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

/// Copia el `data/` real a un directorio temporal propio del test (en `target/`, sin reloj).
fn fixture(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("validate-data")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    for file in FILES {
        let to = dir.join(file);
        fs::create_dir_all(to.parent().unwrap()).unwrap();
        fs::copy(real_data().join(file), to).unwrap();
    }
    dir
}

fn write(dir: &Path, file: &str, content: impl AsRef<[u8]>) {
    fs::write(dir.join(file), content).unwrap();
}

/// Corre `validate-data` sobre `dir`: (código, stdout, líneas de stderr con `/` como separador).
fn validate(dir: &Path) -> (Option<i32>, String, Vec<String>) {
    let output = cli(&["validate-data", "--data-dir", dir.to_str().unwrap()]);
    let err = text(&output.stderr).replace('\\', "/");
    (
        output.status.code(),
        text(&output.stdout),
        err.lines().map(str::to_owned).collect(),
    )
}

fn has(lines: &[String], parts: &[&str]) -> bool {
    lines.iter().any(|l| parts.iter().all(|p| l.contains(p)))
}

#[test]
fn el_data_real_es_valido() {
    let (code, out, err) = validate(&real_data());
    assert_eq!(code, Some(0), "{err:?}");
    assert!(err.is_empty(), "{err:?}");
    assert_eq!(
        out.trim(),
        format!("OK: {} archivos validados", FILES.len())
    );
}

#[test]
fn data_dir_por_defecto_es_data_relativo_al_directorio_actual() {
    let output = Command::new(env!("CARGO_BIN_EXE_heliarca-cli"))
        .arg("validate-data")
        .current_dir(real_data().join(".."))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
}

#[test]
fn un_directorio_inexistente_sale_con_1_y_lo_dice() {
    let dir = fixture("inexistente").join("no-existe");
    let (code, out, err) = validate(&dir);
    assert_eq!(code, Some(1));
    assert_eq!(err.len(), 1, "{err:?}");
    assert!(
        err[0].contains("el directorio de datos no existe"),
        "{err:?}"
    );
    assert_eq!(out.trim(), "1 error");
}

#[test]
fn una_ruta_que_es_un_archivo_sale_con_1() {
    let dir = fixture("es-archivo");
    let (code, _, err) = validate(&dir.join("bodies.json"));
    assert_eq!(code, Some(1));
    assert!(has(&err, &["no es un directorio"]), "{err:?}");
}

#[test]
fn juntan_todos_los_errores_de_archivos_en_una_sola_corrida() {
    let dir = fixture("archivos");
    // Esquema: tipo equivocado y campo desconocido, en la entrada 1 (la 0 es válida).
    write(
        &dir,
        "bodies.json",
        format!(
            r#"[{TIERRA},{{"id":"body.x","kind":"moon","radius_km":"grande","detect_difficulty":1,"color":"rojo"}}]"#
        ),
    );
    write(&dir, "goods.json", "[{ esto no es json"); // sintaxis
    fs::remove_file(dir.join("recipes.json")).unwrap(); // faltante
    write(&dir, "facilities.json", ""); // vacío
    write(&dir, "techs.json", [0xff_u8, 0xfe, 0x00, 0x9f]); // binario
    write(&dir, "designs.json", "{}"); // raíz equivocada
    let (code, out, err) = validate(&dir);
    assert_eq!(code, Some(1), "{err:?}");
    assert!(has(&err, &["bodies.json: /1/radius_km:"]), "{err:?}");
    assert!(has(&err, &["bodies.json: /1:", "color"]), "{err:?}");
    assert!(
        has(&err, &["goods.json: JSON malformado en la línea 1"]),
        "{err:?}"
    );
    assert!(has(&err, &["recipes.json: falta el archivo"]), "{err:?}");
    assert!(has(&err, &["facilities.json: JSON malformado"]), "{err:?}");
    assert!(has(&err, &["techs.json: no es UTF-8 válido"]), "{err:?}");
    assert!(has(&err, &["designs.json: /:"]), "{err:?}");
    assert_eq!(out.trim(), format!("{} errores en 6 archivos", err.len()));
    // Orden determinista: el de DATA_FILES.
    let order: Vec<usize> = err
        .iter()
        .map(|l| {
            FILES
                .iter()
                .position(|f| l.starts_with(&format!("{f}: ")))
                .unwrap()
        })
        .collect();
    assert!(order.windows(2).all(|w| w[0] <= w[1]), "{err:?}");
}

#[test]
fn un_problema_no_se_informa_dos_veces() {
    let dir = fixture("sin-duplicados");
    write(
        &dir,
        "bodies.json",
        format!(
            r#"[{TIERRA},{{"id":"body.x","kind":"moon","radius_km":"grande","detect_difficulty":1}}]"#
        ),
    );
    let (code, _, err) = validate(&dir);
    assert_eq!(code, Some(1));
    assert_eq!(err.len(), 1, "{err:?}");
    // Formato del esquema (ruta JSON), no el del cargador (id de la entrada).
    assert!(err[0].starts_with("bodies.json: /1/radius_km: "), "{err:?}");
}

#[test]
fn el_cargador_informa_lo_que_el_esquema_no_expresa() {
    // El esquema acepta 3.0 para un entero; el cargador (serde) no.
    let dir = fixture("cargador");
    write(
        &dir,
        "bodies.json",
        r#"[{"id":"body.x","kind":"moon","radius_km":1.0,"detect_difficulty":3.0}]"#,
    );
    let (code, _, err) = validate(&dir);
    assert_eq!(code, Some(1));
    assert_eq!(err.len(), 1, "{err:?}");
    assert!(
        err[0].starts_with("bodies.json: body.x: detect_difficulty: "),
        "{err:?}"
    );
}

#[test]
fn juntan_todos_los_errores_del_registro_en_una_sola_corrida() {
    let dir = fixture("registro");
    write(
        &dir,
        "bodies.json",
        format!(
            r#"[{TIERRA},{LUNA},{LUNA},
            {{"id":"body.a","kind":"moon","parent":"body.b","radius_km":1.0,"detect_difficulty":1}},
            {{"id":"body.b","kind":"moon","parent":"body.a","radius_km":1.0,"detect_difficulty":1}},
            {{"id":"body.c","kind":"moon","parent":"body.fantasma","radius_km":1.0,"detect_difficulty":1}},
            {{"id":"body.d","kind":"moon","radius_km":1.0,"detect_difficulty":1,
              "deposits":[{{"good":"good.nada","reserve_mt":1.0,"access":0.5}}]}}]"#
        ),
    );
    let (code, out, err) = validate(&dir);
    assert_eq!(code, Some(1), "{err:?}");
    assert!(
        has(&err, &["bodies.json:", "body.luna", "duplicado"]),
        "{err:?}"
    );
    assert!(
        has(&err, &["bodies.json:", "ciclo", "body.a", "body.b"]),
        "{err:?}"
    );
    assert!(
        has(&err, &["bodies.json: body.c: parent:", "body.fantasma"]),
        "{err:?}"
    );
    assert!(
        has(
            &err,
            &["bodies.json: body.d: deposits[0].good:", "good.nada"]
        ),
        "{err:?}"
    );
    assert_eq!(err.len(), 4, "{err:?}");
    assert_eq!(out.trim(), "4 errores en 1 archivo");
}

#[test]
fn si_hay_errores_de_archivo_no_se_valida_el_registro() {
    // body.c apunta a un padre inexistente, pero otro archivo está roto: sólo se informa éste.
    let dir = fixture("etapas");
    write(
        &dir,
        "bodies.json",
        r#"[{"id":"body.c","kind":"moon","parent":"body.fantasma","radius_km":1.0,"detect_difficulty":1}]"#,
    );
    write(&dir, "goods.json", "[");
    let (code, _, err) = validate(&dir);
    assert_eq!(code, Some(1));
    assert_eq!(err.len(), 1, "{err:?}");
    assert!(err[0].starts_with("goods.json: JSON malformado"), "{err:?}");
}

#[test]
fn datos_validos_no_triviales_salen_con_0() {
    let dir = fixture("valido");
    write(&dir, "bodies.json", format!("[{TIERRA},{LUNA}]"));
    let (code, out, err) = validate(&dir);
    assert_eq!(code, Some(0), "{err:?}");
    assert_eq!(
        out.trim(),
        format!("OK: {} archivos validados", FILES.len())
    );
}

#[test]
fn un_archivo_enorme_no_hace_panic_y_acota_los_errores() {
    let dir = fixture("enorme");
    // Lista de 200 000 números donde se esperan cuerpos: 200 000 violaciones de esquema.
    let big = format!("[{}0]", "0,".repeat(200_000));
    write(&dir, "bodies.json", big);
    let (code, _, err) = validate(&dir);
    assert_eq!(code, Some(1));
    assert!(err.len() <= 101, "{}", err.len());
    assert!(
        has(&err, &["bodies.json: ... y ", "errores de esquema más"]),
        "{err:?}"
    );
}
