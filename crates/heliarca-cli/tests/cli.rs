//! Pruebas de integración: corren el binario real y verifican salida y códigos de salida.

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
fn los_subcomandos_sin_implementar_salen_con_1() {
    for args in [&["run"][..], &["validate-data"]] {
        let output = cli(args);
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert!(text(&output.stderr).contains("no está implementado"));
    }
}
