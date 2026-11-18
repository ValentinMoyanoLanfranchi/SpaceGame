//! Migraciones del formato de guardado (issue #52), sólo con la API **pública**.
//!
//! El fixture `tests/fixtures/save_v0.json` es un guardado en el formato **v0** (el histórico; ver
//! el módulo `save_migrate`): debe cargar en la versión actual con el `state_hash` esperado.
//!
//! # Regenerar el fixture
//!
//! Sólo si cambia el escenario de [`fixture_sim`] (no hace falta al subir `SAVE_FORMAT_VERSION`:
//! el fixture v0 debe seguir cargando). Se arma el `Sim`, se guarda en la versión actual y se
//! convierte hacia atrás a v0 con [`v1_to_v0`]:
//!
//! ```text
//! cargo test -p heliarca-sim --test save_migrations -- --ignored generar_fixture_v0
//! ```
//!
//! La prueba imprime el `state_hash` del `Sim` de origen: es la constante [`FIXTURE_V0_HASH`].

use std::fs;
use std::path::{Path, PathBuf};

use heliarca_sim::{
    CommandRegistry, Increment, Payload, SAVE_FORMAT_VERSION, SaveError, SaveFileError, Scenario,
    Sim, Value,
};
use ruzstd::encoding::{CompressionLevel, compress_to_vec};
use serde_json::{Map, Value as Json};

const FIXTURE_V0: &str = include_str!("fixtures/save_v0.json");

/// `state_hash` (formato de hash 3) del `Sim` del que se generó el fixture v0.
const FIXTURE_V0_HASH: &str = "5a35aa4877df537affed7a969d631b76";

fn mark(sim: &mut Sim, label: &str) {
    let args = Payload::new().with("label", Value::Text(label.into()));
    let command = CommandRegistry::builtin()
        .build("debug.mark", &args)
        .unwrap();
    sim.submit_command(command).unwrap();
}

/// El `Sim` de origen del fixture: eventos, historial, un comando pendiente y eventos sin sondear.
fn fixture_sim() -> Sim {
    let mut sim = Sim::new(Scenario::default(), 7);
    mark(&mut sim, "uno");
    sim.advance(Increment::D5).unwrap();
    mark(&mut sim, "dos");
    sim.advance(Increment::H1).unwrap();
    let _ = sim.poll_events();
    mark(&mut sim, "tres");
    sim.advance(Increment::D1).unwrap();
    mark(&mut sim, "pendiente");
    sim
}

/// Convierte un guardado v1 al formato v0 (el inverso de la migración; sólo para generar el fixture).
fn v1_to_v0(mut v1: Json) -> Json {
    let save = v1.as_object_mut().unwrap();
    let mut rng = Map::new();
    rng.insert("seed".into(), save.remove("seed").unwrap());
    rng.insert("state".into(), save.remove("rng_state").unwrap());
    let time = save.remove("time").unwrap();
    let hash = save.remove("state_hash").unwrap()["value"].clone();
    save.remove("data_fingerprint").unwrap();
    save.insert("format_version".into(), Json::from(0));
    save.insert("rng".into(), Json::Object(rng));
    save.insert("time_s".into(), time);
    save.insert("state_hash".into(), hash);
    v1
}

#[test]
#[ignore = "genera el fixture: ver la doc del módulo"]
fn generar_fixture_v0() {
    let sim = fixture_sim();
    let v1: Json = serde_json::from_str(&sim.to_save_json(None).unwrap()).unwrap();
    let mut text = serde_json::to_string_pretty(&v1_to_v0(v1)).unwrap();
    text.push('\n');
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/save_v0.json");
    fs::write(path, text).unwrap();
    println!("state_hash = {:032x}", sim.state_hash().as_u128());
}

fn fixture() -> Json {
    serde_json::from_str(FIXTURE_V0).unwrap()
}

fn load(value: &Json) -> Result<Sim, SaveError> {
    Sim::from_save_json(&value.to_string(), None)
}

fn hex(sim: &Sim) -> String {
    format!("{:032x}", sim.state_hash().as_u128())
}

#[test]
fn el_fixture_es_v0() {
    let v = fixture();
    assert_eq!(v["format_version"], 0);
    assert!(v.get("rng").is_some() && v.get("seed").is_none());
    assert!(v.get("time_s").is_some() && v["state_hash"].is_string());
}

#[test]
fn el_fixture_v0_carga_con_el_hash_esperado() {
    let sim = Sim::from_save_json(FIXTURE_V0, None).unwrap();
    assert_eq!(hex(&sim), FIXTURE_V0_HASH);
    assert_eq!(hex(&sim), hex(&fixture_sim()));
    assert_eq!(sim.seed(), 7);
    assert!(sim.check_invariants().is_ok());
    // Es un `Sim` vivo: sigue avanzando igual que el original.
    let (mut a, mut b) = (sim, fixture_sim());
    for inc in [Increment::D30, Increment::D1, Increment::S5] {
        a.advance(inc).unwrap();
        b.advance(inc).unwrap();
        assert_eq!(a.state_hash(), b.state_hash());
    }
}

#[test]
fn guardar_lo_migrado_da_la_version_actual_y_es_estable() {
    let sim = Sim::from_save_json(FIXTURE_V0, None).unwrap();
    let saved = sim.to_save_json(None).unwrap();
    let v: Json = serde_json::from_str(&saved).unwrap();
    assert_eq!(v["format_version"], SAVE_FORMAT_VERSION);
    assert!(v.get("rng").is_none() && v.get("time_s").is_none());
    // Recargar da el mismo estado y el mismo texto: la migración es idempotente.
    let again = Sim::from_save_json(&saved, None).unwrap();
    assert_eq!(again.state_hash(), sim.state_hash());
    assert_eq!(again.to_save_json(None).unwrap(), saved);
    // Y es el mismo texto que escribe el `Sim` original.
    assert_eq!(saved, fixture_sim().to_save_json(None).unwrap());
}

#[test]
fn el_fixture_v0_carga_desde_archivo() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("save_migrations");
    fs::create_dir_all(&dir).unwrap();
    let path: PathBuf = dir.join("v0.sav.zst");
    fs::write(
        &path,
        compress_to_vec(FIXTURE_V0.as_bytes(), CompressionLevel::Fastest),
    )
    .unwrap();
    let sim = Sim::load_from_file(&path, None).unwrap();
    assert_eq!(hex(&sim), FIXTURE_V0_HASH);
    // Una versión futura en archivo es `Save(UnsupportedSaveVersion)`.
    let mut v = fixture();
    v["format_version"] = Json::from(SAVE_FORMAT_VERSION + 1);
    fs::write(
        &path,
        compress_to_vec(v.to_string().as_bytes(), CompressionLevel::Fastest),
    )
    .unwrap();
    assert!(matches!(
        Sim::load_from_file(&path, None),
        Err(SaveFileError::Save(
            SaveError::UnsupportedSaveVersion { .. }
        ))
    ));
}

#[test]
fn una_version_futura_se_rechaza_con_un_error_claro() {
    for found in [
        u64::from(SAVE_FORMAT_VERSION) + 1,
        99,
        u64::from(u32::MAX) + 1,
        u64::MAX,
    ] {
        let mut v = fixture();
        v["format_version"] = Json::from(found);
        let err = load(&v).unwrap_err();
        assert_eq!(
            err,
            SaveError::UnsupportedSaveVersion {
                found,
                supported: SAVE_FORMAT_VERSION
            }
        );
        assert!(err.to_string().contains(&found.to_string()), "{err}");
    }
}

#[test]
fn version_ausente_e_invalida_son_errores_distintos() {
    let mut v = fixture();
    v.as_object_mut().unwrap().remove("format_version");
    assert_eq!(load(&v).unwrap_err(), SaveError::MissingFormatVersion);
    for bad in [
        Json::from("0"),
        Json::from(-1),
        Json::from(0.5),
        Json::Null,
        Json::Bool(false),
    ] {
        let mut v = fixture();
        v["format_version"] = bad;
        assert!(
            matches!(load(&v), Err(SaveError::InvalidFormatVersion { .. })),
            "{v}"
        );
    }
}

#[test]
fn v0_con_campos_faltantes_da_error_de_migracion() {
    for key in ["rng", "time_s", "state_hash"] {
        let mut v = fixture();
        v.as_object_mut().unwrap().remove(key);
        assert!(
            matches!(load(&v), Err(SaveError::Migration { from: 0, .. })),
            "{key}"
        );
    }
    for key in ["seed", "state"] {
        let mut v = fixture();
        v["rng"].as_object_mut().unwrap().remove(key);
        assert!(
            matches!(load(&v), Err(SaveError::Migration { from: 0, .. })),
            "rng.{key}"
        );
    }
    // Campos del v1 que v0 no tiene: dañado, no se pisan en silencio.
    for (key, value) in [
        ("seed", Json::from(1)),
        ("time", Json::from(1)),
        ("data_fingerprint", Json::Null),
    ] {
        let mut v = fixture();
        v[key] = value;
        assert!(
            matches!(load(&v), Err(SaveError::Migration { from: 0, .. })),
            "{key}"
        );
    }
    let mut v = fixture();
    v["rng"]["otro"] = Json::from(1);
    assert!(matches!(
        load(&v),
        Err(SaveError::Migration { from: 0, .. })
    ));
}

#[test]
fn v0_con_tipos_erroneos_da_error_sin_panico() {
    let weird = [
        Json::Null,
        Json::Bool(true),
        Json::from(-3),
        Json::from(1.5),
        Json::from("x"),
        Json::Array(vec![]),
        Json::Object(Map::new()),
    ];
    let keys: Vec<String> = fixture().as_object().unwrap().keys().cloned().collect();
    for key in keys.iter().filter(|k| k.as_str() != "format_version") {
        for bad in &weird {
            let mut v = fixture();
            v[key] = bad.clone();
            let _ = load(&v); // no debe entrar en pánico; casi siempre es error
        }
    }
    // Los que migra la migración misma dan `Migration` o, ya migrados, `Malformed`/`InvalidField`.
    let mut v = fixture();
    v["state_hash"] = Json::from(5);
    assert!(matches!(
        load(&v),
        Err(SaveError::Migration { from: 0, .. })
    ));
    let mut v = fixture();
    v["rng"] = Json::from("x");
    assert!(matches!(
        load(&v),
        Err(SaveError::Migration { from: 0, .. })
    ));
    let mut v = fixture();
    v["time_s"] = Json::from("x");
    assert!(matches!(load(&v), Err(SaveError::Malformed(_))));
    let mut v = fixture();
    v["rng"]["state"] = Json::from("abc");
    assert!(matches!(load(&v), Err(SaveError::InvalidField { .. })));
}

#[test]
fn un_v0_alterado_no_carga() {
    // El hash de v0 se verifica (migra a la versión del hash actual): un cambio se detecta.
    let mut v = fixture();
    v["rng"]["seed"] = Json::from(8);
    assert!(matches!(load(&v), Err(SaveError::HashMismatch { .. })));
    let mut v = fixture();
    v["steps"] = Json::from(1000); // `steps` no entra al hash: se acepta
    assert!(load(&v).is_ok());
}

// ---- Fuzz breve y determinista de mutaciones sobre el fixture ----

struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % u64::try_from(n).unwrap()).unwrap()
    }
}

fn junk(rng: &mut XorShift) -> Json {
    match rng.below(9) {
        0 => Json::Null,
        1 => Json::Bool(true),
        2 => Json::from(-1),
        3 => Json::from(1.5),
        4 => Json::from(u64::MAX),
        5 => Json::from("zz"),
        6 => Json::from(""),
        7 => Json::Array(vec![]),
        _ => Json::Object(Map::new()),
    }
}

fn mutate(value: &mut Json, rng: &mut XorShift) {
    match value {
        Json::Object(map) if !map.is_empty() => {
            let key = map.keys().nth(rng.below(map.len())).cloned().unwrap();
            match rng.below(4) {
                0 => {
                    map.remove(&key);
                }
                1 => {
                    map.insert(key, junk(rng));
                }
                _ => mutate(map.get_mut(&key).unwrap(), rng),
            }
        }
        Json::Array(items) if !items.is_empty() => {
            let at = rng.below(items.len());
            match rng.below(4) {
                0 => {
                    items.remove(at);
                }
                1 => items[at] = junk(rng),
                _ => mutate(&mut items[at], rng),
            }
        }
        other => *other = junk(rng),
    }
}

#[test]
fn fuzz_de_mutaciones_sobre_el_fixture_v0_no_entra_en_panico() {
    let mut rng = XorShift(0x4845_4C49_4152_4341);
    let (mut loaded, mut failed) = (0u32, 0u32);
    for _ in 0..3000 {
        let mut v = fixture();
        for _ in 0..=rng.below(3) {
            mutate(&mut v, &mut rng);
        }
        match load(&v) {
            Ok(sim) => {
                assert!(sim.check_invariants().is_ok());
                loaded += 1;
            }
            Err(_) => failed += 1,
        }
    }
    // Y mutaciones de bytes sobre el texto.
    let bytes = FIXTURE_V0.as_bytes();
    for _ in 0..3000 {
        let mut b = bytes.to_vec();
        let at = rng.below(b.len());
        match rng.below(3) {
            0 => b[at] = u8::try_from(rng.below(256)).unwrap(),
            1 => {
                b.remove(at);
            }
            _ => b.truncate(at),
        }
        if let Ok(text) = String::from_utf8(b) {
            let _ = Sim::from_save_json(&text, None);
        }
    }
    assert!(
        failed > 0 && loaded < 3000,
        "{loaded} cargaron, {failed} fallaron"
    );
}
