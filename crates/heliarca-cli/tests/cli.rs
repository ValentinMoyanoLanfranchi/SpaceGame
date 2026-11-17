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

// ---- run ----

/// Corre `run` sobre el `data/` real con los argumentos dados.
fn run_real(args: &[&str]) -> Output {
    let data = real_data();
    let mut all = vec!["run", "--data-dir", data.to_str().unwrap()];
    all.extend_from_slice(args);
    cli(&all)
}

/// Valor de la línea `<clave>: <valor>` de la salida.
fn field<'a>(out: &'a str, key: &str) -> &'a str {
    out.lines()
        .find_map(|line| line.strip_prefix(key).and_then(|r| r.strip_prefix(": ")))
        .unwrap_or_else(|| panic!("falta la línea «{key}»: {out}"))
}

/// La salida sin la línea de duración real (lo único no determinista).
fn without_duration(out: &str) -> String {
    out.lines()
        .filter(|line| !line.starts_with("Duración real: "))
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_hash(text: &str) -> bool {
    text.len() == 32 && text.chars().all(|c| c.is_ascii_hexdigit())
}

#[test]
fn run_40_anos_termina_y_muestra_el_hash() {
    let output = run_real(&["--years", "40", "--seed", "1"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let out = text(&output.stdout);
    assert_eq!(field(&out, "Fecha final"), "Año 41, día 1, 00:00:00");
    assert_eq!(field(&out, "Eventos"), "0");
    assert_eq!(field(&out, "Pasos"), "14600");
    assert!(is_hash(field(&out, "Hash de estado")), "{out}");
}

#[test]
fn run_muestra_la_duracion_real_con_formato_numerico() {
    let out = text(&run_real(&["--years", "1"]).stdout);
    let line = field(&out, "Duración real");
    // «<s>,<mmm> s» y, si el tiempo es > 0, « (<n> pasos/s)».
    let (value, rest) = line.split_once(" s").expect("contiene « s»");
    assert!(
        rest.is_empty()
            || rest
                .strip_prefix(" (")
                .and_then(|r| r.strip_suffix(" pasos/s)"))
                .is_some_and(|n| n.parse::<u64>().is_ok()),
        "{line}"
    );
    let (secs, millis) = value.split_once(',').expect("coma decimal");
    assert!(secs.parse::<u64>().is_ok() && millis.len() == 3, "{value}");
    assert!(millis.parse::<u16>().is_ok(), "{value}");
}

#[test]
fn run_es_determinista_y_depende_de_semilla_y_anos() {
    let base = text(&run_real(&["--years", "2", "--seed", "7"]).stdout);
    let again = text(&run_real(&["--years", "2", "--seed", "7"]).stdout);
    assert_eq!(without_duration(&base), without_duration(&again));
    let other_seed = text(&run_real(&["--years", "2", "--seed", "8"]).stdout);
    assert_ne!(
        field(&base, "Hash de estado"),
        field(&other_seed, "Hash de estado")
    );
    let other_years = text(&run_real(&["--years", "3", "--seed", "7"]).stdout);
    assert_ne!(
        field(&base, "Hash de estado"),
        field(&other_years, "Hash de estado")
    );
}

#[test]
fn run_con_distintos_incrementos_llega_a_la_misma_fecha() {
    // 3 años = 1095 días: múltiplo de D1/D5 pero no de D30 (el último tramo se achica).
    let mut dates = Vec::new();
    // (S5 haría 19 millones de pasos para 3 años: queda fuera por tiempo.)
    for increment in ["h1", "d1", "d5", "d30"] {
        let output = run_real(&["--years", "3", "--increment", increment]);
        assert_eq!(output.status.code(), Some(0), "{increment}");
        let out = text(&output.stdout);
        assert_eq!(field(&out, "Incremento"), increment);
        dates.push(field(&out, "Fecha final").to_owned());
    }
    assert!(
        dates.iter().all(|d| d == "Año 4, día 1, 00:00:00"),
        "{dates:?}"
    );
}

#[test]
fn run_con_anos_0_es_valido_y_no_da_pasos() {
    let output = run_real(&["--years", "0"]);
    assert_eq!(output.status.code(), Some(0));
    let out = text(&output.stdout);
    assert_eq!(field(&out, "Pasos"), "0");
    assert_eq!(field(&out, "Fecha final"), "Año 1, día 1, 00:00:00");
}

#[test]
fn run_acepta_la_semilla_maxima() {
    let output = run_real(&["--years", "1", "--seed", "18446744073709551615"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        field(&text(&output.stdout), "Semilla"),
        u64::MAX.to_string()
    );
}

#[test]
fn run_rechaza_anos_enormes_y_incrementos_invalidos_con_2() {
    for args in [
        &["--years", "4294967295"][..],
        &["--years", "10001"],
        &["--years", "-1"],
        &["--increment", "d2"],
    ] {
        let output = run_real(args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert!(!output.stderr.is_empty(), "{args:?}");
    }
}

#[test]
fn run_con_directorio_inexistente_sale_con_1_y_dice_por_que() {
    let output = cli(&["run", "--years", "1", "--data-dir", "no/existe/nunca"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(text(&output.stdout), "");
    assert!(
        text(&output.stderr).contains("el directorio de datos no existe"),
        "{}",
        text(&output.stderr)
    );
}

#[test]
fn run_con_datos_rotos_sale_con_1_con_los_errores_del_cargador() {
    let dir = fixture("run-roto");
    fs::write(dir.join("goods.json"), "{ no es json").unwrap();
    let output = cli(&["run", "--years", "1", "--data-dir", dir.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(text(&output.stdout), "");
    assert!(
        text(&output.stderr).contains("goods.json"),
        "{}",
        text(&output.stderr)
    );
}

/// Etiquetas (antes de «:») de las líneas de un bloque de salida, en orden.
fn labels(block: &str) -> Vec<&str> {
    block
        .lines()
        .filter_map(|line| line.split_once(": ").map(|(label, _)| label))
        .collect()
}

#[test]
fn el_ejemplo_del_readme_tiene_el_formato_de_la_salida_real() {
    let readme = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../README.md"))
        .unwrap()
        .replace("\r\n", "\n");
    let start = readme
        .find("Años simulados:")
        .expect("el README no tiene el ejemplo de `run`");
    let block = &readme[start..];
    let block = &block[..block.find("```").expect("el ejemplo no cierra")];
    let real = text(&run_real(&["--years", "1", "--seed", "1"]).stdout);
    assert_eq!(
        labels(block),
        labels(&real),
        "README desactualizado:\n{block}"
    );
    assert_eq!(labels(&real).len(), 8);
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

// ---- run --script (#42) ----

fn script_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/script-ejemplo.json")
}

/// Escribe un script de prueba en un directorio propio del test y devuelve su ruta.
fn script_file(name: &str, content: impl AsRef<[u8]>) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("scripts");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, content).unwrap();
    path
}

fn run_script(script: &Path, extra: &[&str]) -> Output {
    let mut args = vec!["--script", script.to_str().unwrap()];
    args.extend_from_slice(extra);
    run_real(&args)
}

fn marks(entries: &[(u64, &str)]) -> String {
    let items: Vec<String> = entries
        .iter()
        .map(|(at, label)| {
            format!(
                r#"{{"at":{at},"command":{{"kind":"debug.mark","args":{{"label":"{label}"}}}}}}"#
            )
        })
        .collect();
    format!(r#"{{"version":1,"commands":[{}]}}"#, items.join(","))
}

#[test]
fn el_script_de_ejemplo_corre_y_cuenta_sus_comandos() {
    let output = run_script(&script_path(), &["--years", "1", "--seed", "3"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let out = text(&output.stdout);
    assert!(
        field(&out, "Script").ends_with("script-ejemplo.json (3 comandos)"),
        "{out}"
    );
    assert_eq!(field(&out, "Comandos aplicados"), "3");
    assert_eq!(field(&out, "Eventos"), "3");
    assert_eq!(field(&out, "Fecha final"), "Año 2, día 1, 00:00:00");
    assert!(is_hash(field(&out, "Hash de estado")), "{out}");
}

#[test]
fn sin_script_no_hay_lineas_de_script() {
    let out = text(&run_real(&["--years", "1"]).stdout);
    assert!(
        !out.contains("Script:") && !out.contains("Comandos aplicados"),
        "{out}"
    );
}

#[test]
fn el_mismo_script_con_la_misma_semilla_reproduce_el_hash() {
    let script = script_file(
        "repro.json",
        marks(&[(0, "a"), (10, "b"), (10, "c"), (200, "d")]),
    );
    let args = ["--years", "1", "--seed", "11"];
    let first = text(&run_script(&script, &args).stdout);
    let second = text(&run_script(&script, &args).stdout);
    assert_eq!(without_duration(&first), without_duration(&second));
    assert_eq!(field(&first, "Comandos aplicados"), "4");

    // Otra semilla, otro hash (el hash incluye el estado del RNG).
    let other_seed = text(&run_script(&script, &["--years", "1", "--seed", "12"]).stdout);
    assert_ne!(
        field(&first, "Hash de estado"),
        field(&other_seed, "Hash de estado")
    );

    // Otro script, otro hash: distinto `at`, distinto texto o distinto orden.
    let hash = |content: String, name: &str| {
        let path = script_file(name, content);
        text(&run_script(&path, &args).stdout)
            .lines()
            .find_map(|l| l.strip_prefix("Hash de estado: ").map(str::to_owned))
            .unwrap()
    };
    let base = field(&first, "Hash de estado").to_owned();
    assert_ne!(
        base,
        hash(
            marks(&[(0, "a"), (10, "b"), (10, "c"), (201, "d")]),
            "repro-at.json"
        )
    );
    assert_ne!(
        base,
        hash(
            marks(&[(0, "a"), (10, "b"), (10, "c"), (200, "e")]),
            "repro-texto.json"
        )
    );
    assert_ne!(
        base,
        hash(
            marks(&[(0, "a"), (10, "c"), (10, "b"), (200, "d")]),
            "repro-orden.json"
        )
    );
}

#[test]
fn un_script_vacio_da_el_mismo_hash_que_sin_script() {
    let empty = script_file("vacio.json", r#"{"version": 1, "commands": []}"#);
    let args = ["--years", "2", "--seed", "5"];
    let with = text(&run_script(&empty, &args).stdout);
    let without = text(&run_real(&args).stdout);
    assert_eq!(
        field(&with, "Hash de estado"),
        field(&without, "Hash de estado")
    );
    assert_eq!(field(&with, "Comandos aplicados"), "0");
    assert!(
        field(&with, "Script").ends_with("vacio.json (0 comandos)"),
        "{with}"
    );
}

#[test]
fn el_script_con_otros_incrementos_llega_a_la_misma_fecha_y_aplica_todo() {
    let script = script_file("incs.json", marks(&[(0, "a"), (7, "b"), (364, "c")]));
    for increment in ["h1", "d1", "d5", "d30"] {
        let output = run_script(&script, &["--years", "1", "--increment", increment]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{increment}: {}",
            text(&output.stderr)
        );
        let out = text(&output.stdout);
        assert_eq!(
            field(&out, "Fecha final"),
            "Año 2, día 1, 00:00:00",
            "{increment}"
        );
        assert_eq!(field(&out, "Comandos aplicados"), "3", "{increment}");
    }
}

/// Corre un script malo: debe salir con 1 sin imprimir resultados, y stderr debe contener todas
/// las partes dadas.
fn assert_script_error(name: &str, content: impl AsRef<[u8]>, years: &str, parts: &[&str]) {
    let script = script_file(name, content);
    let output = run_script(&script, &["--years", years]);
    let (out, err) = (text(&output.stdout), text(&output.stderr));
    assert_eq!(output.status.code(), Some(1), "{name}: {err}");
    assert!(out.is_empty(), "{name}: no debe imprimir resultados: {out}");
    assert!(
        err.contains(name),
        "{name}: el mensaje nombra el archivo: {err}"
    );
    for part in parts {
        assert!(err.contains(part), "{name}: falta {part:?} en: {err}");
    }
}

#[test]
fn un_comando_invalido_indica_su_posicion_y_el_motivo() {
    let mark = |at: u64, args: &str| {
        format!(r#"{{"at":{at},"command":{{"kind":"debug.mark","args":{args}}}}}"#)
    };
    let ok = mark(0, r#"{"label":"a"}"#);
    let wrap =
        |entries: &[String]| format!(r#"{{"version":1,"commands":[{}]}}"#, entries.join(","));

    assert_script_error(
        "kind.json",
        wrap(&[
            ok.clone(),
            ok.clone(),
            r#"{"at":1,"command":{"kind":"flota.crear"}}"#.into(),
        ]),
        "1",
        &[
            "entrada 3",
            "command.kind",
            "comando desconocido: flota.crear",
            "debug.mark",
        ],
    );
    assert_script_error(
        "falta.json",
        wrap(&[ok.clone(), mark(1, "{}")]),
        "1",
        &["entrada 2", "command.args", "falta el argumento", "label"],
    );
    assert_script_error(
        "sobra.json",
        wrap(&[mark(1, r#"{"label":"a","extra":1}"#)]),
        "1",
        &["entrada 1", "command.args.extra", "argumento desconocido"],
    );
    assert_script_error(
        "tipo.json",
        wrap(&[
            ok.clone(),
            ok.clone(),
            ok.clone(),
            mark(4, r#"{"label":7}"#),
        ]),
        "1",
        &["entrada 4", "command.args.label", "debe ser texto"],
    );
    assert_script_error(
        "orden.json",
        wrap(&[mark(5, r#"{"label":"a"}"#), mark(4, r#"{"label":"b"}"#)]),
        "1",
        &["entrada 2", "campo at", "fuera de orden"],
    );
    assert_script_error(
        "negativo.json",
        wrap(&[mark(0, r#"{"label":"a"}"#).replace("\"at\":0", "\"at\":-3")]),
        "1",
        &["entrada 1", "campo at", "negativo"],
    );
    assert_script_error(
        "decimal.json",
        wrap(&[mark(0, r#"{"label":"a"}"#).replace("\"at\":0", "\"at\":1.5")]),
        "1",
        &["entrada 1", "campo at", "no es un entero"],
    );
    assert_script_error(
        "enorme.json",
        wrap(&[ok.clone(), mark(u64::MAX, r#"{"label":"a"}"#)]),
        "1",
        &["entrada 2", "campo at", "fuera de la corrida"],
    );
    assert_script_error(
        "rango.json",
        wrap(&[mark(365, r#"{"label":"a"}"#)]),
        "1",
        &["entrada 1", "campo at", "debe ser menor que 365"],
    );
    assert_script_error(
        "sin-anos.json",
        wrap(std::slice::from_ref(&ok)),
        "0",
        &["entrada 1", "--years 0"],
    );
    assert_script_error(
        "campo-extra.json",
        r#"{"version":1,"commands":[{"at":0,"cmd":1,"command":{"kind":"debug.mark","args":{"label":"a"}}}]}"#,
        "1",
        &["entrada 1", "campo cmd", "campo desconocido"],
    );
    assert_script_error(
        "sin-command.json",
        r#"{"version":1,"commands":[{"at":0}]}"#,
        "1",
        &["entrada 1", "campo command", "falta"],
    );
}

#[test]
fn errores_de_raiz_version_y_json_malformado() {
    assert_script_error(
        "v2.json",
        r#"{"version":2,"commands":[]}"#,
        "1",
        &["campo version", "no soportada: 2", "versión 1"],
    );
    assert_script_error(
        "sin-version.json",
        r#"{"commands":[]}"#,
        "1",
        &["campo version", "falta"],
    );
    assert_script_error("lista.json", "[]", "1", &["la raíz debe ser un objeto"]);
    assert_script_error(
        "raiz-extra.json",
        r#"{"version":1,"commands":[],"x":0}"#,
        "1",
        &["campo x"],
    );
    assert_script_error(
        "malo.json",
        "{\n  \"version\": 1,\n  \"commands\": [\n",
        "1",
        &["JSON malformado en la línea 4"],
    );
    assert_script_error("cero-bytes.json", "", "1", &["JSON malformado"]);
    assert_script_error(
        "binario.json",
        [0xff, 0xfe, 0x00, 0x80, 0x01],
        "1",
        &["UTF-8"],
    );
    assert_script_error(
        "profundo.json",
        "[".repeat(100_000),
        "1",
        &["JSON malformado"],
    );
    assert_script_error(
        "gigante.json",
        vec![b' '; 17 * 1024 * 1024],
        "1",
        &["supera el máximo"],
    );
}

#[test]
fn un_script_inexistente_o_un_directorio_salen_con_1() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("scripts");
    fs::create_dir_all(&dir).unwrap();
    for path in [dir.join("no-existe.json"), dir.clone()] {
        let output = run_script(&path, &[]);
        assert_eq!(output.status.code(), Some(1), "{}", path.display());
        let err = text(&output.stderr);
        assert!(err.contains("no se pudo leer"), "{err}");
        assert_eq!(text(&output.stdout), "");
    }
}

#[test]
fn script_sin_valor_es_un_error_de_uso() {
    let output = run_real(&["--script"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(text(&output.stdout), "");
    assert!(text(&output.stderr).contains("--script"));
}

#[test]
fn el_script_se_valida_antes_de_correr_aunque_sea_largo() {
    // Con 10000 años un error de forma debe salir de inmediato, no tras minutos de corrida.
    let script = script_file("largo.json", marks(&[(0, "a"), (0, "")]));
    let output = run_script(&script, &["--years", "10000", "--increment", "s5"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("entrada 2"));
}
