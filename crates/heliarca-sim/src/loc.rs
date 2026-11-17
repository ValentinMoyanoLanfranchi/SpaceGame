//! Localización por claves (doc 21 §2, doc 22 §11).
//!
//! El código **nunca** contiene texto de cara al jugador: sólo claves estables
//! (`command.error.invalid_target`) que se resuelven contra `data/loc/es.json` con [`Loc::get`].
//!
//! # Formato de las claves
//!
//! `dominio.sub.clave`: al menos dos segmentos separados por `.`, cada uno de minúsculas ASCII,
//! dígitos y `_`, empezando por una letra ([`is_valid_key`]). Los errores de comando usan el
//! dominio `command.error.*` ([`CommandError::loc_key`]).
//!
//! # Fallback visible
//!
//! Una clave que falta **no** rompe la ejecución ni es un `panic`: [`Loc::get`] devuelve
//! `¡clave!`, de modo que el hueco se ve en pantalla y es fácil de reportar.
//!
//! # Sin interpolación (todavía)
//!
//! Los textos son fijos: ningún error lleva hoy datos de contexto que el jugador deba leer en el
//! mensaje. Cuando haga falta (`{nombre}`) se agrega aquí sin cambiar las claves.
//!
//! # Cómo se mantiene sincronizado con el código
//!
//! Los tests de este módulo fallan si el código usa una clave que `data/loc/es.json` no define:
//! (1) construyen **todas** las variantes de [`CommandError`] (una `match` exhaustiva obliga a
//! actualizarlos al agregar una) y exigen su `loc_key()` en `es.json`; (2) escanean el fuente de
//! `src/` en busca de literales con forma de clave de localización (dominios de [`LOC_DOMAINS`] o
//! argumento literal de `.get(`/`Loc::get(`) y exigen que estén definidos. Las claves definidas
//! que nadie usa sólo se informan (la salida del test), no fallan.

use std::borrow::Cow;
use std::path::Path;

use crate::command::CommandError;
use crate::data::{DataLoadErrors, DataLoader, GameData, Localization};

/// Textos localizados (español): clave -> texto. Determinista (`BTreeMap`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Loc {
    texts: Localization,
}

impl Loc {
    /// Arma la localización desde un mapa clave -> texto.
    #[must_use]
    pub fn from_texts(texts: Localization) -> Self {
        Self { texts }
    }

    /// Toma los textos de `loc/es.json` ya cargados en `data`.
    #[must_use]
    pub fn from_game_data(data: &GameData) -> Self {
        Self::from_texts(data.loc_es.clone())
    }

    /// Carga `loc/es.json` (y el resto de `data/`) desde el directorio `dir`.
    ///
    /// # Errors
    ///
    /// Los [`DataLoadErrors`] de [`DataLoader::load_dir`].
    pub fn load(dir: impl AsRef<Path>) -> Result<Self, DataLoadErrors> {
        DataLoader::load_dir(dir).map(|data| Self::from_texts(data.loc_es))
    }

    /// Texto de `key`; si no está definida, el fallback visible `¡key!` (no falla nunca).
    #[must_use]
    pub fn get(&self, key: &str) -> Cow<'_, str> {
        match self.texts.get(key) {
            Some(text) => Cow::Borrowed(text.as_str()),
            None => Cow::Owned(format!("¡{key}!")),
        }
    }

    /// Texto para el jugador de un error de comando: `get(err.loc_key())`.
    #[must_use]
    pub fn command_error(&self, error: &CommandError) -> Cow<'_, str> {
        self.get(error.loc_key())
    }

    /// `true` si `key` está definida.
    #[must_use]
    pub fn contains(&self, key: &str) -> bool {
        self.texts.contains_key(key)
    }

    /// Cantidad de claves definidas.
    #[must_use]
    pub fn len(&self) -> usize {
        self.texts.len()
    }

    /// `true` si no hay ninguna clave.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.texts.is_empty()
    }

    /// Las claves definidas, en orden alfabético.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.texts.keys().map(String::as_str)
    }
}

/// Dominios (primer segmento) de las claves de localización que el código puede usar.
/// Agregar uno nuevo aquí lo incluye en el escaneo de claves usadas.
pub const LOC_DOMAINS: [&str; 1] = ["command"];

/// `true` si `key` cumple el formato `dominio.sub.clave` (ver el módulo).
#[must_use]
pub fn is_valid_key(key: &str) -> bool {
    let mut segments = 0;
    for segment in key.split('.') {
        let mut bytes = segment.bytes();
        let Some(first) = bytes.next() else {
            return false;
        };
        if !first.is_ascii_lowercase()
            || !bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return false;
        }
        segments += 1;
    }
    segments >= 2
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;

    use super::*;
    use crate::events::EventError;

    fn repo_data() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
    }

    fn real_loc() -> Loc {
        Loc::load(repo_data()).expect("data/ debe cargar")
    }

    fn loc_of(pairs: &[(&str, &str)]) -> Loc {
        Loc::from_texts(
            pairs
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
        )
    }

    /// Todas las claves que `CommandError` puede devolver. Si se agrega una variante, la `match`
    /// deja de compilar y obliga a actualizar la lista.
    fn command_error_keys() -> Vec<&'static str> {
        let all = [
            CommandError::InvalidTarget { target: "x".into() },
            CommandError::InvalidArgument { argument: "x" },
            CommandError::NotAllowedNow { reason: "x" },
            CommandError::EventLog(EventError::LogFull),
            CommandError::IdsExhausted,
        ];
        for e in &all {
            match e {
                CommandError::InvalidTarget { .. }
                | CommandError::InvalidArgument { .. }
                | CommandError::NotAllowedNow { .. }
                | CommandError::EventLog(_)
                | CommandError::IdsExhausted => {}
            }
        }
        all.iter().map(CommandError::loc_key).collect()
    }

    /// Literales de cadena (sin escapes) del fuente `src`.
    fn string_literals(src: &str) -> Vec<(usize, String)> {
        let mut out = Vec::new();
        for (n, line) in src.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue; // comentarios y doc-comments: ejemplos, no uso real
            }
            let mut parts = line.split('"');
            parts.next();
            for (i, part) in parts.enumerate() {
                if i % 2 == 0 {
                    out.push((n + 1, part.to_owned()));
                }
            }
        }
        out
    }

    /// Claves "usadas" por el código: literales con forma de clave en un dominio de `LOC_DOMAINS`
    /// o que son argumento literal de `.get(` / `Loc::get(`; se excluye este archivo.
    fn used_keys_in_source(files: &[(String, String)]) -> BTreeSet<(String, usize, String)> {
        let mut used = BTreeSet::new();
        for (name, src) in files {
            for (line, lit) in string_literals(src) {
                let domain = lit.split('.').next().unwrap_or("");
                let in_domain = LOC_DOMAINS.contains(&domain) && is_valid_key(&lit);
                let in_get = src.lines().nth(line - 1).is_some_and(|l| {
                    ["loc.get", "Loc::get", "loc().get"]
                        .iter()
                        .any(|p| l.contains(&format!("{p}(\"{lit}\")")))
                }) && is_valid_key(&lit);
                if in_domain || in_get {
                    used.insert((name.clone(), line, lit));
                }
            }
        }
        used
    }

    fn source_files() -> Vec<(String, String)> {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        for entry in fs::read_dir(&dir).expect("leer src") {
            let path = entry.expect("entrada").path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if path.extension().is_some_and(|e| e == "rs") && name != "loc.rs" {
                files.push((name.to_owned(), fs::read_to_string(&path).expect("leer")));
            }
        }
        files.sort();
        files
    }

    fn missing_used_keys(loc: &Loc, files: &[(String, String)]) -> Vec<String> {
        let mut missing: Vec<String> = used_keys_in_source(files)
            .into_iter()
            .filter(|(_, _, key)| !loc.contains(key))
            .map(|(file, line, key)| {
                format!("{file}:{line}: clave `{key}` sin definir en loc/es.json")
            })
            .collect();
        for key in command_error_keys() {
            if !loc.contains(key) {
                missing.push(format!(
                    "CommandError::loc_key: clave `{key}` sin definir en loc/es.json"
                ));
            }
        }
        missing
    }

    #[test]
    fn una_clave_faltante_se_muestra_marcada_y_no_rompe() {
        let loc = loc_of(&[("a.b", "texto")]);
        assert_eq!(loc.get("no.existe"), "¡no.existe!");
        assert_eq!(loc.get("a.b"), "texto");
        assert!(matches!(loc.get("a.b"), Cow::Borrowed(_)));
        assert!(!loc.contains("no.existe") && loc.contains("a.b"));
        assert_eq!(Loc::default().get("x.y"), "¡x.y!");
        assert!(Loc::default().is_empty());
        assert_eq!(loc.len(), 1);
    }

    #[test]
    fn las_claves_se_listan_en_orden() {
        let loc = loc_of(&[("b.x", "1"), ("a.x", "2")]);
        assert_eq!(loc.keys().collect::<Vec<_>>(), ["a.x", "b.x"]);
    }

    #[test]
    fn el_es_json_real_define_todas_las_claves_usadas() {
        let missing = missing_used_keys(&real_loc(), &source_files());
        assert!(
            missing.is_empty(),
            "claves usadas y no definidas:\n{}",
            missing.join("\n")
        );
    }

    #[test]
    fn el_test_detecta_una_clave_usada_inexistente() {
        // El escaneo debe fallar si el código usa una clave no definida.
        let files = vec![(
            "falso.rs".to_owned(),
            "fn f() -> &'static str { \"command.error.inventada\" }\nfn g(loc: &Loc) { loc.get(\"ui.otra\"); }\n"
                .to_owned(),
        )];
        let missing = missing_used_keys(&real_loc(), &files);
        assert_eq!(missing.len(), 2, "{missing:?}");
        assert!(missing[0].contains("command.error.inventada"));
        assert!(missing[1].contains("ui.otra"));
        // Y una clave de CommandError sin texto también se detecta.
        let vacio = missing_used_keys(&Loc::default(), &[]);
        assert_eq!(vacio.len(), command_error_keys().len());
    }

    #[test]
    fn el_escaneo_ve_las_claves_reales_del_codigo() {
        let used = used_keys_in_source(&source_files());
        assert!(
            used.iter()
                .any(|(f, _, k)| f == "command.rs" && k == "command.error.event_log")
        );
    }

    #[test]
    fn claves_y_textos_del_es_json_son_validos() {
        let loc = real_loc();
        assert!(!loc.is_empty());
        for key in loc.keys() {
            assert!(is_valid_key(key), "formato de clave inválido: {key}");
            assert!(!loc.get(key).trim().is_empty(), "texto vacío: {key}");
            assert!(!loc.get(key).starts_with('¡'), "{key}");
        }
    }

    #[test]
    fn claves_definidas_que_nadie_usa_se_informan_sin_fallar() {
        let loc = real_loc();
        let used: BTreeSet<String> = used_keys_in_source(&source_files())
            .into_iter()
            .map(|(_, _, k)| k)
            .chain(command_error_keys().into_iter().map(str::to_owned))
            .collect();
        for key in loc.keys().filter(|k| !used.contains(*k)) {
            eprintln!("aviso: la clave `{key}` está definida en es.json pero el código no la usa");
        }
    }

    #[test]
    fn formato_de_clave() {
        for ok in ["a.b", "command.error.invalid_target", "ui.panel2.titulo_1"] {
            assert!(is_valid_key(ok), "{ok}");
        }
        for bad in [
            "", "a", "a.", ".a", "a..b", "A.b", "a.B", "a b.c", "a.1b", "1a.b", "á.b", "a-b.c",
        ] {
            assert!(!is_valid_key(bad), "{bad}");
        }
    }

    #[test]
    fn cada_error_de_comando_tiene_texto_localizado() {
        let loc = real_loc();
        let err = CommandError::IdsExhausted;
        assert_eq!(
            loc.command_error(&err),
            loc.get("command.error.ids_exhausted")
        );
        assert!(!loc.command_error(&err).starts_with('¡'));
    }

    #[test]
    fn load_informa_los_errores_de_datos() {
        assert!(Loc::load(repo_data().join("no-existe")).is_err());
    }

    #[test]
    fn from_game_data_toma_loc_es() {
        let mut data = GameData::default();
        data.loc_es.insert("a.b".into(), "t".into());
        assert_eq!(Loc::from_game_data(&data).get("a.b"), "t");
    }
}
