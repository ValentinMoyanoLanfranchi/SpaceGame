//! Guardado y carga versionados del estado (issue #50), sólo con la API **pública**.
//!
//! Cubre: ida y vuelta con el mismo `state_hash`, continuar tras cargar, formato (`format_version`
//! primero, salida determinista), rechazo sin pánico de JSON dañado y huella de `data/`.

use heliarca_sim::data::{Body, DataLoader, GameData};
use heliarca_sim::{
    CommandRegistry, Increment, Payload, SAVE_FORMAT_VERSION, SaveError, Scenario, Sim, Value,
};
use proptest::prelude::*;
use serde_json::Value as Json;

mod common;

fn mark(sim: &mut Sim, label: &str) {
    let args = Payload::new().with("label", Value::Text(label.into()));
    let command = CommandRegistry::builtin()
        .build("debug.mark", &args)
        .unwrap();
    sim.submit_command(command).unwrap();
}

/// 365 días en pasos de distinto tamaño, con comandos `debug.mark` en varios de ellos.
fn year_with_marks(seed: u64) -> Sim {
    let mut sim = Sim::new(Scenario::default(), seed);
    let plan = [
        Increment::D30,
        Increment::D5,
        Increment::H1,
        Increment::S5,
        Increment::D1,
    ];
    let mut days = 0u64;
    let mut step = 0usize;
    while days < 365 {
        if step.is_multiple_of(3) {
            mark(&mut sim, &format!("paso {step}"));
        }
        let inc = plan[step % plan.len()];
        sim.advance(inc).unwrap();
        days += inc.seconds() / 86_400;
        // Se sondea a veces: el cursor de eventos también se guarda.
        if step.is_multiple_of(4) {
            let _ = sim.poll_events();
        }
        step += 1;
    }
    sim
}

fn roundtrip(sim: &Sim) -> Sim {
    Sim::from_save_json(&sim.to_save_json(None).unwrap(), None).unwrap()
}

fn repo_data() -> GameData {
    DataLoader::load_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data"))
        .unwrap()
}

#[test]
fn estado_inicial_ida_y_vuelta() {
    let sim = Sim::new(Scenario::default(), 42);
    let loaded = roundtrip(&sim);
    assert_eq!(loaded.state_hash(), sim.state_hash());
    assert_eq!(loaded.seed(), 42);
    assert_eq!(loaded.steps(), 0);
    assert!(loaded.check_invariants().is_ok());
}

#[test]
fn un_anio_con_comandos_ida_y_vuelta() {
    let sim = year_with_marks(7);
    assert!(!sim.events().is_empty() && !sim.command_history().is_empty());
    let loaded = roundtrip(&sim);
    assert_eq!(loaded.state_hash(), sim.state_hash());
    assert_eq!(loaded.steps(), sim.steps());
    assert_eq!(loaded.time(), sim.time());
    assert_eq!(loaded.command_history(), sim.command_history());
    assert_eq!(loaded.events().len(), sim.events().len());
    assert_eq!(loaded.events().pending(), sim.events().pending());
    assert!(loaded.check_invariants().is_ok());
}

#[test]
fn con_comandos_pendientes_en_la_cola() {
    let mut sim = Sim::new(Scenario::default(), 3);
    mark(&mut sim, "a");
    sim.advance(Increment::D1).unwrap();
    mark(&mut sim, "b");
    mark(&mut sim, "c");
    assert_eq!(sim.pending_command_count(), 2);
    let loaded = roundtrip(&sim);
    assert_eq!(loaded.pending_command_count(), 2);
    assert_eq!(loaded.state_hash(), sim.state_hash());
    assert!(loaded.check_invariants().is_ok());

    // Los pendientes se aplican igual tras cargar.
    let (mut a, mut b) = (sim, loaded);
    a.advance(Increment::D5).unwrap();
    b.advance(Increment::D5).unwrap();
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(b.command_history().len(), 3);
}

#[test]
fn continuar_tras_cargar_da_lo_mismo_que_sin_guardar() {
    let mut original = year_with_marks(11);
    let mut loaded = roundtrip(&original);
    // Mismas operaciones sobre ambos: comandos, avances de distinto tamaño y sondeos.
    for (i, inc) in [Increment::D30, Increment::D1, Increment::S5, Increment::D5]
        .into_iter()
        .enumerate()
    {
        mark(&mut original, &format!("sigue {i}"));
        mark(&mut loaded, &format!("sigue {i}"));
        original.advance(inc).unwrap();
        loaded.advance(inc).unwrap();
        assert_eq!(loaded.state_hash(), original.state_hash(), "paso {i}");
        assert_eq!(loaded.steps(), original.steps());
        assert_eq!(
            loaded.poll_events().to_vec(),
            original.poll_events().to_vec()
        );
    }
    assert!(loaded.check_invariants().is_ok());
}

#[test]
fn guardar_es_determinista_y_estable_tras_cargar() {
    let sim = year_with_marks(5);
    let json = sim.to_save_json(None).unwrap();
    assert_eq!(json, sim.to_save_json(None).unwrap());
    let loaded = Sim::from_save_json(&json, None).unwrap();
    assert_eq!(loaded.to_save_json(None).unwrap(), json);
}

#[test]
fn el_json_lleva_format_version_primero() {
    let json = Sim::new(Scenario::default(), 1).to_save_json(None).unwrap();
    assert!(
        json.starts_with(&format!("{{\"format_version\":{SAVE_FORMAT_VERSION},")),
        "{json}"
    );
    let value: Json = serde_json::from_str(&json).unwrap();
    assert_eq!(value["format_version"], 1);
    assert_eq!(value["seed"], 1);
}

#[test]
fn el_formato_del_hash_no_cambio() {
    // El guardado tiene su propia versión; el hash de estado sigue siendo el de #45
    // (los golden de `state_hash` lo fijan; acá se comprueba que el guardado lo registra).
    let value: Json =
        serde_json::from_str(&Sim::new(Scenario::default(), 1).to_save_json(None).unwrap())
            .unwrap();
    assert_eq!(value["state_hash"]["format_version"], 3);
}

fn json_of(sim: &Sim) -> Json {
    serde_json::from_str(&sim.to_save_json(None).unwrap()).unwrap()
}

fn load(value: &Json) -> Result<Sim, SaveError> {
    Sim::from_save_json(&value.to_string(), None)
}

#[test]
fn el_json_truncado_o_dañado_da_error_sin_panico() {
    let json = year_with_marks(2).to_save_json(None).unwrap();
    // Todos los prefijos propios (en cada frontera de carácter) son JSON incompleto.
    let step = json.len() / 400 + 1;
    for end in (0..json.len()).step_by(step) {
        if json.is_char_boundary(end) {
            assert!(Sim::from_save_json(&json[..end], None).is_err(), "{end}");
        }
    }
    for text in [
        "",
        "null",
        "[]",
        "7",
        "{}",
        "{\"format_version\":1}",
        "\u{0}",
    ] {
        assert!(Sim::from_save_json(text, None).is_err(), "{text:?}");
    }
}

#[test]
fn rechaza_campos_de_mas_faltantes_y_tipos_incorrectos() {
    let base = json_of(&year_with_marks(2));
    // Sobrante, en la raíz y anidado.
    let mut v = base.clone();
    v["extra"] = Json::Bool(true);
    assert!(matches!(load(&v), Err(SaveError::Malformed(m)) if m.contains("extra")));
    let mut v = base.clone();
    v["calendar"]["otro"] = Json::from(1);
    assert!(matches!(load(&v), Err(SaveError::Malformed(_))));
    let mut v = base.clone();
    v["events"]["log"][0]["otro"] = Json::from(1);
    assert!(matches!(load(&v), Err(SaveError::Malformed(_))));
    // Faltantes (también `data_fingerprint`, aunque sea opcional en valor).
    for key in [
        "seed",
        "rng_state",
        "calendar",
        "params",
        "time",
        "steps",
        "events",
        "commands",
        "data_fingerprint",
        "state_hash",
    ] {
        let mut v = base.clone();
        v.as_object_mut().unwrap().remove(key);
        assert!(
            matches!(load(&v), Err(SaveError::Malformed(m)) if m.contains(key)),
            "{key}"
        );
    }
    let mut v = base.clone();
    v["events"]["log"][0]
        .as_object_mut()
        .unwrap()
        .remove("cause");
    assert!(matches!(load(&v), Err(SaveError::Malformed(_))));
    // Tipos incorrectos.
    for (key, bad) in [
        ("seed", Json::from("42")),
        ("time", Json::from(-1)),
        ("steps", Json::from(1.5)),
        ("params", Json::from(3)),
        ("rng_state", Json::from(3)),
    ] {
        let mut v = base.clone();
        v[key] = bad;
        assert!(matches!(load(&v), Err(SaveError::Malformed(_))), "{key}");
    }
    let mut v = base.clone();
    v["events"]["log"][0]["kind"] = Json::from("no_existe");
    assert!(matches!(load(&v), Err(SaveError::Malformed(_))));
}

#[test]
fn version_ausente_o_desconocida_es_error_claro() {
    let base = json_of(&Sim::new(Scenario::default(), 1));
    let mut v = base.clone();
    v.as_object_mut().unwrap().remove("format_version");
    assert_eq!(load(&v).unwrap_err(), SaveError::MissingFormatVersion);
    for bad in [
        Json::from(0),
        Json::from(2),
        Json::from(99),
        Json::from("1"),
    ] {
        let mut v = base.clone();
        v["format_version"] = bad;
        assert!(
            matches!(
                load(&v),
                Err(SaveError::UnknownFormatVersion { supported: 1, .. })
            ),
            "{v}"
        );
    }
}

#[test]
fn valores_invalidos_son_error() {
    let base = json_of(&year_with_marks(2));
    let case = |edit: &dyn Fn(&mut Json)| {
        let mut v = base.clone();
        edit(&mut v);
        load(&v).unwrap_err()
    };
    // Estado del RNG par, con largo incorrecto o no hexadecimal.
    for bad in [
        "00000000000000000000000000000002",
        "abc",
        "+0000000000000000000000000000001",
        "zz000000000000000000000000000001",
    ] {
        let e = case(&|v| v["rng_state"] = Json::from(bad));
        assert!(
            matches!(
                e,
                SaveError::InvalidField {
                    field: "rng_state",
                    ..
                }
            ),
            "{bad}: {e}"
        );
    }
    let e = case(&|v| v["calendar"]["day"] = Json::from(400));
    assert!(matches!(
        e,
        SaveError::InvalidField {
            field: "calendar",
            ..
        }
    ));
    let e = case(&|v| v["params"]["days_per_month"] = Json::from(0));
    assert!(matches!(
        e,
        SaveError::InvalidField {
            field: "params",
            ..
        }
    ));
    // Un comando pendiente desconocido, o con argumentos que no reconstruyen.
    let mut sim = Sim::new(Scenario::default(), 1);
    mark(&mut sim, "x");
    let pending = json_of(&sim);
    let mut v = pending.clone();
    v["commands"]["pending"][0]["kind"] = Json::from("nada.nada");
    assert!(matches!(
        load(&v),
        Err(SaveError::UnknownCommandKind { id: 0, .. })
    ));
    let mut v = pending.clone();
    v["commands"]["pending"][0]["args"] = Json::Array(vec![]);
    assert!(matches!(
        load(&v),
        Err(SaveError::CommandRebuild { id: 0, .. })
    ));
    // Un historial con un kind que no se conoce.
    let mut v = base.clone();
    v["commands"]["history"][0]["kind"] = Json::from("nada.nada");
    assert!(matches!(
        load(&v),
        Err(SaveError::UnknownCommandKind { .. })
    ));
}

#[test]
fn un_estado_incoherente_o_alterado_no_se_carga() {
    let base = json_of(&year_with_marks(2));
    // Un evento con id fuera de secuencia rompe un invariante.
    let mut v = base.clone();
    v["events"]["log"][0]["id"] = Json::from(5);
    assert!(matches!(load(&v), Err(SaveError::Inconsistent(_))));
    // Un cursor más allá del final del log también.
    let mut v = base.clone();
    v["events"]["cursor"] = Json::from(1_000_000);
    assert!(matches!(load(&v), Err(SaveError::Inconsistent(_))));
    // Un valor alterado que no rompe invariantes cambia el hash: se detecta.
    let mut v = base.clone();
    v["seed"] = Json::from(99);
    assert!(matches!(load(&v), Err(SaveError::HashMismatch { .. })));
    let mut v = base.clone();
    v["events"]["log"][0]["payload"][0]["value"]["value"] = Json::from("otro");
    assert!(matches!(load(&v), Err(SaveError::HashMismatch { .. })));
}

#[test]
fn huella_de_datos() {
    let data = repo_data();
    assert_eq!(
        data.fingerprint(),
        repo_data().fingerprint(),
        "determinista"
    );
    assert_eq!(data.fingerprint().to_string().len(), 32);

    let mut sim = Sim::new(Scenario::from_game_data(&data), 9);
    sim.advance(Increment::D5).unwrap();
    let json = sim.to_save_json(Some(&data)).unwrap();
    // Con los mismos datos carga; sin pasar datos no se comprueba nada.
    let loaded = Sim::from_save_json(&json, Some(&data)).unwrap();
    assert_eq!(loaded.state_hash(), sim.state_hash());
    assert!(Sim::from_save_json(&json, None).is_ok());

    // Datos distintos (cualquier cambio): DataMismatch.
    let mut other = repo_data();
    other.loc_es.insert("zz.cambio".into(), "otro".into());
    assert_ne!(other.fingerprint(), data.fingerprint());
    let err = Sim::from_save_json(&json, Some(&other)).unwrap_err();
    assert!(
        matches!(&err, SaveError::DataMismatch { saved: Some(_), .. }),
        "{err}"
    );

    // Un guardado sin huella no puede verificarse contra unos datos: error, no se ignora.
    let sin_huella = sim.to_save_json(None).unwrap();
    assert!(matches!(
        Sim::from_save_json(&sin_huella, Some(&data)),
        Err(SaveError::DataMismatch { saved: None, .. })
    ));
}

#[test]
fn la_huella_ve_los_cambios_numericos() {
    let body = |radius_km| Body {
        id: "body.luna".into(),
        kind: "moon".into(),
        parent: None,
        orbit: None,
        radius_km,
        detect_difficulty: 1,
        habitability: Some(0.0),
        deposits: vec![],
    };
    let con = |radius_km| {
        let mut data = repo_data();
        data.bodies.push(body(radius_km));
        data.fingerprint()
    };
    assert_eq!(con(1737.4), con(1737.4));
    assert_ne!(con(1737.4), con(1737.5));
    assert_ne!(con(1737.4), repo_data().fingerprint());
    // -0 y +0 son la misma cantidad.
    assert_eq!(con(0.0), con(-0.0));
}

proptest! {
    #![proptest_config(common::config())]

    /// Para cualquier secuencia de avances y comandos: guardar/cargar conserva el hash y
    /// continuar da lo mismo.
    #[test]
    fn ida_y_vuelta_en_cualquier_momento(
        seed in any::<u64>(),
        ops in proptest::collection::vec((0usize..5, any::<bool>()), 0..12),
    ) {
        let mut sim = Sim::new(Scenario::default(), seed);
        for (i, (inc, with_mark)) in ops.into_iter().enumerate() {
            if with_mark {
                mark(&mut sim, &format!("m{i}"));
            }
            sim.advance(Increment::ALL[inc]).unwrap();
        }
        mark(&mut sim, "pendiente");
        let mut loaded = roundtrip(&sim);
        prop_assert_eq!(loaded.state_hash(), sim.state_hash());
        sim.advance(Increment::D5).unwrap();
        loaded.advance(Increment::D5).unwrap();
        prop_assert_eq!(loaded.state_hash(), sim.state_hash());
        prop_assert!(loaded.check_invariants().is_ok());
    }
}
