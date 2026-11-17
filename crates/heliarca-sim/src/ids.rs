//! IDs tipados y su forma de texto estable (doc 21 §2 «Identificadores y arenas»; doc 22 §0).
//!
//! # Dos formas de un mismo id
//!
//! - **Forma numérica** (`u32`): un nuevo tipo por entidad ([`BodyId`], [`ShipId`], …) creado con
//!   [`define_id!`]. Es barata, `Copy` y ordenable. Los ids de entidades dinámicas los entrega una
//!   [`Arena`](crate::arena::Arena), que empaqueta índice y generación en el `u32`.
//! - **Forma de texto** (`body.luna`): la «clave de datos» estable que viaja en los archivos de
//!   datos y en los guardados. Tiene el formato `<prefijo>.<nombre>`: el prefijo lo fija el tipo
//!   ([`EntityId::PREFIX`]) y el nombre es un *slug* validado (ver [`validate_name`]).
//!
//! Un `u32` **no** se puede convertir a texto sin una tabla de nombres (el número no contiene el
//! nombre). Esa tabla es el [`IdRegistry`]: asigna ids secuenciales y de forma determinista a los
//! nombres (en orden de registro) y los traduce en ambos sentidos.
//!
//! # Qué queda para #32
//!
//! Este módulo sólo da las piezas. El *cargador de datos* (#31/#32) deberá: poblar un
//! [`IdRegistry`] por tipo desde los archivos de datos, validar que toda referencia textual
//! resuelva (validador de referencias) y decidir cómo se vincula un id de datos con la entidad
//! viva en su arena (p. ej. guardando la clave de datos en la entidad).
//!
//! # Seguridad de tipos
//!
//! ```compile_fail,E0308
//! use heliarca_sim::ids::{BodyId, EntityId, ShipId};
//!
//! fn necesita_nave(_: ShipId) {}
//! necesita_nave(BodyId::from_raw(1));
//! ```

use std::collections::BTreeMap;
use std::fmt;

/// Longitud máxima, en bytes, del nombre (la parte posterior al prefijo).
pub const MAX_NAME_LEN: usize = 64;

/// Comportamiento común de los ids tipados; lo implementa [`define_id!`].
pub trait EntityId: Copy + Eq + Ord + fmt::Debug {
    /// Prefijo de texto del tipo de entidad (p. ej. `"body"`).
    const PREFIX: &'static str;

    /// Crea el id a partir de su valor numérico crudo.
    #[must_use]
    fn from_raw(raw: u32) -> Self;

    /// Valor numérico crudo del id.
    #[must_use]
    fn raw(self) -> u32;
}

/// Define un nuevo tipo de id sobre `u32` e implementa [`EntityId`].
///
/// ```
/// use heliarca_sim::define_id;
/// use heliarca_sim::ids::EntityId;
///
/// define_id! {
///     /// Id de una estación.
///     StationId, "station"
/// }
/// assert_eq!(StationId::PREFIX, "station");
/// assert_eq!(StationId::from_raw(7).raw(), 7);
/// ```
#[macro_export]
macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident, $prefix:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u32);

        impl $crate::ids::EntityId for $name {
            const PREFIX: &'static str = $prefix;

            fn from_raw(raw: u32) -> Self {
                Self(raw)
            }

            fn raw(self) -> u32 {
                self.0
            }
        }
    };
}

define_id! {
    /// Id de un cuerpo celeste (planeta, luna, asteroide). Texto: `body.<nombre>`.
    BodyId, "body"
}
define_id! {
    /// Id de una nave. Texto: `ship.<nombre>`.
    ShipId, "ship"
}
define_id! {
    /// Id de un bien (mercancía). Texto: `good.<nombre>`.
    GoodId, "good"
}
define_id! {
    /// Id de una tecnología. Texto: `tech.<nombre>`.
    TechId, "tech"
}
define_id! {
    /// Id de un diseño de nave. Texto: `design.<nombre>`.
    DesignId, "design"
}

/// Error al convertir entre ids y texto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdTextError {
    /// El texto no tiene la forma `<prefijo>.<nombre>`.
    MissingSeparator,
    /// El prefijo del texto no es el del tipo pedido.
    WrongPrefix {
        /// Prefijo esperado.
        expected: &'static str,
        /// Prefijo encontrado.
        found: String,
    },
    /// El nombre está vacío.
    EmptyName,
    /// El nombre supera [`MAX_NAME_LEN`] bytes.
    NameTooLong,
    /// El nombre tiene un carácter inválido o no empieza con minúscula o dígito.
    InvalidName,
    /// El nombre ya estaba registrado.
    Duplicate,
    /// El nombre no está registrado.
    Unknown,
    /// El registro no admite más ids (o el valor no cabe en un `u32`).
    RegistryFull,
}

impl fmt::Display for IdTextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSeparator => write!(f, "falta el separador «.» (formato prefijo.nombre)"),
            Self::WrongPrefix { expected, found } => {
                write!(f, "prefijo «{found}» incorrecto, se esperaba «{expected}»")
            }
            Self::EmptyName => write!(f, "el nombre está vacío"),
            Self::NameTooLong => write!(f, "el nombre supera {MAX_NAME_LEN} bytes"),
            Self::InvalidName => write!(
                f,
                "nombre inválido: sólo a-z, 0-9 y «_», y debe empezar con letra o dígito"
            ),
            Self::Duplicate => write!(f, "nombre ya registrado"),
            Self::Unknown => write!(f, "nombre no registrado"),
            Self::RegistryFull => write!(f, "el registro de ids está lleno"),
        }
    }
}

impl std::error::Error for IdTextError {}

/// Valida un nombre (*slug*): 1 a [`MAX_NAME_LEN`] bytes de `a-z`, `0-9` y `_`, que no empiece
/// con `_`. Cubre los ejemplos del doc 22 (`luna`, `pro_0_1`, `lv_1`).
///
/// # Errors
///
/// [`IdTextError::EmptyName`], [`IdTextError::NameTooLong`] o [`IdTextError::InvalidName`].
pub fn validate_name(name: &str) -> Result<(), IdTextError> {
    if name.is_empty() {
        return Err(IdTextError::EmptyName);
    }
    if name.len() > MAX_NAME_LEN {
        return Err(IdTextError::NameTooLong);
    }
    let valid_chars = name
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    if !valid_chars || name.starts_with('_') {
        return Err(IdTextError::InvalidName);
    }
    Ok(())
}

/// Arma el texto `<prefijo>.<nombre>` del tipo `I`.
///
/// # Errors
///
/// Si `name` no pasa [`validate_name`].
pub fn format_text<I: EntityId>(name: &str) -> Result<String, IdTextError> {
    validate_name(name)?;
    Ok(format!("{}.{name}", I::PREFIX))
}

/// Separa un texto `<prefijo>.<nombre>` del tipo `I` y devuelve el nombre validado.
///
/// # Errors
///
/// Si falta el separador, el prefijo no es el de `I` o el nombre es inválido.
pub fn parse_text<I: EntityId>(text: &str) -> Result<&str, IdTextError> {
    let (prefix, name) = text.split_once('.').ok_or(IdTextError::MissingSeparator)?;
    if prefix != I::PREFIX {
        return Err(IdTextError::WrongPrefix {
            expected: I::PREFIX,
            found: prefix.to_owned(),
        });
    }
    validate_name(name)?;
    Ok(name)
}

/// Tabla determinista nombre ↔ id para un tipo de id.
///
/// Los ids se asignan secuencialmente (0, 1, 2, …) en orden de registro, de modo que registrar
/// los mismos nombres en el mismo orden da siempre los mismos ids. La iteración es en orden de id.
#[derive(Debug, Clone)]
pub struct IdRegistry<I: EntityId> {
    names: Vec<String>,
    by_name: BTreeMap<String, I>,
}

impl<I: EntityId> Default for IdRegistry<I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: EntityId> IdRegistry<I> {
    /// Crea un registro vacío.
    #[must_use]
    pub fn new() -> Self {
        Self {
            names: Vec::new(),
            by_name: BTreeMap::new(),
        }
    }

    /// Cantidad de ids registrados.
    #[must_use]
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Indica si no hay ids registrados.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// Registra `name` y devuelve su id nuevo.
    ///
    /// # Errors
    ///
    /// Nombre inválido, [`IdTextError::Duplicate`] o [`IdTextError::RegistryFull`].
    pub fn register(&mut self, name: &str) -> Result<I, IdTextError> {
        validate_name(name)?;
        if self.by_name.contains_key(name) {
            return Err(IdTextError::Duplicate);
        }
        let raw = u32::try_from(self.names.len()).map_err(|_| IdTextError::RegistryFull)?;
        let id = I::from_raw(raw);
        self.names.push(name.to_owned());
        self.by_name.insert(name.to_owned(), id);
        Ok(id)
    }

    /// Id de un nombre registrado.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<I> {
        self.by_name.get(name).copied()
    }

    /// Nombre de un id, si pertenece a este registro.
    #[must_use]
    pub fn name(&self, id: I) -> Option<&str> {
        let index = usize::try_from(id.raw()).ok()?;
        self.names.get(index).map(String::as_str)
    }

    /// Texto estable (`prefijo.nombre`) de un id, si pertenece a este registro.
    #[must_use]
    pub fn to_text(&self, id: I) -> Option<String> {
        self.name(id).map(|name| format!("{}.{name}", I::PREFIX))
    }

    /// Resuelve un texto `prefijo.nombre` a su id.
    ///
    /// # Errors
    ///
    /// Texto mal formado o [`IdTextError::Unknown`] si el nombre no está registrado.
    pub fn from_text(&self, text: &str) -> Result<I, IdTextError> {
        let name = parse_text::<I>(text)?;
        self.get(name).ok_or(IdTextError::Unknown)
    }

    /// Recorre `(id, nombre)` en orden ascendente de id (orden de registro).
    pub fn iter(&self) -> impl Iterator<Item = (I, &str)> + '_ {
        self.names
            .iter()
            .enumerate()
            .filter_map(|(i, name)| Some((I::from_raw(u32::try_from(i).ok()?), name.as_str())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ida_y_vuelta_texto_id() {
        let mut reg = IdRegistry::<BodyId>::new();
        let tierra = reg.register("tierra").unwrap();
        let luna = reg.register("luna").unwrap();
        assert_ne!(tierra, luna);
        assert_eq!(reg.to_text(luna).as_deref(), Some("body.luna"));
        assert_eq!(reg.from_text("body.luna"), Ok(luna));
        assert_eq!(reg.from_text(&reg.to_text(tierra).unwrap()), Ok(tierra));
        assert_eq!(reg.name(luna), Some("luna"));
        assert_eq!(reg.get("luna"), Some(luna));
    }

    #[test]
    fn orden_estable_por_registro() {
        let mut reg = IdRegistry::<GoodId>::new();
        for name in ["hierro", "hielo", "silicatos"] {
            reg.register(name).unwrap();
        }
        let names: Vec<_> = reg.iter().map(|(_, n)| n).collect();
        assert_eq!(names, ["hierro", "hielo", "silicatos"]);
        let raws: Vec<_> = reg.iter().map(|(id, _)| id.raw()).collect();
        assert_eq!(raws, [0, 1, 2]);
        assert_eq!(reg.len(), 3);
        assert!(!reg.is_empty());
    }

    #[test]
    fn errores_de_texto() {
        let mut reg = IdRegistry::<BodyId>::new();
        reg.register("luna").unwrap();
        assert_eq!(reg.from_text("luna"), Err(IdTextError::MissingSeparator));
        assert_eq!(
            reg.from_text("ship.luna"),
            Err(IdTextError::WrongPrefix {
                expected: "body",
                found: "ship".to_owned()
            })
        );
        assert_eq!(reg.from_text("body."), Err(IdTextError::EmptyName));
        assert_eq!(reg.from_text("body.Luna"), Err(IdTextError::InvalidName));
        assert_eq!(reg.from_text("body.marte"), Err(IdTextError::Unknown));
        assert_eq!(reg.register("luna"), Err(IdTextError::Duplicate));
        assert_eq!(reg.register("_x"), Err(IdTextError::InvalidName));
        assert_eq!(reg.register("a b"), Err(IdTextError::InvalidName));
        assert_eq!(
            reg.register(&"a".repeat(MAX_NAME_LEN + 1)),
            Err(IdTextError::NameTooLong)
        );
        assert!(reg.register(&"a".repeat(MAX_NAME_LEN)).is_ok());
    }

    #[test]
    fn nombres_del_doc_22_son_validos() {
        for name in [
            "luna", "tierra", "hierro", "pro_0_1", "ene_0_1", "lv_1", "snd_1",
        ] {
            assert_eq!(validate_name(name), Ok(()), "{name}");
        }
    }

    #[test]
    fn formato_y_parseo_sin_registro() {
        assert_eq!(format_text::<ShipId>("alba").as_deref(), Ok("ship.alba"));
        assert_eq!(parse_text::<ShipId>("ship.alba"), Ok("alba"));
        assert_eq!(format_text::<ShipId>(""), Err(IdTextError::EmptyName));
    }

    #[test]
    fn id_de_otro_registro_no_resuelve() {
        let reg = IdRegistry::<TechId>::new();
        assert_eq!(reg.name(TechId::from_raw(0)), None);
        assert_eq!(reg.to_text(TechId::from_raw(5)), None);
    }

    #[test]
    fn mensajes_de_error_no_estan_vacios() {
        assert_ne!(IdTextError::Unknown.to_string(), "");
        assert_ne!(IdTextError::RegistryFull.to_string(), "");
    }
}
