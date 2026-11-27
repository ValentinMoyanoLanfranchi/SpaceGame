//! Parámetros globales del juego (`data/params.json`, doc 22 §0).
//!
//! # La regla: todo número del GDD es un parámetro
//!
//! Cualquier número que el diseño pueda querer ajustar (o que el GDD fije como convención, como
//! los días por semana, mes y año de las cadencias, doc 22 §0) **vive en datos**, no hardcodeado
//! en el código (doc 21 §2 y §4, CONTRIBUTING.md). Cambiar el valor en `data/params.json` cambia el
//! comportamiento **sin recompilar**.
//!
//! **Qué sí queda como constante de código:** las *definiciones de unidad* y las identidades
//! físicas que ningún diseño ajusta (60 s por minuto, 60 min por hora, 24 h por día, y por lo tanto
//! `SECONDS_PER_DAY`). Si dudás, preguntate «¿el diseño podría querer otro valor?»: si sí, es un
//! parámetro.
//!
//! # Cómo agregar un parámetro
//!
//! 1. Agregar el campo (con la unidad en el nombre y en el doc) a [`Params`], su valor por defecto
//!    del GDD en [`Params::default`] y, si hace falta, una validación de rango en `validate`.
//! 2. Agregar el nombre del campo a `FIELDS` y su lectura en `Params::from_json_value`.
//! 3. Agregar el valor a `data/params.json` (todos los parámetros son obligatorios: no hay valores
//!    implícitos que oculten un archivo incompleto).
//! 4. Si el repo ya genera esquemas JSON de los modelos (`data/schema/`), regenerar el de params.
//! 5. Leerlo vía [`Sim::params`](crate::Sim::params); nunca con un literal.
//! 6. Agregar tests: valor por defecto = GDD, valor inválido, y que cambiarlo cambia el comportamiento.
//!
//! # Carga y errores
//!
//! [`DataLoader::load_dir`](crate::data::DataLoader::load_dir) lee `params.json` junto al resto de
//! los datos y lo deja en [`GameData::params`](crate::data::GameData::params). Todos los
//! parámetros son obligatorios y `params.json` no admite campos desconocidos (un typo es un error,
//! no se ignora). Los errores se acumulan con el resto y dicen archivo y campo:
//! `params.json: days_per_month: falta el parámetro obligatorio`.
//!
//! # Validación
//!
//! - cada parámetro de días es un entero entre 1 y [`Params::MAX_DAYS`], salvo `days_per_year`,
//!   cuyo tope es [`Params::MAX_DAYS_PER_YEAR`] (el día del año es un `u16`,
//!   [`CalendarDate::day`](crate::CalendarDate::day)); por el orden de abajo, ese tope acota
//!   también a la semana y al mes;
//! - `days_per_week <= days_per_month <= days_per_year` (una semana más larga que un mes, o un mes
//!   más largo que un año, no tiene sentido). El mes **no** tiene que dividir al año (doc 22 §0:
//!   12 meses = 360 días).
//!
//! # Inmutabilidad y alcance
//!
//! [`Params`] no tiene setters ni campos públicos: se construye ya validado ([`Params::new`], la
//! carga de datos o [`Params::default`]) y [`Sim`](crate::Sim) lo guarda sin exponer mutación.
//!
//! Los parámetros gobiernan las **cadencias del planificador** (semanal, mensual y anual) y el
//! largo del año del [`Calendar`](crate::Calendar): la cadencia anual y el calendario usan el mismo
//! `days_per_year`. Las cadencias cuentan desde el inicio de la partida y el calendario desde su
//! fecha de inicio, así que la cadencia anual coincide con el cambio de año cuando la partida
//! empieza en el día 1 a las 00:00:00 (la fecha por defecto).

use std::fmt;

use schemars::Schema;
use serde_json::{Value, json};

use crate::schedule::{DAYS_PER_MONTH, DAYS_PER_WEEK};
use crate::time::DAYS_PER_YEAR;

/// Campos de `params.json`, en el orden en que se leen e informan.
const FIELDS: [&str; 3] = ["days_per_week", "days_per_month", "days_per_year"];

/// Esquema JSON de `params.json` (ver `crate::schema`). `Params` se parsea a mano (para dar
/// mensajes en español), así que su esquema se arma a mano, pero **desde los mismos datos que el
/// cargador**: la lista de campos (`FIELDS`: obligatorios, sin campos desconocidos) y el tope de
/// cada uno (`max_days`). Agregar un parámetro a `FIELDS` lo agrega al esquema.
///
/// No expresa `days_per_week <= days_per_month <= days_per_year`: es validación semántica del
/// cargador (JSON Schema no compara campos entre sí). Tampoco coincide con el cargador en `7.0`
/// (entero para el esquema; `as_u64` del cargador lo rechaza).
pub(crate) fn json_schema() -> Schema {
    let mut properties = serde_json::Map::new();
    for field in FIELDS {
        let max = max_days(field);
        properties.insert(
            field.to_owned(),
            json!({
                "description": format!("Parámetro de días `{field}` (entero entre 1 y {max})."),
                "type": "integer",
                "minimum": 1,
                "maximum": max,
            }),
        );
    }
    schemars::json_schema!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Params",
        "description": "Parámetros globales del juego (`data/params.json`, doc 22 §0). Todos son obligatorios; no se admiten campos desconocidos. Además debe cumplirse days_per_week <= days_per_month <= days_per_year (lo valida el cargador).",
        "type": "object",
        "properties": properties,
        "required": FIELDS,
        "additionalProperties": false,
    })
}

/// Tope de un parámetro de días de `FIELDS`.
fn max_days(field: &str) -> u64 {
    if field == "days_per_year" {
        Params::MAX_DAYS_PER_YEAR
    } else {
        Params::MAX_DAYS
    }
}

/// Un parámetro inválido: su nombre y qué le pasa. `Display`: `days_per_month: debe ser >= 1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamsError {
    field: String,
    message: String,
}

impl ParamsError {
    fn new(field: &str, message: impl Into<String>) -> Self {
        Self {
            field: field.to_owned(),
            message: message.into(),
        }
    }

    /// Nombre del parámetro (`days_per_month`).
    #[must_use]
    pub fn field(&self) -> &str {
        &self.field
    }

    /// Descripción del problema.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ParamsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ParamsError {}

/// Parámetros globales del juego: inmutables una vez construidos.
///
/// # Ejemplo
///
/// ```
/// use heliarca_sim::{Params, Scenario, Sim};
///
/// let params = Params::new(7, 10, 365)?; // meses de 10 días
/// let sim = Sim::new(Scenario::new().with_params(params), 1)?;
/// assert_eq!(sim.params().days_per_month(), 10);
/// assert!(Params::new(7, 0, 365).is_err()); // 0 días por mes: inválido
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
// Los nombres de los campos son los de `params.json`, con el prefijo común a propósito.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Params {
    days_per_week: u64,
    days_per_month: u64,
    days_per_year: u64,
}

impl Params {
    /// Máximo de días de un período (evita desbordes al pasar a segundos). El año tiene un tope
    /// propio, [`Params::MAX_DAYS_PER_YEAR`].
    pub const MAX_DAYS: u64 = 1_000_000;

    /// Máximo de `days_per_year`: `u16::MAX`, porque el día del año
    /// ([`CalendarDate::day`](crate::CalendarDate::day)) es un `u16`.
    pub const MAX_DAYS_PER_YEAR: u64 = u16::MAX as u64;

    /// Los valores del GDD (doc 22 §0): semana 7, mes 30, año 365 días.
    pub const DEFAULT: Self = Self {
        days_per_week: DAYS_PER_WEEK,
        days_per_month: DAYS_PER_MONTH,
        days_per_year: DAYS_PER_YEAR,
    };

    /// Crea parámetros validados.
    ///
    /// # Errors
    ///
    /// El primer [`ParamsError`] de la validación (ver la documentación del módulo).
    pub fn new(
        days_per_week: u64,
        days_per_month: u64,
        days_per_year: u64,
    ) -> Result<Self, ParamsError> {
        match validate(days_per_week, days_per_month, days_per_year)
            .into_iter()
            .next()
        {
            Some(error) => Err(error),
            None => Ok(Self {
                days_per_week,
                days_per_month,
                days_per_year,
            }),
        }
    }

    /// Días por semana (convención de cadencias).
    #[must_use]
    pub const fn days_per_week(&self) -> u64 {
        self.days_per_week
    }

    /// Días por mes (convención de cadencias).
    #[must_use]
    pub const fn days_per_month(&self) -> u64 {
        self.days_per_month
    }

    /// Días por año (cadencia anual y largo del año del calendario).
    #[must_use]
    pub const fn days_per_year(&self) -> u64 {
        self.days_per_year
    }

    /// Campos que entran al hash de estado, en orden: `(days_per_week, days_per_month,
    /// days_per_year)`. Desestructura **sin `..`**: agregar un parámetro sin decidir si se
    /// hashea no compila (y todo parámetro afecta el comportamiento, así que debería hashearse).
    pub(crate) const fn hash_parts(&self) -> [u64; 3] {
        let Self {
            days_per_week,
            days_per_month,
            days_per_year,
        } = *self;
        [days_per_week, days_per_month, days_per_year]
    }

    /// Lee y valida un `params.json` ya parseado. Devuelve **todos** los errores.
    pub(crate) fn from_json_value(root: &Value) -> Result<Self, Vec<ParamsError>> {
        let Value::Object(map) = root else {
            // El cargador ya verificó que la raíz es un objeto; por si se llama directo.
            return Err(vec![ParamsError::new("(raíz)", "debe ser un objeto")]);
        };
        let mut errors = Vec::new();
        // `map` es un `BTreeMap` (sin `preserve_order`): los desconocidos salen en orden estable.
        for key in map.keys() {
            if !FIELDS.contains(&key.as_str()) {
                errors.push(ParamsError::new(
                    key,
                    format!("campo desconocido (esperados: {})", FIELDS.join(", ")),
                ));
            }
        }
        let mut values = [0_u64; FIELDS.len()];
        let mut complete = true;
        for (slot, field) in values.iter_mut().zip(FIELDS) {
            match map.get(field) {
                None => {
                    errors.push(ParamsError::new(field, "falta el parámetro obligatorio"));
                    complete = false;
                }
                Some(value) => {
                    if let Some(n) = value.as_u64() {
                        *slot = n;
                    } else {
                        errors.push(ParamsError::new(
                            field,
                            format!("se esperaba un entero no negativo, se obtuvo {value}"),
                        ));
                        complete = false;
                    }
                }
            }
        }
        let [week, month, year] = values;
        if complete {
            errors.extend(validate(week, month, year));
        }
        if errors.is_empty() {
            Ok(Self {
                days_per_week: week,
                days_per_month: month,
                days_per_year: year,
            })
        } else {
            Err(errors)
        }
    }
}

impl Default for Params {
    /// Los valores del GDD ([`Params::DEFAULT`]).
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Todas las violaciones de rango, en orden estable.
fn validate(week: u64, month: u64, year: u64) -> Vec<ParamsError> {
    let mut errors = Vec::new();
    for (field, days) in FIELDS.into_iter().zip([week, month, year]) {
        let max = max_days(field);
        if !(1..=max).contains(&days) {
            errors.push(ParamsError::new(
                field,
                format!("fuera de rango: {days} (debe estar entre 1 y {max})"),
            ));
        }
    }
    if week > month {
        errors.push(ParamsError::new(
            "days_per_week",
            format!("{week} no puede superar a days_per_month ({month})"),
        ));
    }
    if month > year {
        errors.push(ParamsError::new(
            "days_per_month",
            format!("{month} no puede superar a days_per_year ({year})"),
        ));
    }
    errors
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn parse(v: &Value) -> Result<Params, Vec<String>> {
        Params::from_json_value(v)
            .map_err(|e| e.iter().map(ToString::to_string).collect::<Vec<_>>())
    }

    #[test]
    fn el_default_es_el_del_gdd() {
        let p = Params::default();
        assert_eq!(
            (p.days_per_week(), p.days_per_month(), p.days_per_year()),
            (7, 30, 365)
        );
        assert_eq!(Params::new(7, 30, 365), Ok(p));
    }

    #[test]
    fn json_valido() {
        let p = parse(&json!({"days_per_week": 5, "days_per_month": 10, "days_per_year": 100}));
        assert_eq!(p, Ok(Params::new(5, 10, 100).unwrap()));
    }

    #[test]
    fn falta_un_obligatorio() {
        let e = parse(&json!({"days_per_week": 7, "days_per_year": 365})).unwrap_err();
        assert_eq!(e, ["days_per_month: falta el parámetro obligatorio"]);
    }

    #[test]
    fn tipo_equivocado_y_no_enteros() {
        for bad in [
            json!("30"),
            json!(30.5),
            json!(-1),
            json!(null),
            json!(true),
        ] {
            let e =
                parse(&json!({"days_per_week": 7, "days_per_month": bad, "days_per_year": 365}))
                    .unwrap_err();
            assert_eq!(e.len(), 1, "{bad}");
            assert!(
                e[0].starts_with("days_per_month: se esperaba un entero no negativo"),
                "{e:?}"
            );
        }
    }

    #[test]
    fn campo_desconocido() {
        let e = parse(&json!({"days_per_week": 7, "days_per_month": 30, "days_per_year": 365, "days_per_mes": 1}))
            .unwrap_err();
        assert_eq!(e.len(), 1);
        assert!(e[0].starts_with("days_per_mes: campo desconocido"), "{e:?}");
    }

    #[test]
    fn rangos_y_orden() {
        assert!(Params::new(0, 30, 365).is_err());
        assert!(Params::new(7, 0, 365).is_err());
        assert!(Params::new(7, 30, 0).is_err());
        assert!(Params::new(7, 30, Params::MAX_DAYS + 1).is_err());
        assert!(Params::new(31, 30, 365).is_err());
        assert!(Params::new(7, 366, 365).is_err());
        assert!(Params::new(30, 30, 30).is_ok());
        assert!(Params::new(7, 25, 100).is_ok()); // el mes no tiene que dividir al año
    }

    #[test]
    fn el_anio_tiene_tope_propio_de_u16() {
        assert_eq!(Params::MAX_DAYS_PER_YEAR, 65_535);
        assert!(Params::new(7, 30, 65_535).is_ok());
        let error = Params::new(7, 30, 65_536).unwrap_err();
        assert_eq!(
            error.to_string(),
            "days_per_year: fuera de rango: 65536 (debe estar entre 1 y 65535)"
        );
        // Semana y mes quedan acotados por el orden: no pueden superar al año.
        assert!(Params::new(7, 65_536, 65_535).is_err());
    }

    #[test]
    fn acumula_errores() {
        let e = parse(&json!({"days_per_week": 0, "days_per_month": 0, "days_per_year": 365}))
            .unwrap_err();
        assert_eq!(e.len(), 2, "{e:?}");
        let e = parse(&json!({"x": 1})).unwrap_err();
        assert_eq!(e.len(), 4, "{e:?}"); // 1 desconocido + 3 faltantes
    }
}
