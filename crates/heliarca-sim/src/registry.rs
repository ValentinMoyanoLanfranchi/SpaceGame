//! Registro de ids de los datos y validador de referencias cruzadas (#32, doc 22 §0 y §11).
//!
//! Se usa desde [`crate::data`] (que reexporta [`Registry`], [`RegistryError`], [`RegistryErrors`],
//! [`ValidatedData`] y [`ValidatedLoadError`]); el módulo en sí es interno.
//!
//! # Qué hace
//!
//! [`Registry::build`] (o [`GameData::build_registry`]) recorre un [`GameData`] ya cargado y:
//!
//! 1. **Registra los ids** de cada archivo en un [`IdRegistry`] tipado ([`BodyId`], [`GoodId`],
//!    [`TechId`], [`DesignId`], [`RecipeId`], [`FacilityId`], [`CompanyId`], [`SocialEventId`]),
//!    validando el formato `tipo.nombre` con las reglas de [`crate::ids`] (no se duplican aquí).
//! 2. **Verifica las referencias cruzadas** una vez que todos los ids están registrados (la
//!    referencia puede apuntar a una entrada posterior del archivo o a otro archivo).
//! 3. **Verifica que `parent` no forme ciclos**: la jerarquía de cuerpos es un árbol.
//!
//! Devuelve **todos** los errores de una corrida ([`RegistryErrors`]), nunca sólo el primero ni un
//! `panic`. El orden es estable: primero los errores de ids (archivo por archivo, en el orden de
//! [`DATA_FILES`](crate::data::DATA_FILES), entrada por entrada), luego los de referencias (mismo
//! orden, campo por campo) y al final los ciclos.
//!
//! # Qué ids existen
//!
//! | Archivo | Prefijo | Id tipado |
//! |---|---|---|
//! | `bodies.json` | `body` o `jp` (punto de salto, ver abajo) | [`BodyId`] |
//! | `goods.json` | `good` | [`GoodId`] |
//! | `recipes.json` | `recipe` | [`RecipeId`] |
//! | `facilities.json` | `facility` | [`FacilityId`] |
//! | `techs.json` | `tech` | [`TechId`] |
//! | `designs.json` | `design` | [`DesignId`] |
//! | `companies.json` | `company` | [`CompanyId`] |
//! | `events.json` | `event` | [`SocialEventId`] |
//!
//! El doc 22 §0 sólo nombra `body.`, `good.`, `tech.` y `design.`; los prefijos de recetas,
//! instalaciones, empresas y eventos son una elección de esta tarea (el tipo de la entrada, en
//! singular y en inglés como los demás) y se cambian en un solo lugar si el doc fija otros.
//! `scenario_earth.json` (objeto libre) y `loc/es.json` (claves de texto, #35) no tienen ids de
//! entrada y no se validan aquí.
//!
//! # Excepción: el punto de salto (`jp.*`)
//!
//! Doc 22 §11 dice que `bodies.json` trae «21 cuerpos + punto de salto + yacimientos» y §1.2 nombra
//! al punto de salto `jp.alfa`. Por eso `bodies.json` acepta los prefijos `body` **y** `jp`
//! ([`BodyRegistry`]): ambos son cuerpos, comparten la secuencia de [`BodyId`] y el texto conserva
//! su prefijo (`jp.alfa` <-> id <-> `jp.alfa`). `body.alfa` y `jp.alfa` son ids distintos (no son
//! duplicados). Un `jp.*` en cualquier otro archivo es un error de prefijo, y un nombre inválido
//! bajo `jp` es `InvalidId` como siempre.
//!
//! # Asignación de ids
//!
//! Los ids numéricos se asignan **en orden de aparición** en el archivo (0, 1, 2, …), por tipo.
//! Las entradas con id inválido o repetido no consumen número. Es determinista: los mismos datos dan
//! siempre los mismos ids, y `texto -> id -> texto` es ida y vuelta estable. Un mismo texto no puede
//! estar en dos archivos: cada archivo exige su prefijo, así que el segundo aparecería como
//! [`RegistryError::InvalidId`] (prefijo incorrecto).
//!
//! # Referencias que se validan
//!
//! Sólo las que el doc 22 define con nombre de campo:
//!
//! | Origen | Campo | Debe ser |
//! |---|---|---|
//! | `bodies.json` | `parent` | un `body.*` o `jp.*` existente (y sin ciclos) |
//! | `bodies.json` | `deposits[].good` | un `good.*` existente |
//! | `techs.json` | `prereqs[]` (§6.1) | un `tech.*` existente |
//! | `techs.json` | `unlocks.designs[]` (§6.1) | un `design.*` existente |
//!
//! Los demás archivos usan [`Entry`](crate::data::Entry) con campos libres y el doc no fija sus
//! campos (las recetas de §2.1 son una tabla, sin nombres de campo: no hay `good` ni `inputs`
//! definidos). Sus referencias (`good` de recetas, instalaciones de una receta, `name_key` contra
//! `loc/es.json`, ciclos de `prereqs`...) quedan como extensión pendiente de #33 / los datos reales.
//!
//! # Ejemplo
//!
//! ```
//! use heliarca_sim::data::{Body, Entry, GameData, Registry, RegistryError};
//! use heliarca_sim::ids::BodyId;
//!
//! fn cuerpo(id: &str, parent: Option<&str>) -> Body {
//!     Body {
//!         id: id.to_owned(),
//!         kind: "moon".to_owned(),
//!         parent: parent.map(str::to_owned),
//!         orbit: None,
//!         radius_km: 1737.0,
//!         detect_difficulty: 0,
//!         habitability: None,
//!         deposits: Vec::new(),
//!     }
//! }
//!
//! let mut data = GameData::default();
//! data.bodies = vec![cuerpo("body.tierra", None), cuerpo("body.luna", Some("body.tierra"))];
//! let registry = Registry::build(&data)?;
//! let luna: BodyId = registry.bodies().from_text("body.luna")?;
//! assert_eq!(registry.bodies().to_text(luna).as_deref(), Some("body.luna"));
//!
//! // Un padre inexistente es un error tipado, con archivo, origen, campo y valor.
//! data.bodies = vec![cuerpo("body.luna", Some("body.tierra"))];
//! let errors = Registry::build(&data).unwrap_err();
//! assert!(matches!(errors.errors()[0], RegistryError::UnknownReference { .. }));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

use serde_json::Value;
use thiserror::Error;

use crate::data::{DATA_FILES, DataLoadErrors, DataLoader, Entry, GameData};
use crate::define_id;
use crate::ids::{
    BodyId, DesignId, EntityId, GoodId, IdRegistry, IdTextError, TechId, parse_text, validate_name,
};

/// Prefijo de los puntos de salto, que viven en `bodies.json` (doc 22 §1.2 y §11).
pub const JUMP_POINT_PREFIX: &str = "jp";

/// Texto esperado en los errores de referencia a un cuerpo (admite dos prefijos).
const BODY_REF_LABEL: &str = "body|jp";

/// Tabla texto <-> [`BodyId`] de `bodies.json`.
///
/// Es una excepción derivada del doc 22: §11 dice que `bodies.json` trae «21 cuerpos + punto de
/// salto + yacimientos» y §1.2 llama al punto de salto `jp.alfa`. Por eso aquí se aceptan **dos**
/// prefijos, `body.*` y `jp.*`, y ambos son cuerpos: reciben [`BodyId`] de la misma secuencia
/// (orden de aparición, sin importar el prefijo), pueden ser `parent` de otro cuerpo y participan
/// en la detección de ciclos. Se guarda el texto completo (`jp.alfa`), así que el texto vuelve
/// con su prefijo original; `body.alfa` y `jp.alfa` son ids distintos (no son duplicados).
/// Fuera de `bodies.json` el prefijo `jp` sigue siendo inválido.
#[derive(Debug, Clone, Default)]
pub struct BodyRegistry {
    texts: Vec<String>,
    by_text: BTreeMap<String, BodyId>,
}

impl BodyRegistry {
    /// Cantidad de cuerpos registrados.
    #[must_use]
    pub fn len(&self) -> usize {
        self.texts.len()
    }

    /// Indica si no hay cuerpos registrados.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.texts.is_empty()
    }

    /// Id de un texto registrado (`body.luna`, `jp.alfa`).
    #[must_use]
    pub fn get(&self, text: &str) -> Option<BodyId> {
        self.by_text.get(text).copied()
    }

    /// Texto estable de un id, con su prefijo original.
    #[must_use]
    pub fn to_text(&self, id: BodyId) -> Option<String> {
        let index = usize::try_from(id.raw()).ok()?;
        self.texts.get(index).cloned()
    }

    /// Resuelve un texto `body.*` o `jp.*` a su id.
    ///
    /// # Errors
    ///
    /// Texto mal formado, prefijo que no es `body` ni `jp`, o [`IdTextError::Unknown`].
    pub fn from_text(&self, text: &str) -> Result<BodyId, IdTextError> {
        validate_body_text(text)?;
        self.get(text).ok_or(IdTextError::Unknown)
    }

    /// Recorre `(id, texto)` en orden de registro.
    pub fn iter(&self) -> impl Iterator<Item = (BodyId, &str)> + '_ {
        self.texts
            .iter()
            .enumerate()
            .filter_map(|(i, t)| Some((BodyId::from_raw(u32::try_from(i).ok()?), t.as_str())))
    }

    fn register(&mut self, text: &str) -> Result<BodyId, IdTextError> {
        validate_body_text(text)?;
        if self.by_text.contains_key(text) {
            return Err(IdTextError::Duplicate);
        }
        let raw = u32::try_from(self.texts.len()).map_err(|_| IdTextError::RegistryFull)?;
        let id = BodyId::from_raw(raw);
        self.texts.push(text.to_owned());
        self.by_text.insert(text.to_owned(), id);
        Ok(id)
    }
}

/// Valida `body.<nombre>` o `jp.<nombre>` con las reglas de [`crate::ids`].
fn validate_body_text(text: &str) -> Result<(), IdTextError> {
    match text.split_once('.') {
        Some((JUMP_POINT_PREFIX, name)) => validate_name(name),
        _ => parse_text::<BodyId>(text).map(|_| ()),
    }
}

fn register_bodies<'a>(
    file: &'static str,
    ids: impl Iterator<Item = &'a str>,
    errors: &mut Vec<RegistryError>,
) -> BodyRegistry {
    let mut registry = BodyRegistry::default();
    let mut first_seen: BTreeMap<&str, usize> = BTreeMap::new();
    for (position, id) in ids.enumerate() {
        match registry.register(id) {
            Ok(_) => {
                first_seen.insert(id, position);
            }
            Err(IdTextError::Duplicate) => errors.push(RegistryError::DuplicateId {
                file,
                id: id.to_owned(),
                first: first_seen.get(id).copied().unwrap_or_default(),
                second: position,
            }),
            Err(reason) => errors.push(RegistryError::InvalidId {
                file,
                position,
                id: id.to_owned(),
                reason,
            }),
        }
    }
    registry
}

/// Destino de una referencia: algo que resuelve textos de ids.
trait RefTarget {
    /// Prefijo(s) esperado(s), para los mensajes.
    fn expected(&self) -> &'static str;
    /// Resuelve el texto; ver [`IdRegistry::from_text`].
    fn lookup(&self, text: &str) -> Result<(), IdTextError>;
}

impl<I: EntityId> RefTarget for IdRegistry<I> {
    fn expected(&self) -> &'static str {
        I::PREFIX
    }

    fn lookup(&self, text: &str) -> Result<(), IdTextError> {
        self.from_text(text).map(|_| ())
    }
}

impl RefTarget for BodyRegistry {
    fn expected(&self) -> &'static str {
        BODY_REF_LABEL
    }

    fn lookup(&self, text: &str) -> Result<(), IdTextError> {
        self.from_text(text).map(|_| ())
    }
}

define_id! {
    /// Id de una receta. Texto: `recipe.<nombre>`.
    RecipeId, "recipe"
}
define_id! {
    /// Id de una instalación. Texto: `facility.<nombre>`.
    FacilityId, "facility"
}
define_id! {
    /// Id de una empresa. Texto: `company.<nombre>`.
    CompanyId, "company"
}
define_id! {
    /// Id de un evento social de `events.json`. Texto: `event.<nombre>`.
    SocialEventId, "event"
}

/// Un error del registro de ids. Su `Display` es texto técnico en español para logs y la CLI.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RegistryError {
    /// El id de una entrada no tiene el formato `tipo.nombre` del archivo (sin punto, prefijo
    /// equivocado, nombre vacío, mayúsculas, caracteres inválidos...).
    #[error("{file}: [{position}]: id «{id}» inválido: {reason}")]
    InvalidId {
        /// Archivo, relativo a `data/`.
        file: &'static str,
        /// Posición (desde 0) de la entrada en el archivo.
        position: usize,
        /// Id tal cual figura en el archivo.
        id: String,
        /// Qué regla incumple.
        reason: IdTextError,
    },
    /// El mismo id aparece dos veces en un archivo.
    #[error("{file}: id «{id}» duplicado (posiciones [{first}] y [{second}])")]
    DuplicateId {
        /// Archivo, relativo a `data/`.
        file: &'static str,
        /// El id repetido.
        id: String,
        /// Posición de la primera aparición.
        first: usize,
        /// Posición de la repetición.
        second: usize,
    },
    /// Una referencia no tiene el formato de id (`tipo.nombre`).
    #[error("{file}: {entry}: {field}: referencia «{value}» mal formada: {reason}")]
    InvalidReference {
        /// Archivo de la entrada que referencia.
        file: &'static str,
        /// Id de la entrada que referencia (o su posición `[n]` si no tiene id).
        entry: String,
        /// Campo que contiene la referencia (`parent`, `deposits[1].good`...).
        field: String,
        /// Valor referido.
        value: String,
        /// Qué regla incumple.
        reason: IdTextError,
    },
    /// La referencia está bien formada pero apunta a otro tipo (p. ej. `parent` a un `good.*`).
    #[error("{file}: {entry}: {field}: «{value}» es de tipo «{found}» y se esperaba «{expected}»")]
    WrongTypeReference {
        /// Archivo de la entrada que referencia.
        file: &'static str,
        /// Id de la entrada que referencia (o su posición `[n]`).
        entry: String,
        /// Campo que contiene la referencia.
        field: String,
        /// Valor referido.
        value: String,
        /// Prefijo del valor referido.
        found: String,
        /// Prefijo esperado.
        expected: &'static str,
    },
    /// La referencia es del tipo correcto pero no existe ninguna entrada con ese id.
    #[error(
        "{file}: {entry}: {field}: «{value}» no existe (se esperaba un «{expected}.*» definido)"
    )]
    UnknownReference {
        /// Archivo de la entrada que referencia.
        file: &'static str,
        /// Id de la entrada que referencia (o su posición `[n]`).
        entry: String,
        /// Campo que contiene la referencia.
        field: String,
        /// Valor referido.
        value: String,
        /// Prefijo del tipo esperado.
        expected: &'static str,
    },
    /// Un campo de referencias (`prereqs`, `unlocks.designs`) no tiene la forma esperada.
    #[error("{file}: {entry}: {field}: {message}")]
    InvalidField {
        /// Archivo de la entrada.
        file: &'static str,
        /// Id de la entrada (o su posición `[n]`).
        entry: String,
        /// Campo.
        field: String,
        /// Qué se esperaba.
        message: &'static str,
    },
    /// La cadena de `parent` de los cuerpos forma un ciclo (un cuerpo que es su propio ancestro).
    #[error("{file}: ciclo en «parent»: {}", format_cycle(.cycle))]
    ParentCycle {
        /// Archivo, relativo a `data/`.
        file: &'static str,
        /// Ids del ciclo, empezando por el de la entrada más temprana del archivo; el último
        /// apunta de nuevo al primero. Un cuerpo que es su propio padre da un solo id.
        cycle: Vec<String>,
    },
}

fn format_cycle(cycle: &[String]) -> String {
    let mut out = cycle.join(" -> ");
    if let Some(first) = cycle.first() {
        out.push_str(" -> ");
        out.push_str(first);
    }
    out
}

/// Todos los errores de una validación (nunca vacío), uno por línea en su `Display`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryErrors(pub Vec<RegistryError>);

impl RegistryErrors {
    /// Los errores, en orden estable (ver el módulo).
    #[must_use]
    pub fn errors(&self) -> &[RegistryError] {
        &self.0
    }
}

impl fmt::Display for RegistryErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} error(es) de ids y referencias", self.0.len())?;
        for e in &self.0 {
            write!(f, "\n  - {e}")?;
        }
        Ok(())
    }
}

impl std::error::Error for RegistryErrors {}

/// Tablas texto <-> id de todos los datos, ya validados.
#[derive(Debug, Clone, Default)]
pub struct Registry {
    bodies: BodyRegistry,
    goods: IdRegistry<GoodId>,
    recipes: IdRegistry<RecipeId>,
    facilities: IdRegistry<FacilityId>,
    techs: IdRegistry<TechId>,
    designs: IdRegistry<DesignId>,
    companies: IdRegistry<CompanyId>,
    events: IdRegistry<SocialEventId>,
}

impl Registry {
    /// Construye el registro de `data` y verifica sus referencias cruzadas.
    ///
    /// # Errors
    ///
    /// [`RegistryErrors`] con **todos** los errores (ids inválidos o duplicados, referencias mal
    /// formadas, de tipo equivocado o inexistentes, ciclos de `parent`), en orden estable.
    pub fn build(data: &GameData) -> Result<Self, RegistryErrors> {
        let [
            f_bodies,
            f_goods,
            f_recipes,
            f_facilities,
            f_techs,
            f_designs,
            f_companies,
            _,
            f_events,
            _,
        ] = DATA_FILES;
        let mut errors = Vec::new();
        let registry = Self {
            bodies: register_bodies(
                f_bodies,
                data.bodies.iter().map(|b| b.id.as_str()),
                &mut errors,
            ),
            goods: register_entries(f_goods, &data.goods, &mut errors),
            recipes: register_entries(f_recipes, &data.recipes, &mut errors),
            facilities: register_entries(f_facilities, &data.facilities, &mut errors),
            techs: register_entries(f_techs, &data.techs, &mut errors),
            designs: register_entries(f_designs, &data.designs, &mut errors),
            companies: register_entries(f_companies, &data.companies, &mut errors),
            events: register_entries(f_events, &data.events, &mut errors),
        };
        registry.check_bodies(f_bodies, data, &mut errors);
        registry.check_techs(f_techs, &data.techs, &mut errors);
        registry.check_parent_cycles(f_bodies, data, &mut errors);
        if errors.is_empty() {
            Ok(registry)
        } else {
            Err(RegistryErrors(errors))
        }
    }

    /// Ids de `bodies.json`.
    #[must_use]
    pub fn bodies(&self) -> &BodyRegistry {
        &self.bodies
    }

    /// Ids de `goods.json`.
    #[must_use]
    pub fn goods(&self) -> &IdRegistry<GoodId> {
        &self.goods
    }

    /// Ids de `recipes.json`.
    #[must_use]
    pub fn recipes(&self) -> &IdRegistry<RecipeId> {
        &self.recipes
    }

    /// Ids de `facilities.json`.
    #[must_use]
    pub fn facilities(&self) -> &IdRegistry<FacilityId> {
        &self.facilities
    }

    /// Ids de `techs.json`.
    #[must_use]
    pub fn techs(&self) -> &IdRegistry<TechId> {
        &self.techs
    }

    /// Ids de `designs.json`.
    #[must_use]
    pub fn designs(&self) -> &IdRegistry<DesignId> {
        &self.designs
    }

    /// Ids de `companies.json`.
    #[must_use]
    pub fn companies(&self) -> &IdRegistry<CompanyId> {
        &self.companies
    }

    /// Ids de `events.json`.
    #[must_use]
    pub fn events(&self) -> &IdRegistry<SocialEventId> {
        &self.events
    }

    fn check_bodies(&self, file: &'static str, data: &GameData, errors: &mut Vec<RegistryError>) {
        for (index, body) in data.bodies.iter().enumerate() {
            let entry = label(&body.id, index);
            if let Some(parent) = &body.parent {
                check_ref(&self.bodies, file, &entry, "parent", parent, errors);
            }
            for (i, deposit) in body.deposits.iter().enumerate() {
                let field = format!("deposits[{i}].good");
                check_ref(&self.goods, file, &entry, &field, &deposit.good, errors);
            }
        }
    }

    fn check_techs(&self, file: &'static str, techs: &[Entry], errors: &mut Vec<RegistryError>) {
        for (index, tech) in techs.iter().enumerate() {
            let entry = label(&tech.id, index);
            if let Some(prereqs) = tech.fields.get("prereqs") {
                check_ref_list(&self.techs, file, &entry, "prereqs", prereqs, errors);
            }
            match tech.fields.get("unlocks") {
                None => {}
                Some(Value::Object(unlocks)) => {
                    if let Some(designs) = unlocks.get("designs") {
                        let field = "unlocks.designs";
                        check_ref_list(&self.designs, file, &entry, field, designs, errors);
                    }
                }
                Some(_) => errors.push(RegistryError::InvalidField {
                    file,
                    entry,
                    field: "unlocks".to_owned(),
                    message: "debe ser un objeto",
                }),
            }
        }
    }

    /// La jerarquía de cuerpos es un árbol: cada cuerpo tiene a lo sumo un padre, así que basta
    /// seguir la cadena de padres desde cada cuerpo y detectar si vuelve a un cuerpo de la
    /// misma cadena. Cada ciclo se informa una sola vez.
    fn check_parent_cycles(
        &self,
        file: &'static str,
        data: &GameData,
        errors: &mut Vec<RegistryError>,
    ) {
        // Padre de cada cuerpo registrado, indexado por el id numérico (orden de registro).
        let mut parents: Vec<Option<usize>> = vec![None; self.bodies.len()];
        let mut seen = BTreeSet::new();
        for body in &data.bodies {
            let Ok(id) = self.bodies.from_text(&body.id) else {
                continue;
            };
            // Una repetición del mismo id ya es un error `DuplicateId`: cuenta la primera.
            if !seen.insert(id.raw()) {
                continue;
            }
            let parent = body
                .parent
                .as_deref()
                .and_then(|p| self.bodies.from_text(p).ok())
                .and_then(|p| usize::try_from(p.raw()).ok());
            if let Some(slot) = usize::try_from(id.raw())
                .ok()
                .and_then(|i| parents.get_mut(i))
            {
                *slot = parent;
            }
        }

        // 0 = sin visitar, 1 = en la cadena actual, 2 = terminado.
        let mut state = vec![0_u8; parents.len()];
        for start in 0..parents.len() {
            let mut path: Vec<usize> = Vec::new();
            let mut current = Some(start);
            while let Some(node) = current {
                match state.get(node).copied() {
                    Some(0) => {
                        if let Some(s) = state.get_mut(node) {
                            *s = 1;
                        }
                        path.push(node);
                        current = parents.get(node).copied().flatten();
                    }
                    Some(1) => {
                        if let Some(at) = path.iter().position(|&n| n == node) {
                            let members = path.get(at..).unwrap_or_default();
                            errors.push(self.cycle_error(file, members));
                        }
                        break;
                    }
                    _ => break,
                }
            }
            for node in path {
                if let Some(s) = state.get_mut(node) {
                    *s = 2;
                }
            }
        }
    }

    /// Arma el error de un ciclo rotándolo para que empiece por el id más bajo (la entrada más
    /// temprana del archivo), así no depende de desde dónde se lo descubrió.
    fn cycle_error(&self, file: &'static str, members: &[usize]) -> RegistryError {
        let start = members
            .iter()
            .enumerate()
            .min_by_key(|&(_, &n)| n)
            .map_or(0, |(i, _)| i);
        let cycle = members
            .iter()
            .cycle()
            .skip(start)
            .take(members.len())
            .map(|&n| {
                u32::try_from(n)
                    .ok()
                    .and_then(|raw| self.bodies.to_text(BodyId::from_raw(raw)))
                    .unwrap_or_default()
            })
            .collect();
        RegistryError::ParentCycle { file, cycle }
    }
}

impl GameData {
    /// Construye el [`Registry`] de estos datos y verifica sus referencias (ver [`Registry::build`]).
    ///
    /// # Errors
    ///
    /// [`RegistryErrors`] con todos los errores encontrados.
    pub fn build_registry(&self) -> Result<Registry, RegistryErrors> {
        Registry::build(self)
    }
}

/// Resultado de [`DataLoader::load_validated`]: los datos y su registro, ya consistentes.
#[derive(Debug, Clone)]
pub struct ValidatedData {
    /// Los datos cargados.
    pub data: GameData,
    /// Sus ids y referencias, verificados.
    pub registry: Registry,
}

/// Error de [`DataLoader::load_validated`]: dice en qué etapa falló.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatedLoadError {
    /// Falló la carga de los archivos (lectura, JSON, esquema). No se llegó a validar ids.
    Load(DataLoadErrors),
    /// Los archivos cargaron pero los ids o las referencias son inválidos.
    Registry(RegistryErrors),
}

impl fmt::Display for ValidatedLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(e) => e.fmt(f),
            Self::Registry(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for ValidatedLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Load(e) => Some(e),
            Self::Registry(e) => Some(e),
        }
    }
}

impl From<DataLoadErrors> for ValidatedLoadError {
    fn from(e: DataLoadErrors) -> Self {
        Self::Load(e)
    }
}

impl From<RegistryErrors> for ValidatedLoadError {
    fn from(e: RegistryErrors) -> Self {
        Self::Registry(e)
    }
}

impl DataLoader {
    /// Carga `dir` ([`DataLoader::load_dir`]) y valida ids y referencias ([`Registry::build`]).
    ///
    /// Son dos etapas y se informa **una**: si la carga falla (archivo roto), los datos están
    /// incompletos y validar sus referencias daría falsos «no existe», así que sólo se devuelven
    /// los errores de carga ([`ValidatedLoadError::Load`]); si la carga anda, se devuelven todos
    /// los errores del registro ([`ValidatedLoadError::Registry`]). `load_dir` no cambia.
    ///
    /// # Errors
    ///
    /// [`ValidatedLoadError`], según la etapa que falle.
    pub fn load_validated(dir: impl AsRef<Path>) -> Result<ValidatedData, ValidatedLoadError> {
        let data = Self::load_dir(dir)?;
        let registry = Registry::build(&data)?;
        Ok(ValidatedData { data, registry })
    }
}

/// Nombre con que se identifica una entrada en los errores: su id o, si está vacío, `[posición]`.
fn label(id: &str, index: usize) -> String {
    if id.is_empty() {
        format!("[{index}]")
    } else {
        id.to_owned()
    }
}

fn register_entries<I: EntityId>(
    file: &'static str,
    entries: &[Entry],
    errors: &mut Vec<RegistryError>,
) -> IdRegistry<I> {
    register_all(file, entries.iter().map(|e| e.id.as_str()), errors)
}

/// Registra los ids de un archivo (en orden de aparición) y acumula los errores de formato y
/// de duplicados.
fn register_all<'a, I: EntityId>(
    file: &'static str,
    ids: impl Iterator<Item = &'a str>,
    errors: &mut Vec<RegistryError>,
) -> IdRegistry<I> {
    let mut registry = IdRegistry::new();
    let mut first_seen: BTreeMap<&str, usize> = BTreeMap::new();
    for (position, id) in ids.enumerate() {
        let invalid = |reason| RegistryError::InvalidId {
            file,
            position,
            id: id.to_owned(),
            reason,
        };
        let name = match parse_text::<I>(id) {
            Ok(name) => name,
            Err(reason) => {
                errors.push(invalid(reason));
                continue;
            }
        };
        match registry.register(name) {
            Ok(_) => {
                first_seen.insert(id, position);
            }
            Err(IdTextError::Duplicate) => errors.push(RegistryError::DuplicateId {
                file,
                id: id.to_owned(),
                first: first_seen.get(id).copied().unwrap_or_default(),
                second: position,
            }),
            Err(reason) => errors.push(invalid(reason)),
        }
    }
    registry
}

/// Verifica que `value` sea un id bien formado, del tipo de `target`, y que exista.
fn check_ref(
    target: &impl RefTarget,
    file: &'static str,
    entry: &str,
    field: &str,
    value: &str,
    errors: &mut Vec<RegistryError>,
) {
    let (entry, field, value) = (entry.to_owned(), field.to_owned(), value.to_owned());
    match target.lookup(&value) {
        Ok(()) => {}
        Err(IdTextError::Unknown) => errors.push(RegistryError::UnknownReference {
            file,
            entry,
            field,
            value,
            expected: target.expected(),
        }),
        Err(IdTextError::WrongPrefix { found, .. }) => {
            errors.push(RegistryError::WrongTypeReference {
                file,
                entry,
                field,
                value,
                found,
                expected: target.expected(),
            });
        }
        Err(reason) => errors.push(RegistryError::InvalidReference {
            file,
            entry,
            field,
            value,
            reason,
        }),
    }
}

/// Verifica un campo que debe ser una lista de ids del tipo de `target`.
fn check_ref_list(
    target: &impl RefTarget,
    file: &'static str,
    entry: &str,
    field: &str,
    list: &Value,
    errors: &mut Vec<RegistryError>,
) {
    let Value::Array(items) = list else {
        errors.push(RegistryError::InvalidField {
            file,
            entry: entry.to_owned(),
            field: field.to_owned(),
            message: "debe ser una lista de ids de texto",
        });
        return;
    };
    for (i, item) in items.iter().enumerate() {
        let item_field = format!("{field}[{i}]");
        match item.as_str() {
            Some(value) => check_ref(target, file, entry, &item_field, value, errors),
            None => errors.push(RegistryError::InvalidField {
                file,
                entry: entry.to_owned(),
                field: item_field,
                message: "debe ser un id de texto",
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::data::{Body, Deposit};

    fn body(id: &str, parent: Option<&str>, goods: &[&str]) -> Body {
        Body {
            id: id.to_owned(),
            kind: "moon".to_owned(),
            parent: parent.map(str::to_owned),
            orbit: None,
            radius_km: 1.0,
            detect_difficulty: 0,
            habitability: None,
            deposits: goods
                .iter()
                .map(|g| Deposit {
                    good: (*g).to_owned(),
                    reserve_mt: 1.0,
                    access: 0.5,
                })
                .collect(),
        }
    }

    fn entry(id: &str, fields: Value) -> Entry {
        let Value::Object(map) = fields else {
            panic!("los campos de prueba deben ser un objeto");
        };
        Entry {
            id: id.to_owned(),
            fields: map.into_iter().collect(),
        }
    }

    fn plain(ids: &[&str]) -> Vec<Entry> {
        ids.iter().map(|id| entry(id, json!({}))).collect()
    }

    fn errors_of(data: &GameData) -> Vec<RegistryError> {
        Registry::build(data).expect_err("se esperaban errores").0
    }

    #[test]
    fn datos_vacios_validan_ok() {
        let registry = Registry::build(&GameData::default()).unwrap();
        assert!(registry.bodies().is_empty() && registry.techs().is_empty());
    }

    #[test]
    fn el_ejemplo_de_la_luna_del_doc_22_valida() {
        let data = GameData {
            bodies: vec![
                body("body.tierra", None, &[]),
                body(
                    "body.luna",
                    Some("body.tierra"),
                    &["good.silicatos", "good.hierro", "good.hielo"],
                ),
            ],
            goods: plain(&["good.silicatos", "good.hierro", "good.hielo"]),
            ..GameData::default()
        };
        let reg = data.build_registry().unwrap();
        let luna = reg.bodies().from_text("body.luna").unwrap();
        assert_eq!(luna.raw(), 1);
        assert_eq!(reg.bodies().to_text(luna).as_deref(), Some("body.luna"));
        assert_eq!(reg.goods().from_text("good.hielo").unwrap().raw(), 2);
    }

    #[test]
    fn ids_deterministas_en_orden_de_aparicion_y_ida_y_vuelta() {
        let data = GameData {
            techs: plain(&["tech.b", "tech.INVALIDO", "tech.a", "tech.c"]),
            ..GameData::default()
        };
        // El inválido no consume número.
        let reg = Registry::build(&data).unwrap_err();
        assert_eq!(reg.errors().len(), 1);
        let data = GameData {
            techs: plain(&["tech.b", "tech.a", "tech.c"]),
            ..GameData::default()
        };
        let one = Registry::build(&data).unwrap();
        let two = Registry::build(&data).unwrap();
        for (id, name) in one.techs().iter() {
            assert_eq!(two.techs().get(name), Some(id));
            let text = one.techs().to_text(id).unwrap();
            assert_eq!(one.techs().from_text(&text), Ok(id));
        }
        assert_eq!(one.techs().get("b").map(EntityId::raw), Some(0));
        assert_eq!(one.techs().get("c").map(EntityId::raw), Some(2));
    }

    #[test]
    fn id_duplicado_invalido_y_referencia_inexistente_son_variantes_distintas() {
        let data = GameData {
            bodies: vec![
                body("body.a", None, &[]),
                body("body.a", None, &[]),
                body("Body.b", None, &[]),
                body("body.c", Some("body.fantasma"), &[]),
            ],
            ..GameData::default()
        };
        let errors = errors_of(&data);
        assert_eq!(errors.len(), 3, "{errors:?}");
        assert!(matches!(
            &errors[0],
            RegistryError::DuplicateId { id, first: 0, second: 1, .. } if id == "body.a"
        ));
        assert!(matches!(
            &errors[1],
            RegistryError::InvalidId {
                position: 2,
                reason: IdTextError::WrongPrefix { .. },
                ..
            }
        ));
        assert!(matches!(
            &errors[2],
            RegistryError::UnknownReference { entry, field, value, expected: "body|jp", .. }
                if entry == "body.c" && field == "parent" && value == "body.fantasma"
        ));
        assert_eq!(
            errors[2].to_string(),
            "bodies.json: body.c: parent: «body.fantasma» no existe (se esperaba un «body|jp.*» definido)"
        );
    }

    #[test]
    fn formatos_invalidos_de_id() {
        let data = GameData {
            goods: plain(&[
                "",
                "hierro",
                "good.",
                "good.Hierro",
                "good.a-b",
                "good._x",
                "good.a.b",
                "tech.x",
            ]),
            ..GameData::default()
        };
        let errors = errors_of(&data);
        let reasons: Vec<_> = errors
            .iter()
            .map(|e| match e {
                RegistryError::InvalidId { reason, .. } => reason.clone(),
                otro => panic!("no es InvalidId: {otro:?}"),
            })
            .collect();
        assert_eq!(reasons.len(), 8);
        assert_eq!(reasons[0], IdTextError::MissingSeparator);
        assert_eq!(reasons[1], IdTextError::MissingSeparator);
        assert_eq!(reasons[2], IdTextError::EmptyName);
        assert_eq!(reasons[3], IdTextError::InvalidName);
        assert_eq!(reasons[4], IdTextError::InvalidName);
        assert_eq!(reasons[5], IdTextError::InvalidName);
        assert_eq!(reasons[6], IdTextError::InvalidName);
        assert!(matches!(
            reasons[7],
            IdTextError::WrongPrefix {
                expected: "good",
                ..
            }
        ));
    }

    #[test]
    fn el_mismo_texto_en_dos_archivos_es_error_de_prefijo() {
        let data = GameData {
            goods: plain(&["good.hierro"]),
            techs: plain(&["good.hierro"]),
            ..GameData::default()
        };
        let errors = errors_of(&data);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            RegistryError::InvalidId {
                file: "techs.json",
                reason: IdTextError::WrongPrefix { .. },
                ..
            }
        ));
    }

    #[test]
    fn referencia_a_tipo_equivocado_y_mal_formada() {
        let data = GameData {
            bodies: vec![
                body("body.a", Some("good.hierro"), &["body.a"]),
                body("body.b", Some("Tierra"), &["good.NoExiste"]),
            ],
            goods: plain(&["good.hierro"]),
            ..GameData::default()
        };
        let errors = errors_of(&data);
        assert_eq!(errors.len(), 4, "{errors:?}");
        assert!(matches!(
            &errors[0],
            RegistryError::WrongTypeReference { field, found, expected: "body|jp", .. }
                if field == "parent" && found == "good"
        ));
        assert!(matches!(
            &errors[1],
            RegistryError::WrongTypeReference { field, found, expected: "good", .. }
                if field == "deposits[0].good" && found == "body"
        ));
        assert!(matches!(
            &errors[2],
            RegistryError::InvalidReference {
                reason: IdTextError::MissingSeparator,
                ..
            }
        ));
        assert!(matches!(
            &errors[3],
            RegistryError::InvalidReference { field, reason: IdTextError::InvalidName, .. }
                if field == "deposits[0].good"
        ));
    }

    #[test]
    fn techs_validan_prereqs_y_unlocks_designs() {
        let data = GameData {
            techs: vec![
                entry("tech.a", json!({})),
                entry(
                    "tech.b",
                    json!({
                        "prereqs": ["tech.a", "tech.zzz", "design.x"],
                        "unlocks": { "designs": ["design.ok", "design.no"], "effects": [] }
                    }),
                ),
                entry("tech.c", json!({ "prereqs": "tech.a", "unlocks": 3 })),
                entry(
                    "tech.d",
                    json!({ "prereqs": [7], "unlocks": { "designs": {} } }),
                ),
                entry("tech.e", json!({ "prereqs": ["tech.e"] })),
            ],
            designs: plain(&["design.ok"]),
            ..GameData::default()
        };
        let errors = errors_of(&data);
        let texts: Vec<_> = errors.iter().map(ToString::to_string).collect();
        assert_eq!(
            texts,
            [
                "techs.json: tech.b: prereqs[1]: «tech.zzz» no existe (se esperaba un «tech.*» definido)",
                "techs.json: tech.b: prereqs[2]: «design.x» es de tipo «design» y se esperaba «tech»",
                "techs.json: tech.b: unlocks.designs[1]: «design.no» no existe (se esperaba un «design.*» definido)",
                "techs.json: tech.c: prereqs: debe ser una lista de ids de texto",
                "techs.json: tech.c: unlocks: debe ser un objeto",
                "techs.json: tech.d: prereqs[0]: debe ser un id de texto",
                "techs.json: tech.d: unlocks.designs: debe ser una lista de ids de texto",
            ]
        );
    }

    #[test]
    fn ciclos_de_parent_se_informan_una_vez() {
        let data = GameData {
            bodies: vec![
                body("body.cola", Some("body.b"), &[]),
                body("body.a", Some("body.b"), &[]),
                body("body.b", Some("body.a"), &[]),
                body("body.solo", Some("body.solo"), &[]),
                body("body.sol", None, &[]),
            ],
            ..GameData::default()
        };
        let errors = errors_of(&data);
        assert_eq!(
            errors,
            [
                RegistryError::ParentCycle {
                    file: "bodies.json",
                    cycle: vec!["body.a".into(), "body.b".into()],
                },
                RegistryError::ParentCycle {
                    file: "bodies.json",
                    cycle: vec!["body.solo".into()],
                },
            ]
        );
        assert_eq!(
            errors[0].to_string(),
            "bodies.json: ciclo en «parent»: body.a -> body.b -> body.a"
        );
        assert_eq!(
            errors[1].to_string(),
            "bodies.json: ciclo en «parent»: body.solo -> body.solo"
        );
    }

    #[test]
    fn un_ciclo_no_oculta_ni_duplica_otros_errores() {
        let data = GameData {
            bodies: vec![
                body("body.a", Some("body.a"), &["good.x"]),
                body("body.a", Some("body.b"), &[]),
            ],
            ..GameData::default()
        };
        let errors = errors_of(&data);
        assert_eq!(errors.len(), 4, "{errors:?}");
        assert!(matches!(errors[0], RegistryError::DuplicateId { .. }));
        assert!(matches!(errors[1], RegistryError::UnknownReference { .. }));
        assert!(matches!(errors[2], RegistryError::UnknownReference { .. }));
        assert!(matches!(errors[3], RegistryError::ParentCycle { .. }));
    }

    #[test]
    fn entradas_sin_id_se_identifican_por_posicion_y_no_hacen_panic() {
        let data = GameData {
            bodies: vec![body("", Some(""), &[""])],
            ..GameData::default()
        };
        let errors = errors_of(&data);
        assert_eq!(errors.len(), 3, "{errors:?}");
        assert!(
            errors[1]
                .to_string()
                .starts_with("bodies.json: [0]: parent: ")
        );
    }

    #[test]
    fn el_display_agrupa_los_errores_una_por_linea() {
        let data = GameData {
            goods: plain(&["x", "good.a", "good.a"]),
            ..GameData::default()
        };
        let text = Registry::build(&data).unwrap_err().to_string();
        assert_eq!(text.lines().count(), 3);
        assert!(text.starts_with("2 error(es) de ids y referencias"));
    }

    #[test]
    fn el_orden_de_errores_es_estable() {
        let data = GameData {
            bodies: vec![
                body("body.a", Some("body.q"), &["good.q"]),
                body("x", None, &[]),
            ],
            goods: plain(&["good.a", "good.a"]),
            techs: vec![entry("tech.t", json!({ "prereqs": ["tech.q"] }))],
            ..GameData::default()
        };
        let first = errors_of(&data);
        assert_eq!(first, errors_of(&data));
        let files: Vec<_> = first
            .iter()
            .map(|e| match e {
                RegistryError::InvalidId { file, .. }
                | RegistryError::DuplicateId { file, .. }
                | RegistryError::UnknownReference { file, .. } => *file,
                otro => panic!("inesperado {otro:?}"),
            })
            .collect();
        // Primero los ids (bodies, goods), luego las referencias (bodies x2, techs).
        assert_eq!(
            files,
            [
                "bodies.json",
                "goods.json",
                "bodies.json",
                "bodies.json",
                "techs.json"
            ]
        );
    }

    #[test]
    fn el_punto_de_salto_es_un_cuerpo_con_prefijo_jp() {
        let data = GameData {
            bodies: vec![
                body("body.sol", None, &[]),
                body("jp.alfa", Some("body.sol"), &["good.hierro"]),
                body("body.nave", Some("jp.alfa"), &[]),
                body("body.alfa", None, &[]),
            ],
            goods: plain(&["good.hierro"]),
            ..GameData::default()
        };
        let reg = data.build_registry().unwrap();
        let jp = reg.bodies().from_text("jp.alfa").unwrap();
        assert_eq!(jp.raw(), 1);
        assert_eq!(reg.bodies().to_text(jp).as_deref(), Some("jp.alfa"));
        // `body.alfa` y `jp.alfa` son ids distintos, no duplicados.
        let alfa = reg.bodies().from_text("body.alfa").unwrap();
        assert_ne!(alfa, jp);
        assert_eq!(reg.bodies().to_text(alfa).as_deref(), Some("body.alfa"));
        let texts: Vec<_> = reg.bodies().iter().map(|(_, t)| t).collect();
        assert_eq!(texts, ["body.sol", "jp.alfa", "body.nave", "body.alfa"]);
    }

    #[test]
    fn jp_en_otro_archivo_es_error_de_prefijo() {
        let data = GameData {
            goods: plain(&["jp.alfa"]),
            techs: vec![entry("tech.t", json!({ "prereqs": ["jp.alfa"] }))],
            ..GameData::default()
        };
        let errors = errors_of(&data);
        assert!(matches!(
            &errors[0],
            RegistryError::InvalidId {
                file: "goods.json",
                reason: IdTextError::WrongPrefix {
                    expected: "good",
                    ..
                },
                ..
            }
        ));
        assert!(
            matches!(&errors[1], RegistryError::WrongTypeReference { found, .. } if found == "jp")
        );
    }

    #[test]
    fn jp_duplicado_y_jp_invalido_son_errores_de_id() {
        let data = GameData {
            bodies: vec![
                body("jp.alfa", None, &[]),
                body("jp.alfa", None, &[]),
                body("jp.Alfa", None, &[]),
                body("jp.", None, &[]),
                body("jp", None, &[]),
            ],
            ..GameData::default()
        };
        let errors = errors_of(&data);
        assert!(
            matches!(&errors[0], RegistryError::DuplicateId { id, first: 0, second: 1, .. } if id == "jp.alfa")
        );
        assert!(matches!(
            &errors[1],
            RegistryError::InvalidId {
                reason: IdTextError::InvalidName,
                ..
            }
        ));
        assert!(matches!(
            &errors[2],
            RegistryError::InvalidId {
                reason: IdTextError::EmptyName,
                ..
            }
        ));
        assert!(matches!(
            &errors[3],
            RegistryError::InvalidId {
                reason: IdTextError::MissingSeparator,
                ..
            }
        ));
        assert_eq!(errors.len(), 4);
    }

    #[test]
    fn parent_a_un_jp_inexistente_o_de_tipo_equivocado_y_ciclo_por_un_jp() {
        let data = GameData {
            bodies: vec![
                body("body.a", Some("jp.nada"), &[]),
                body("body.b", Some("tech.x"), &[]),
                body("jp.p", Some("body.q"), &[]),
                body("body.q", Some("jp.p"), &[]),
            ],
            ..GameData::default()
        };
        let texts: Vec<_> = errors_of(&data).iter().map(ToString::to_string).collect();
        assert_eq!(
            texts,
            [
                "bodies.json: body.a: parent: «jp.nada» no existe (se esperaba un «body|jp.*» definido)",
                "bodies.json: body.b: parent: «tech.x» es de tipo «tech» y se esperaba «body|jp»",
                "bodies.json: ciclo en «parent»: jp.p -> body.q -> jp.p",
            ]
        );
    }
}
