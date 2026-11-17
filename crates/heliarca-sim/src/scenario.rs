//! Escenario: la descripción de partida con la que se crea un [`Sim`](crate::Sim).
//!
//! **Mínimo a propósito.** Hoy un [`Scenario`] sólo lleva lo que el núcleo ya sabe interpretar:
//! el [`Calendar`] (qué fecha corresponde a `SimTime = 0`). **No** tiene estado del mundo
//! (cuerpos, bienes, tecnologías, naves...): eso llega con la carga de datos de M0.3+ (doc 22),
//! que extenderá este tipo con más campos y sus constructores. Por eso se construye con
//! [`Scenario::new`] / [`Scenario::default`] y el patrón *builder* (`with_*`) y sus campos son
//! privados: agregar campos no rompe a quien lo usa.

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
    fn with_calendar_reemplaza_el_calendario() {
        let calendar = Calendar::new(CalendarDate::at_day_start(3, 10)).unwrap();
        assert_eq!(Scenario::new().with_calendar(calendar).calendar(), calendar);
    }
}
