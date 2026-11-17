//! Escenario: la descripción de partida con la que se crea un [`Sim`](crate::Sim).
//!
//! **Mínimo a propósito.** Hoy un [`Scenario`] sólo lleva lo que el núcleo ya sabe interpretar:
//! el [`Calendar`] (qué fecha corresponde a `SimTime = 0`) y los [`Params`] globales
//! (`data/params.json`; por defecto, los valores del GDD). **No** tiene estado del mundo
//! (cuerpos, bienes, tecnologías, naves...): eso llega con la carga de datos de M0.3+ (doc 22),
//! que extenderá este tipo con más campos y sus constructores. Por eso se construye con
//! [`Scenario::new`] / [`Scenario::default`] y el patrón *builder* (`with_*`) y sus campos son
//! privados: agregar campos no rompe a quien lo usa.

use crate::data::GameData;
use crate::params::Params;
use crate::time::Calendar;

/// Descripción de una partida: lo que, junto con la semilla, determina por completo su inicio.
///
/// # Ejemplo
///
/// ```
/// use heliarca_sim::{Calendar, CalendarDate, Scenario};
///
/// let calendario = Calendar::new(CalendarDate::at_day_start(5, 100))?;
/// let escenario = Scenario::new().with_calendar(calendario);
/// assert_eq!(escenario.calendar().start().year, 5);
/// # Ok::<(), heliarca_sim::CalendarError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Scenario {
    calendar: Calendar,
    params: Params,
}

impl Scenario {
    /// Escenario por defecto: calendario que empieza en «Año 1, día 1, 00:00:00».
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reemplaza el calendario (la fecha de `SimTime = 0`).
    #[must_use]
    pub const fn with_calendar(mut self, calendar: Calendar) -> Self {
        self.calendar = calendar;
        self
    }

    /// Reemplaza los parámetros globales.
    #[must_use]
    pub const fn with_params(mut self, params: Params) -> Self {
        self.params = params;
        self
    }

    /// Escenario con los parámetros de unos datos ya cargados (`data/params.json`); el resto
    /// (calendario...) es el de [`Scenario::new`].
    #[must_use]
    pub fn from_game_data(data: &GameData) -> Self {
        Self {
            params: data.params,
            ..Self::default()
        }
    }

    /// Parámetros globales del escenario.
    #[must_use]
    pub const fn params(&self) -> Params {
        self.params
    }

    /// Calendario del escenario.
    #[must_use]
    pub const fn calendar(&self) -> Calendar {
        self.calendar
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::CalendarDate;

    #[test]
    fn por_defecto_empieza_en_el_origen() {
        assert_eq!(Scenario::new().calendar().start(), CalendarDate::ORIGIN);
        assert_eq!(Scenario::new(), Scenario::default());
    }

    #[test]
    fn los_params_por_defecto_son_los_del_gdd_y_se_reemplazan() {
        assert_eq!(Scenario::new().params(), Params::default());
        let p = Params::new(7, 10, 365).unwrap();
        assert_eq!(Scenario::new().with_params(p).params(), p);
        let data = GameData {
            params: p,
            ..GameData::default()
        };
        assert_eq!(Scenario::from_game_data(&data).params(), p);
        assert_eq!(
            Scenario::from_game_data(&data).calendar(),
            Scenario::new().calendar()
        );
    }

    #[test]
    fn with_calendar_reemplaza_el_calendario() {
        let calendar = Calendar::new(CalendarDate::at_day_start(3, 10)).unwrap();
        assert_eq!(Scenario::new().with_calendar(calendar).calendar(), calendar);
    }
}
