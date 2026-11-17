//! Tests de integración del registro de ids y del validador de referencias (#32).
//!
//! Igual que `data_loader.rs`: un directorio temporal con copia del `data/` real, pisando los
//! archivos que el caso quiere probar con fixtures de `tests/fixtures/`.

use std::fs;
use std::path::{Path, PathBuf};

use heliarca_sim::data::{DATA_FILES, DataLoader, Registry, RegistryError, ValidatedLoadError};

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
        let dir = std::env::temp_dir().join(format!("heliarca-reg-{}-{caso}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for file in DATA_FILES {
            let dest = dir.join(file);
            fs::create_dir_all(dest.parent().expect("tiene padre")).expect("crear dir");
            fs::copy(repo_data().join(file), &dest).expect("copiar dato");
        }
        Self(dir)
    }

    fn put(&self, file: &str, fixture_name: &str) -> &Self {
        fs::write(self.0.join(file), fixture(fixture_name)).expect("escribir");
        self
    }

    fn registry_errors(&self) -> Vec<RegistryError> {
        match DataLoader::load_validated(&self.0) {
            Err(ValidatedLoadError::Registry(e)) => e.0,
            otro => panic!("se esperaban errores de registro: {otro:?}"),
        }
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn el_data_real_del_repo_valida_ok() {
    let validated = DataLoader::load_validated(repo_data()).expect("data/ debe validar");
    assert!(validated.registry.bodies().is_empty());
}

#[test]
fn el_ejemplo_del_doc_22_valida_y_resuelve_ids() {
    let sb = Sandbox::new("ok");
    sb.put("bodies.json", "registry_bodies_valid.json")
        .put("goods.json", "registry_goods_valid.json");
    let validated = DataLoader::load_validated(&sb.0).expect("debe validar");
    let reg = &validated.registry;
    let luna = reg.bodies().from_text("body.luna").expect("luna");
    assert_eq!(reg.bodies().to_text(luna).as_deref(), Some("body.luna"));
    assert_eq!(heliarca_sim::ids::EntityId::raw(luna), 1);
    // `load_dir` sigue andando igual y el registro se puede reconstruir.
    let data = DataLoader::load_dir(&sb.0).expect("carga");
    assert!(Registry::build(&data).is_ok());
}

#[test]
fn se_reportan_todos_los_errores_de_varios_archivos_en_orden_estable() {
    let sb = Sandbox::new("todos");
    sb.put("bodies.json", "registry_bodies_errors.json")
        .put("goods.json", "registry_goods_errors.json")
        .put("techs.json", "registry_techs_errors.json");
    let errors = sb.registry_errors();
    let texts: Vec<String> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(
        texts,
        [
            // ids, archivo por archivo
            "bodies.json: id «body.a» duplicado (posiciones [0] y [1])",
            "bodies.json: [2]: id «Body.Mayus» inválido: prefijo «Body» incorrecto, se esperaba «body»",
            "bodies.json: [3]: id «sinpunto» inválido: falta el separador «.» (formato prefijo.nombre)",
            "goods.json: id «good.hierro» duplicado (posiciones [0] y [1])",
            "goods.json: [2]: id «good.Hielo» inválido: nombre inválido: sólo a-z, 0-9 y «_», y debe empezar con letra o dígito",
            "techs.json: id «tech.pro_0_1» duplicado (posiciones [0] y [2])",
            // referencias
            "bodies.json: body.huerfano: parent: «body.fantasma» no existe (se esperaba un «body.*» definido)",
            "bodies.json: body.mal_tipo: parent: «good.hierro» es de tipo «good» y se esperaba «body»",
            "bodies.json: body.mal_tipo: deposits[0].good: «good.inexistente» no existe (se esperaba un «good.*» definido)",
            "techs.json: tech.pro_1_1: prereqs[1]: «tech.ene_0_1» no existe (se esperaba un «tech.*» definido)",
            "techs.json: tech.pro_1_1: unlocks.designs[0]: «design.snd_1» no existe (se esperaba un «design.*» definido)",
            // ciclos
            "bodies.json: ciclo en «parent»: body.x -> body.y -> body.x",
        ]
    );
    // Cuatro clases distintas de error.
    let has = |f: fn(&RegistryError) -> bool| errors.iter().any(f);
    assert!(has(|e| matches!(e, RegistryError::DuplicateId { .. })));
    assert!(has(|e| matches!(e, RegistryError::InvalidId { .. })));
    assert!(has(|e| matches!(e, RegistryError::UnknownReference { .. })));
    assert!(has(|e| matches!(
        e,
        RegistryError::WrongTypeReference { .. }
    )));
    assert!(has(|e| matches!(e, RegistryError::ParentCycle { .. })));
    // Misma corrida, mismo resultado.
    assert_eq!(errors, sb.registry_errors());
}

#[test]
fn si_la_carga_falla_se_informa_solo_la_etapa_de_carga() {
    let sb = Sandbox::new("carga");
    fs::write(sb.0.join("bodies.json"), "[ {").expect("escribir");
    fs::write(
        sb.0.join("goods.json"),
        fixture("registry_goods_errors.json"),
    )
    .expect("escribir");
    match DataLoader::load_validated(&sb.0) {
        Err(ValidatedLoadError::Load(e)) => {
            assert_eq!(e.errors().len(), 1);
            assert!(e.to_string().contains("bodies.json: JSON malformado"));
        }
        otro => panic!("se esperaba error de carga: {otro:?}"),
    }
}

#[test]
fn el_display_del_error_de_etapa_lista_los_errores() {
    let sb = Sandbox::new("display");
    sb.put("goods.json", "registry_goods_errors.json");
    let err = DataLoader::load_validated(&sb.0).expect_err("debe fallar");
    let text = err.to_string();
    assert!(
        text.starts_with("2 error(es) de ids y referencias"),
        "{text}"
    );
    assert_eq!(text.lines().count(), 3);
}
