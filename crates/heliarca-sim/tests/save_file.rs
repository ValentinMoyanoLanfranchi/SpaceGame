//! Guardado a archivo `.sav.zst` (issue #51), sólo con la API **pública**: ida y vuelta, tamaños,
//! sobrescritura, errores sin pánico y límites. El fallo a mitad de escritura (inyección de
//! fallos) está en los tests unitarios de `sim/save_file.rs`, que ven el escritor interno.

use std::fs;
use std::path::{Path, PathBuf};

use heliarca_sim::{
    CommandRegistry, Increment, Payload, SAVE_FILE_EXTENSION, SaveError, SaveFileError, Scenario,
    Sim, Value,
};
use ruzstd::encoding::{CompressionLevel, compress_to_vec};

fn dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("save_file_{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn names(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

fn mark(sim: &mut Sim, label: &str) {
    let args = Payload::new().with("label", Value::Text(label.into()));
    let command = CommandRegistry::builtin()
        .build("debug.mark", &args)
        .unwrap();
    sim.submit_command(command).unwrap();
}

/// 365 días en pasos de distinto tamaño, con un comando `debug.mark` cada tres pasos.
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
        step += 1;
    }
    sim
}

fn zstd(bytes: &[u8]) -> Vec<u8> {
    compress_to_vec(bytes, CompressionLevel::Fastest)
}

#[test]
fn extension_documentada() {
    assert_eq!(SAVE_FILE_EXTENSION, "sav.zst");
}

#[test]
fn ida_y_vuelta_por_archivo_conserva_el_hash_y_pesa_menos_que_el_json() {
    let dir = dir("ida_vuelta");
    let path = dir.join(format!("anio.{SAVE_FILE_EXTENSION}"));
    let sim = year_with_marks(7);
    assert!(!sim.events().is_empty() && !sim.command_history().is_empty());

    sim.save_to_file(&path, None).unwrap();
    let loaded = Sim::load_from_file(&path, None).unwrap();
    assert_eq!(loaded.state_hash(), sim.state_hash());
    assert_eq!(loaded.steps(), sim.steps());
    assert_eq!(
        loaded.to_save_json(None).unwrap(),
        sim.to_save_json(None).unwrap()
    );
    // Sin temporales sobrantes.
    assert_eq!(names(&dir), [format!("anio.{SAVE_FILE_EXTENSION}")]);

    let json = sim.to_save_json(None).unwrap().len();
    let zst = usize::try_from(fs::metadata(&path).unwrap().len()).unwrap();
    println!(
        "tamaños 365 días: json={json} B, .sav.zst={zst} B, razón={}x",
        json / zst
    );
    assert!(zst < json, "comprimido {zst} B >= json {json} B");
}

/// 365 días, un paso de 1 día y un comando por día (mucho más historial que `year_with_marks`).
#[test]
fn tamanos_de_un_anio_con_un_comando_por_dia() {
    let dir = dir("tamanos");
    let path = dir.join("a.sav.zst");
    let mut sim = Sim::new(Scenario::default(), 11);
    for day in 0..365 {
        mark(&mut sim, &format!("dia {day}"));
        sim.advance(Increment::D1).unwrap();
    }
    sim.save_to_file(&path, None).unwrap();
    let json = sim.to_save_json(None).unwrap().len();
    let zst = usize::try_from(fs::metadata(&path).unwrap().len()).unwrap();
    println!(
        "tamaños 365 días x1 comando: json={json} B, .sav.zst={zst} B, razón={}x",
        json / zst
    );
    assert!(zst < json);
    assert_eq!(
        Sim::load_from_file(&path, None).unwrap().state_hash(),
        sim.state_hash()
    );
}

#[test]
fn mismos_bytes_para_el_mismo_guardado() {
    let dir = dir("determinista");
    let sim = year_with_marks(3);
    let (a, b) = (dir.join("a.sav.zst"), dir.join("b.sav.zst"));
    sim.save_to_file(&a, None).unwrap();
    sim.save_to_file(&b, None).unwrap();
    assert_eq!(fs::read(&a).unwrap(), fs::read(&b).unwrap());
}

#[test]
fn sobrescribir_un_guardado_existente() {
    let dir = dir("sobrescribir");
    let path = dir.join("a.sav.zst");
    let mut sim = Sim::new(Scenario::default(), 1);
    sim.save_to_file(&path, None).unwrap();
    let first = Sim::load_from_file(&path, None).unwrap().state_hash();
    sim.advance(Increment::D30).unwrap();
    sim.save_to_file(&path, None).unwrap();
    let second = Sim::load_from_file(&path, None).unwrap().state_hash();
    assert_ne!(first, second);
    assert_eq!(second, sim.state_hash());
    assert_eq!(names(&dir), ["a.sav.zst"]);
}

#[test]
fn guardar_en_directorio_inexistente_es_error() {
    let dir = dir("sin_dir");
    let path = dir.join("no_existe").join("a.sav.zst");
    let err = Sim::new(Scenario::default(), 1)
        .save_to_file(&path, None)
        .unwrap_err();
    assert!(matches!(err, SaveFileError::Io { .. }), "{err}");
    assert_eq!(names(&dir), Vec::<String>::new());
}

#[test]
fn guardar_sobre_un_directorio_es_error_y_no_deja_temporales() {
    let dir = dir("sobre_dir");
    let target = dir.join("a.sav.zst");
    fs::create_dir(&target).unwrap();
    let err = Sim::new(Scenario::default(), 1)
        .save_to_file(&target, None)
        .unwrap_err();
    assert!(matches!(err, SaveFileError::Io { .. }), "{err}");
    assert_eq!(names(&dir), ["a.sav.zst"]);
    assert!(target.is_dir());
}

#[test]
fn ruta_sin_nombre_de_archivo_es_error() {
    let err = Sim::new(Scenario::default(), 1)
        .save_to_file("", None)
        .unwrap_err();
    assert!(matches!(err, SaveFileError::InvalidPath(_)), "{err}");
}

#[test]
fn cargar_archivo_inexistente_o_directorio_es_error() {
    let dir = dir("inexistente");
    let err = Sim::load_from_file(dir.join("no.sav.zst"), None).unwrap_err();
    assert!(matches!(err, SaveFileError::Io { .. }), "{err}");
    // Un directorio no se puede leer como archivo (en algunas plataformas falla al abrir, en otras al leer).
    let err = Sim::load_from_file(&dir, None).unwrap_err();
    assert!(matches!(err, SaveFileError::Io { .. }), "{err}");
}

#[test]
fn vacio_y_no_zstd() {
    let dir = dir("no_zstd");
    let path = dir.join("a.sav.zst");
    for bytes in [&b""[..], b"{\"format_version\":1}", b"\x28\xB5", &[0u8; 64]] {
        fs::write(&path, bytes).unwrap();
        let err = Sim::load_from_file(&path, None).unwrap_err();
        assert!(matches!(err, SaveFileError::NotZstd), "{bytes:?}: {err}");
    }
}

#[test]
fn truncado_en_cualquier_punto_es_error_sin_panico() {
    let dir = dir("truncado");
    let path = dir.join("a.sav.zst");
    let full_path = dir.join("full.sav.zst");
    year_with_marks(5).save_to_file(&full_path, None).unwrap();
    let full = fs::read(&full_path).unwrap();
    let step = (full.len() / 200).max(1);
    for cut in (0..full.len()).step_by(step).chain([full.len() - 1]) {
        fs::write(&path, &full[..cut]).unwrap();
        let result = Sim::load_from_file(&path, None);
        assert!(result.is_err(), "corte en {cut} de {} cargó", full.len());
    }
}

#[test]
fn bytes_corruptos_no_hacen_panico() {
    let dir = dir("corrupto");
    let path = dir.join("a.sav.zst");
    let full_path = dir.join("full.sav.zst");
    let sim = year_with_marks(5);
    sim.save_to_file(&full_path, None).unwrap();
    let full = fs::read(&full_path).unwrap();
    let step = (full.len() / 150).max(1);
    for index in (0..full.len()).step_by(step) {
        let mut bytes = full.clone();
        bytes[index] ^= 0xA5;
        fs::write(&path, &bytes).unwrap();
        // Puede decodificar a otro texto o fallar; si carga, el hash verificado coincide.
        if let Ok(loaded) = Sim::load_from_file(&path, None) {
            assert_eq!(loaded.state_hash(), sim.state_hash(), "byte {index}");
        }
    }
}

#[test]
fn datos_de_mas_tras_el_frame_es_error() {
    let dir = dir("sobrantes");
    let path = dir.join("a.sav.zst");
    Sim::new(Scenario::default(), 1)
        .save_to_file(&path, None)
        .unwrap();
    let mut bytes = fs::read(&path).unwrap();
    bytes.extend_from_slice(b"basura");
    fs::write(&path, &bytes).unwrap();
    let err = Sim::load_from_file(&path, None).unwrap_err();
    assert!(matches!(err, SaveFileError::Corrupt(_)), "{err}");
}

#[test]
fn zstd_valido_pero_no_es_un_guardado() {
    let dir = dir("no_guardado");
    let path = dir.join("a.sav.zst");
    for text in [&b"no es json"[..], b"{}", b"[1,2,3]", b"\xFF\xFE"] {
        fs::write(&path, zstd(text)).unwrap();
        let err = Sim::load_from_file(&path, None).unwrap_err();
        assert!(matches!(err, SaveFileError::Save(_)), "{text:?}: {err}");
    }
    fs::write(&path, zstd(b"{}")).unwrap();
    assert!(matches!(
        Sim::load_from_file(&path, None).unwrap_err(),
        SaveFileError::Save(SaveError::MissingFormatVersion)
    ));
}

#[test]
fn zip_bomb_se_corta_con_el_limite() {
    let dir = dir("bomba");
    let path = dir.join("a.sav.zst");
    // 64 MiB de ceros comprimen a unos pocos KiB.
    let bomb = zstd(&vec![0u8; 64 * 1024 * 1024]);
    assert!(bomb.len() < 1024 * 1024, "la bomba pesa {} B", bomb.len());
    fs::write(&path, &bomb).unwrap();
    let err = Sim::load_from_file_with_limit(&path, None, 1024 * 1024).unwrap_err();
    assert!(
        matches!(err, SaveFileError::TooLarge { limit } if limit == 1024 * 1024),
        "{err}"
    );
    // Con el límite por defecto cabe, pero no es JSON: error de contenido, sin pánico.
    assert!(matches!(
        Sim::load_from_file(&path, None).unwrap_err(),
        SaveFileError::Save(_)
    ));
}

#[test]
fn archivo_mas_grande_que_el_limite() {
    let dir = dir("grande");
    let path = dir.join("a.sav.zst");
    year_with_marks(2).save_to_file(&path, None).unwrap();
    let size = fs::metadata(&path).unwrap().len();
    let err = Sim::load_from_file_with_limit(&path, None, size - 1).unwrap_err();
    assert!(matches!(err, SaveFileError::TooLarge { .. }), "{err}");
    // El JSON descomprimido es mayor que el archivo: con límite = tamaño del archivo también falla.
    let err = Sim::load_from_file_with_limit(&path, None, size).unwrap_err();
    assert!(matches!(err, SaveFileError::TooLarge { .. }), "{err}");
    assert!(Sim::load_from_file(&path, None).is_ok());
}
