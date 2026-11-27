//! Guardado a archivo `.sav.zst`: el JSON de [`Sim::to_save_json`] comprimido con zstd y escrito
//! de forma **atómica** (issue #51, doc 21 §2).
//!
//! # Formato del archivo
//!
//! Extensión recomendada: `.sav.zst` ([`SAVE_FILE_EXTENSION`]); la API no la impone. El contenido
//! es **un único frame zstd** cuyo texto descomprimido es exactamente el JSON de
//! [`Sim::to_save_json`] (UTF-8). Cualquier `zstd -d` lo abre.
//!
//! # Compresión determinista
//!
//! Se usa [`ruzstd`] (Rust puro) con un nivel **fijo**: `CompressionLevel::Fastest` (equivalente
//! a zstd nivel 1). Sin parámetros, sin tiempo ni azar: el mismo guardado produce siempre los
//! mismos bytes **con la misma versión de `ruzstd`** (fijada por `Cargo.lock`); actualizar la
//! dependencia puede cambiar los bytes comprimidos, nunca el JSON descomprimido ni el
//! `state_hash`. Por eso la identidad de un guardado se compara por su JSON/hash, no por los
//! bytes del archivo comprimido.
//!
//! # Escritura atómica
//!
//! [`Sim::save_to_file`] escribe a un temporal **en el mismo directorio** (mismo volumen, requisito
//! para que el renombrado sea atómico), hace `sync_all` y renombra sobre el destino. El destino es,
//! en todo momento, el guardado viejo completo o el nuevo completo: un fallo o un corte a mitad de
//! escritura deja intacto el anterior. Si algo falla, el temporal se borra (mejor esfuerzo).
//!
//! - Nombre del temporal: `.<archivo>.<pid>.<n>.tmp`, con `pid` el id del proceso y `n` un
//!   contador atómico del proceso; se crea con `create_new`, así que dos guardados concurrentes
//!   (en hilos o procesos) nunca comparten temporal. Es determinista (sin tiempo ni azar). Si el
//!   proceso muere a mitad de escritura queda ese temporal huérfano (sin riesgo para el guardado);
//!   se puede borrar a mano.
//! - Windows: `std::fs::rename` usa `MoveFileExW` con `MOVEFILE_REPLACE_EXISTING`, así que
//!   reemplaza un destino existente (lo cubre un test que sobrescribe). En Unix, `rename(2)`
//!   también reemplaza; además se hace `fsync` del directorio para persistir el renombrado.
//! - Dos guardados simultáneos al **mismo** destino no se corrompen, pero gana el último en
//!   renombrar; coordinar quién guarda es del llamador.
//!
//! # Carga y límites
//!
//! [`Sim::load_from_file`] nunca entra en pánico: archivo inexistente o ilegible
//! ([`SaveFileError::Io`]), vacío o que no es zstd ([`SaveFileError::NotZstd`]), truncado o
//! corrupto ([`SaveFileError::Corrupt`]), demasiado grande ([`SaveFileError::TooLarge`]) o con
//! JSON inválido ([`SaveFileError::Save`]). Contra *zip-bombs* hay un límite en bytes tanto del
//! archivo como del texto descomprimido ([`DEFAULT_MAX_SAVE_BYTES`], configurable con
//! [`Sim::load_from_file_with_limit`]); la descompresión se corta al pasarlo y la ventana del
//! decodificador está acotada por el mismo límite.

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use ruzstd::decoding::{DEFAULT_MAX_WINDOW_SIZE, StreamingDecoder};
use ruzstd::encoding::{CompressionLevel, compress_to_vec};
use thiserror::Error;

use super::{SaveError, Sim};
use crate::data::GameData;

/// Extensión recomendada de los archivos de guardado (sin el punto inicial).
pub const SAVE_FILE_EXTENSION: &str = "sav.zst";

/// Límite por defecto (bytes) del tamaño del archivo y del JSON descomprimido al cargar: 256 MiB
/// (un año simulado pesa del orden de cientos de KiB; ver el test de tamaños).
pub const DEFAULT_MAX_SAVE_BYTES: u64 = 256 * 1024 * 1024;

/// Nivel de compresión fijo (determinismo): ver «Compresión determinista» en el módulo.
const LEVEL: CompressionLevel = CompressionLevel::Fastest;

/// Firma de un frame zstd (little-endian de `0xFD2FB528`).
const ZSTD_MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];

/// Error al guardar o cargar un `Sim` desde un archivo. Es `#[non_exhaustive]`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SaveFileError {
    /// Falló una operación de disco (archivo inexistente, sin permisos, directorio inexistente...).
    #[error("{operation} {}: {source}", path.display())]
    Io {
        /// Qué se intentaba (`"crear el temporal"`, `"abrir"`, `"renombrar"`...).
        operation: &'static str,
        /// Ruta involucrada.
        path: PathBuf,
        /// Error original.
        #[source]
        source: io::Error,
    },
    /// La ruta de destino no nombra un archivo (vacía, termina en `..`, etc.).
    #[error("la ruta {} no nombra un archivo", .0.display())]
    InvalidPath(PathBuf),
    /// El archivo está vacío o no empieza con la firma de zstd.
    #[error("el archivo no es zstd (vacío o sin la firma de un frame)")]
    NotZstd,
    /// El frame zstd está corrupto, truncado o tiene datos de más.
    #[error("archivo zstd corrupto o truncado: {0}")]
    Corrupt(String),
    /// El archivo o su contenido descomprimido supera el límite configurado.
    #[error("el guardado supera el límite de {limit} bytes")]
    TooLarge {
        /// Límite aplicado, en bytes.
        limit: u64,
    },
    /// Se descomprimió bien, pero el contenido no es un guardado válido.
    #[error(transparent)]
    Save(#[from] SaveError),
}

fn io_err(operation: &'static str, path: &Path) -> impl FnOnce(io::Error) -> SaveFileError {
    let path = path.to_path_buf();
    move |source| SaveFileError::Io {
        operation,
        path,
        source,
    }
}

/// Puntos de la escritura atómica en los que un test puede inyectar un fallo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WriteStage {
    /// Con el temporal creado y vacío.
    BeforeWrite,
    /// Con la primera mitad escrita.
    MidWrite,
    /// Con todo escrito, antes de `sync_all`.
    BeforeSync,
    /// Con todo escrito y sincronizado, antes de renombrar.
    BeforeRename,
}

/// Gancho de inyección de fallos. En producción es `()` y no hace nada.
pub(crate) trait WriteHook {
    fn at(&mut self, stage: WriteStage) -> io::Result<()>;
}

impl WriteHook for () {
    fn at(&mut self, _stage: WriteStage) -> io::Result<()> {
        Ok(())
    }
}

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_path(target: &Path) -> Result<PathBuf, SaveFileError> {
    let name = target
        .file_name()
        .ok_or_else(|| SaveFileError::InvalidPath(target.to_path_buf()))?;
    let mut tmp = OsString::from(".");
    tmp.push(name);
    let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    tmp.push(format!(".{}.{n}.tmp", std::process::id()));
    let dir = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    Ok(dir.join(tmp))
}

/// Escribe `bytes` en `target` de forma atómica (temporal + `sync_all` + renombrado). Si falla en
/// cualquier punto, borra el temporal y deja `target` como estaba.
pub(crate) fn write_atomic<H: WriteHook>(
    target: &Path,
    bytes: &[u8],
    hook: &mut H,
) -> Result<(), SaveFileError> {
    let tmp = temp_path(target)?;
    let result = write_and_rename(target, &tmp, bytes, hook);
    if result.is_err() {
        // Mejor esfuerzo: si el temporal no llegó a crearse, no hay nada que borrar.
        let _ = fs::remove_file(&tmp);
    }
    result
}

fn write_and_rename<H: WriteHook>(
    target: &Path,
    tmp: &Path,
    bytes: &[u8],
    hook: &mut H,
) -> Result<(), SaveFileError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(tmp)
        .map_err(io_err("crear el temporal", tmp))?;
    hook.at(WriteStage::BeforeWrite)
        .map_err(io_err("escribir", tmp))?;
    let (first, second) = bytes.split_at(bytes.len() / 2);
    file.write_all(first).map_err(io_err("escribir", tmp))?;
    hook.at(WriteStage::MidWrite)
        .map_err(io_err("escribir", tmp))?;
    file.write_all(second).map_err(io_err("escribir", tmp))?;
    hook.at(WriteStage::BeforeSync)
        .map_err(io_err("sincronizar", tmp))?;
    file.sync_all().map_err(io_err("sincronizar", tmp))?;
    // Se cierra antes de renombrar (Windows no renombra un archivo abierto sin permiso de compartir).
    drop(file);
    hook.at(WriteStage::BeforeRename)
        .map_err(io_err("renombrar", target))?;
    fs::rename(tmp, target).map_err(io_err("renombrar", target))?;
    sync_parent_dir(target);
    Ok(())
}

/// Persiste el renombrado (`fsync` del directorio). Sólo Unix; mejor esfuerzo: el guardado ya está
/// completo en su destino, un fallo acá no lo invalida.
#[cfg(unix)]
fn sync_parent_dir(target: &Path) {
    let dir = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if let Ok(dir) = File::open(dir) {
        let _ = dir.sync_all();
    }
}

#[cfg(not(unix))]
fn sync_parent_dir(_target: &Path) {}

/// Comprime el JSON (nivel fijo, determinista).
fn compress_json(json: &str) -> Vec<u8> {
    compress_to_vec(json.as_bytes(), LEVEL)
}

/// Descomprime `bytes` (un único frame) a texto, con `limit` como tope del resultado.
fn decompress_json(bytes: &[u8], limit: u64) -> Result<String, SaveFileError> {
    if bytes.len() as u64 > limit {
        return Err(SaveFileError::TooLarge { limit });
    }
    if !bytes.starts_with(&ZSTD_MAGIC) {
        return Err(SaveFileError::NotZstd);
    }
    // El encoder escribe ventanas de 128 KiB: piso de 1 MiB para que un límite chico dé `TooLarge`
    // y no un error de ventana; techo del decodificador por defecto (128 MiB).
    let window = limit.clamp(1 << 20, DEFAULT_MAX_WINDOW_SIZE);
    let mut decoder = StreamingDecoder::new_with_max_window_size(bytes, window)
        .map_err(|error| SaveFileError::Corrupt(error.to_string()))?;
    let mut out = Vec::new();
    // `take(limit + 1)`: si llega a `limit + 1` bytes, se pasó (y se corta la descompresión).
    let read = (&mut decoder)
        .take(limit.saturating_add(1))
        .read_to_end(&mut out)
        .map_err(|error| SaveFileError::Corrupt(error.to_string()))?;
    if read as u64 > limit {
        return Err(SaveFileError::TooLarge { limit });
    }
    let (rest, frame) = decoder.into_parts();
    // Un frame truncado puede terminar el flujo sin error: hay que comprobar que llegó al final.
    if !frame.is_finished() {
        return Err(SaveFileError::Corrupt("el frame está truncado".into()));
    }
    if !rest.is_empty() {
        return Err(SaveFileError::Corrupt(format!(
            "{} byte(s) de más tras el frame",
            rest.len()
        )));
    }
    String::from_utf8(out)
        .map_err(|_| SaveFileError::Save(SaveError::Malformed("el texto no es UTF-8".into())))
}

impl Sim {
    /// Guarda el estado en `path` como JSON comprimido con zstd, de forma atómica (ver el módulo).
    /// Reemplaza el archivo existente. Extensión recomendada: `.sav.zst`.
    ///
    /// # Errors
    /// Los de [`Sim::to_save_json`] ([`SaveFileError::Save`]); [`SaveFileError::InvalidPath`] si
    /// `path` no nombra un archivo; [`SaveFileError::Io`] si falla el disco (directorio
    /// inexistente, sin permisos, disco lleno...). Ante cualquier error el destino queda como
    /// estaba y no queda ningún temporal.
    pub fn save_to_file(
        &self,
        path: impl AsRef<Path>,
        data: Option<&GameData>,
    ) -> Result<(), SaveFileError> {
        let json = self.to_save_json(data)?;
        write_atomic(path.as_ref(), &compress_json(&json), &mut ())
    }

    /// Carga un `Sim` de un archivo escrito por [`Sim::save_to_file`], con el límite
    /// [`DEFAULT_MAX_SAVE_BYTES`].
    ///
    /// # Errors
    /// Ver [`Sim::load_from_file_with_limit`].
    pub fn load_from_file(
        path: impl AsRef<Path>,
        data: Option<&GameData>,
    ) -> Result<Self, SaveFileError> {
        Self::load_from_file_with_limit(path, data, DEFAULT_MAX_SAVE_BYTES)
    }

    /// Como [`Sim::load_from_file`] con un límite explícito (bytes) para el tamaño del archivo y
    /// del JSON descomprimido.
    ///
    /// # Errors
    /// [`SaveFileError::Io`] (inexistente, ilegible), [`SaveFileError::NotZstd`],
    /// [`SaveFileError::Corrupt`], [`SaveFileError::TooLarge`] y, del contenido,
    /// [`SaveFileError::Save`] (JSON inválido, versión, huella, invariantes; ver [`SaveError`]).
    pub fn load_from_file_with_limit(
        path: impl AsRef<Path>,
        data: Option<&GameData>,
        max_bytes: u64,
    ) -> Result<Self, SaveFileError> {
        let path = path.as_ref();
        let file = File::open(path).map_err(io_err("abrir", path))?;
        let mut bytes = Vec::new();
        // Se lee a lo sumo `max_bytes + 1`: un archivo gigante no se carga entero a memoria.
        file.take(max_bytes.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(io_err("leer", path))?;
        let json = decompress_json(&bytes, max_bytes)?;
        Ok(Self::from_save_json(&json, data)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Increment, Scenario};

    /// Falla en una etapa dada.
    struct FailAt(WriteStage);

    impl WriteHook for FailAt {
        fn at(&mut self, stage: WriteStage) -> io::Result<()> {
            if stage == self.0 {
                Err(io::Error::other("fallo simulado"))
            } else {
                Ok(())
            }
        }
    }

    fn dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "heliarca_save_file_unit_{}_{name}",
            std::process::id()
        ));
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

    fn sim_at(days: u32) -> Sim {
        let mut sim = Sim::new(Scenario::default(), 5).unwrap();
        for _ in 0..days {
            sim.advance(Increment::D1).unwrap();
        }
        sim
    }

    #[test]
    fn fallo_en_cada_etapa_deja_el_guardado_previo_y_ningun_temporal() {
        let old = sim_at(3);
        let new = sim_at(9);
        for stage in [
            WriteStage::BeforeWrite,
            WriteStage::MidWrite,
            WriteStage::BeforeSync,
            WriteStage::BeforeRename,
        ] {
            let dir = dir("etapas");
            let path = dir.join("a.sav.zst");
            old.save_to_file(&path, None).unwrap();
            let before = fs::read(&path).unwrap();

            let bytes = compress_json(&new.to_save_json(None).unwrap());
            let err = write_atomic(&path, &bytes, &mut FailAt(stage)).unwrap_err();
            assert!(matches!(err, SaveFileError::Io { .. }), "{stage:?}: {err}");

            assert_eq!(fs::read(&path).unwrap(), before, "{stage:?}");
            assert_eq!(names(&dir), ["a.sav.zst"], "{stage:?}: quedó un temporal");
            let loaded = Sim::load_from_file(&path, None).unwrap();
            assert_eq!(loaded.state_hash(), old.state_hash(), "{stage:?}");
        }
    }

    #[test]
    fn fallo_sin_guardado_previo_no_crea_el_destino() {
        let dir = dir("sin_previo");
        let path = dir.join("a.sav.zst");
        let bytes = compress_json(&sim_at(2).to_save_json(None).unwrap());
        write_atomic(&path, &bytes, &mut FailAt(WriteStage::MidWrite)).unwrap_err();
        assert_eq!(names(&dir), Vec::<String>::new());
    }

    #[test]
    fn los_temporales_no_colisionan() {
        let path = Path::new("x/a.sav.zst");
        let a = temp_path(path).unwrap();
        let b = temp_path(path).unwrap();
        assert_ne!(a, b);
        assert_eq!(a.parent(), path.parent());
    }

    #[test]
    fn compresion_determinista() {
        let json = sim_at(30).to_save_json(None).unwrap();
        assert_eq!(compress_json(&json), compress_json(&json));
    }
}
