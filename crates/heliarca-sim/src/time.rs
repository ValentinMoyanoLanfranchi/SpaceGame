//! Tiempo de simulación y calendario simple (docs 01 y 22 §0).
//!
//! El tiempo es un contador entero de segundos ([`SimTime`]); nunca se consulta el reloj
//! del sistema (`std::time` está prohibido por `clippy.toml`), de modo que la simulación
//! es determinista. El calendario es de año fijo de 365 días, sin bisiestos ni meses.
//!
//! Convenciones: los años y los días se numeran desde 1 («Año 1, día 1»); la hora, el
//! minuto y el segundo, desde 0. Toda la aritmética es explícita sobre desbordes: devuelve
//! `Option`/`Result` o satura (métodos `saturating_*`), y nunca entra en pánico.

use std::fmt;

/// Segundos por minuto.
pub const SECONDS_PER_MINUTE: u64 = 60;
/// Minutos por hora.
pub const MINUTES_PER_HOUR: u64 = 60;
/// Segundos por hora.
pub const SECONDS_PER_HOUR: u64 = SECONDS_PER_MINUTE * MINUTES_PER_HOUR;
/// Horas por día.
pub const HOURS_PER_DAY: u64 = 24;
/// Segundos por día.
pub const SECONDS_PER_DAY: u64 = SECONDS_PER_HOUR * HOURS_PER_DAY;
/// Días por año (calendario simple: año fijo de 365 días, doc 22 §0).
pub const DAYS_PER_YEAR: u64 = 365;
/// Segundos por año.
pub const SECONDS_PER_YEAR: u64 = SECONDS_PER_DAY * DAYS_PER_YEAR;

/// Primer año del calendario.
pub const FIRST_YEAR: u64 = 1;
/// Primer día de cada año.
pub const FIRST_DAY: u64 = 1;

/// Instante de simulación: segundos transcurridos desde el inicio de la partida.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SimTime(u64);

impl SimTime {
    /// Inicio de la simulación (0 s).
    pub const ZERO: Self = Self(0);
    /// Mayor instante representable.
    pub const MAX: Self = Self(u64::MAX);

    /// Crea un instante a partir de segundos.
    #[must_use]
    pub const fn from_seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    /// Crea un instante a partir de días enteros; `None` si desborda.
    #[must_use]
    pub const fn from_days(days: u64) -> Option<Self> {
        match days.checked_mul(SECONDS_PER_DAY) {
            Some(s) => Some(Self(s)),
            None => None,
        }
    }

    /// Segundos desde el inicio de la simulación.
    #[must_use]
    pub const fn as_seconds(self) -> u64 {
        self.0
    }

    /// Suma segundos; `None` si desborda.
    #[must_use]
    pub const fn checked_add_seconds(self, seconds: u64) -> Option<Self> {
        match self.0.checked_add(seconds) {
            Some(s) => Some(Self(s)),
            None => None,
        }
    }

    /// Suma segundos saturando en [`SimTime::MAX`].
    #[must_use]
    pub const fn saturating_add_seconds(self, seconds: u64) -> Self {
        Self(self.0.saturating_add(seconds))
    }

    /// Avanza un incremento; `None` si desborda.
    #[must_use]
    pub const fn checked_advance(self, increment: Increment) -> Option<Self> {
        self.checked_add_seconds(increment.seconds())
    }

    /// Avanza un incremento saturando en [`SimTime::MAX`].
    #[must_use]
    pub const fn saturating_advance(self, increment: Increment) -> Self {
        self.saturating_add_seconds(increment.seconds())
    }

    /// Segundos transcurridos desde `earlier`; `None` si `earlier` es posterior.
    #[must_use]
    pub const fn checked_seconds_since(self, earlier: Self) -> Option<u64> {
        self.0.checked_sub(earlier.0)
    }
}

/// Incremento de tiempo elegible por el jugador (doc 01 «Tiempo, incrementos y ticks»).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Increment {
    /// 5 segundos.
    S5,
    /// 1 hora.
    H1,
    /// 1 día.
    D1,
    /// 5 días.
    D5,
    /// 30 días.
    D30,
}

impl Increment {
    /// Todos los incrementos, de menor a mayor.
    pub const ALL: [Self; 5] = [Self::S5, Self::H1, Self::D1, Self::D5, Self::D30];

    /// Duración del incremento en segundos.
    #[must_use]
    pub const fn seconds(self) -> u64 {
        match self {
            Self::S5 => 5,
            Self::H1 => SECONDS_PER_HOUR,
            Self::D1 => SECONDS_PER_DAY,
            Self::D5 => 5 * SECONDS_PER_DAY,
            Self::D30 => 30 * SECONDS_PER_DAY,
        }
    }
}

/// Fecha de calendario («Año 1, día 1, 00:00:00» es el origen por defecto).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalendarDate {
    /// Año, desde 1.
    pub year: u64,
    /// Día del año, en `1..=365`.
    pub day: u16,
    /// Hora, en `0..24`.
    pub hour: u8,
    /// Minuto, en `0..60`.
    pub minute: u8,
    /// Segundo, en `0..60`.
    pub second: u8,
}

impl CalendarDate {
    /// «Año 1, día 1, 00:00:00».
    pub const ORIGIN: Self = Self {
        year: 1,
        day: 1,
        hour: 0,
        minute: 0,
        second: 0,
    };

    /// Fecha a medianoche del día indicado (sin validar; ver [`Calendar::new`]).
    #[must_use]
    pub const fn at_day_start(year: u64, day: u16) -> Self {
        Self {
            year,
            day,
            hour: 0,
            minute: 0,
            second: 0,
        }
    }
}

impl fmt::Display for CalendarDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Año {}, día {}, {:02}:{:02}:{:02}",
            self.year, self.day, self.hour, self.minute, self.second
        )
    }
}

/// Errores de calendario.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarError {
    /// La fecha no existe en el calendario (año 0, día fuera de 1..=365, hora fuera de rango...).
    InvalidDate,
    /// La fecha es anterior al inicio de la simulación.
    BeforeStart,
    /// El resultado no cabe en el rango representable.
    Overflow,
}

impl fmt::Display for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidDate => "fecha de calendario inválida",
            Self::BeforeStart => "la fecha es anterior al inicio de la simulación",
            Self::Overflow => "desborde al convertir tiempo y fecha",
        })
    }
}

impl std::error::Error for CalendarError {}

/// Calendario simple: asocia `SimTime = 0` con una fecha de inicio configurable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Calendar {
    start: CalendarDate,
    /// Segundos desde «Año 1, día 1, 00:00:00» hasta `start` (precalculado).
    start_offset: u64,
}

impl Default for Calendar {
    fn default() -> Self {
        Self {
            start: CalendarDate::ORIGIN,
            start_offset: 0,
        }
    }
}

impl Calendar {
    /// Crea un calendario cuyo `SimTime = 0` corresponde a `start`.
    ///
    /// # Errors
    /// [`CalendarError::InvalidDate`] si `start` no es una fecha válida;
    /// [`CalendarError::Overflow`] si su desplazamiento no cabe en `u64`.
    pub fn new(start: CalendarDate) -> Result<Self, CalendarError> {
        let start_offset = absolute_seconds(start)?;
        Ok(Self {
            start,
            start_offset,
        })
    }

    /// Fecha de inicio (la de `SimTime = 0`).
    #[must_use]
    pub const fn start(&self) -> CalendarDate {
        self.start
    }

    /// Fecha correspondiente a un instante.
    ///
    /// # Errors
    /// [`CalendarError::Overflow`] si inicio + tiempo no cabe en `u64` segundos.
    pub fn date_at(&self, time: SimTime) -> Result<CalendarDate, CalendarError> {
        let abs = self
            .start_offset
            .checked_add(time.as_seconds())
            .ok_or(CalendarError::Overflow)?;
        Ok(date_from_absolute(abs))
    }

    /// Instante correspondiente a una fecha (inversa de [`Calendar::date_at`]).
    ///
    /// # Errors
    /// [`CalendarError::InvalidDate`] si la fecha no es válida;
    /// [`CalendarError::BeforeStart`] si es anterior al inicio;
    /// [`CalendarError::Overflow`] si no cabe en `u64`.
    pub fn time_of(&self, date: CalendarDate) -> Result<SimTime, CalendarError> {
        let abs = absolute_seconds(date)?;
        abs.checked_sub(self.start_offset)
            .map(SimTime::from_seconds)
            .ok_or(CalendarError::BeforeStart)
    }
}

/// Segundos desde «Año 1, día 1, 00:00:00» hasta `date`.
fn absolute_seconds(date: CalendarDate) -> Result<u64, CalendarError> {
    let day = u64::from(date.day);
    if date.year < FIRST_YEAR
        || !(FIRST_DAY..=DAYS_PER_YEAR).contains(&day)
        || u64::from(date.hour) >= HOURS_PER_DAY
        || u64::from(date.minute) >= MINUTES_PER_HOUR
        || u64::from(date.second) >= SECONDS_PER_MINUTE
    {
        return Err(CalendarError::InvalidDate);
    }
    let days = (date.year - FIRST_YEAR)
        .checked_mul(DAYS_PER_YEAR)
        .and_then(|d| d.checked_add(day - FIRST_DAY))
        .ok_or(CalendarError::Overflow)?;
    days.checked_mul(SECONDS_PER_DAY)
        .and_then(|s| {
            s.checked_add(
                u64::from(date.hour) * SECONDS_PER_HOUR
                    + u64::from(date.minute) * SECONDS_PER_MINUTE
                    + u64::from(date.second),
            )
        })
        .ok_or(CalendarError::Overflow)
}

/// Fecha para un total de segundos desde «Año 1, día 1, 00:00:00». Total: no puede fallar,
/// porque `u64::MAX` segundos son ~5,8e11 años, que caben en `u64`.
#[allow(clippy::cast_possible_truncation)] // Cada resto es menor que el tope de su tipo destino.
fn date_from_absolute(abs: u64) -> CalendarDate {
    let total_days = abs / SECONDS_PER_DAY;
    let in_day = abs % SECONDS_PER_DAY;
    CalendarDate {
        year: FIRST_YEAR + total_days / DAYS_PER_YEAR,
        day: (FIRST_DAY + total_days % DAYS_PER_YEAR) as u16,
        hour: (in_day / SECONDS_PER_HOUR) as u8,
        minute: (in_day % SECONDS_PER_HOUR / SECONDS_PER_MINUTE) as u8,
        second: (in_day % SECONDS_PER_MINUTE) as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cal() -> Calendar {
        Calendar::default()
    }

    fn d(year: u64, day: u16, hour: u8, minute: u8, second: u8) -> CalendarDate {
        CalendarDate {
            year,
            day,
            hour,
            minute,
            second,
        }
    }

    #[test]
    fn constantes_coherentes() {
        assert_eq!(SECONDS_PER_DAY, 86_400);
        assert_eq!(SECONDS_PER_YEAR, 31_536_000);
    }

    #[test]
    fn tiempo_cero_es_anio_1_dia_1() {
        let date = cal().date_at(SimTime::ZERO).unwrap();
        assert_eq!(date, CalendarDate::ORIGIN);
        assert_eq!(date.to_string(), "Año 1, día 1, 00:00:00");
    }

    #[test]
    fn tras_365_dias_es_anio_2_dia_1() {
        let t = SimTime::from_days(365).unwrap();
        assert_eq!(cal().date_at(t).unwrap(), CalendarDate::at_day_start(2, 1));
    }

    #[test]
    fn incremento_s5() {
        assert_eq!(Increment::S5.seconds(), 5);
        let t = SimTime::ZERO.checked_advance(Increment::S5).unwrap();
        assert_eq!(t.as_seconds(), 5);
        assert_eq!(cal().date_at(t).unwrap(), d(1, 1, 0, 0, 5));
    }

    #[test]
    fn incremento_h1() {
        assert_eq!(Increment::H1.seconds(), 3_600);
        let t = SimTime::ZERO.checked_advance(Increment::H1).unwrap();
        assert_eq!(cal().date_at(t).unwrap(), d(1, 1, 1, 0, 0));
    }

    #[test]
    fn incremento_d1() {
        assert_eq!(Increment::D1.seconds(), 86_400);
        let t = SimTime::ZERO.checked_advance(Increment::D1).unwrap();
        assert_eq!(cal().date_at(t).unwrap(), CalendarDate::at_day_start(1, 2));
    }

    #[test]
    fn incremento_d5() {
        assert_eq!(Increment::D5.seconds(), 5 * 86_400);
        let t = SimTime::ZERO.checked_advance(Increment::D5).unwrap();
        assert_eq!(cal().date_at(t).unwrap(), CalendarDate::at_day_start(1, 6));
    }

    #[test]
    fn incremento_d30() {
        assert_eq!(Increment::D30.seconds(), 30 * 86_400);
        let t = SimTime::ZERO.checked_advance(Increment::D30).unwrap();
        assert_eq!(cal().date_at(t).unwrap(), CalendarDate::at_day_start(1, 31));
    }

    #[test]
    fn incrementos_estan_ordenados_y_son_cinco() {
        let secs: Vec<u64> = Increment::ALL.iter().map(|i| i.seconds()).collect();
        assert_eq!(secs.len(), 5);
        assert!(secs.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn ultimo_segundo_del_dia_y_del_anio() {
        let last_of_day = SimTime::from_seconds(SECONDS_PER_DAY - 1);
        assert_eq!(cal().date_at(last_of_day).unwrap(), d(1, 1, 23, 59, 59));
        let last_of_year = SimTime::from_seconds(SECONDS_PER_YEAR - 1);
        assert_eq!(cal().date_at(last_of_year).unwrap(), d(1, 365, 23, 59, 59));
        let next = SimTime::from_seconds(SECONDS_PER_YEAR);
        assert_eq!(
            cal().date_at(next).unwrap(),
            CalendarDate::at_day_start(2, 1)
        );
    }

    #[test]
    fn fecha_de_inicio_configurable() {
        let c = Calendar::new(d(3, 200, 6, 30, 0)).unwrap();
        assert_eq!(c.start(), d(3, 200, 6, 30, 0));
        assert_eq!(c.date_at(SimTime::ZERO).unwrap(), d(3, 200, 6, 30, 0));
        // 165 días + 17:30 más tarde se cruza al año 4, día 1, 00:00.
        let t = SimTime::from_seconds(165 * SECONDS_PER_DAY + 17 * SECONDS_PER_HOUR + 1_800);
        assert_eq!(c.date_at(t).unwrap(), CalendarDate::at_day_start(4, 1));
    }

    #[test]
    fn fechas_de_inicio_invalidas() {
        for bad in [
            d(0, 1, 0, 0, 0),
            d(1, 0, 0, 0, 0),
            d(1, 366, 0, 0, 0),
            d(1, 1, 24, 0, 0),
            d(1, 1, 0, 60, 0),
            d(1, 1, 0, 0, 60),
        ] {
            assert_eq!(Calendar::new(bad), Err(CalendarError::InvalidDate));
        }
    }

    #[test]
    fn inicio_que_desborda() {
        assert_eq!(
            Calendar::new(CalendarDate::at_day_start(u64::MAX, 1)),
            Err(CalendarError::Overflow)
        );
    }

    #[test]
    fn time_of_antes_del_inicio() {
        let c = Calendar::new(d(2, 1, 0, 0, 0)).unwrap();
        assert_eq!(
            c.time_of(d(1, 365, 23, 59, 59)),
            Err(CalendarError::BeforeStart)
        );
        assert_eq!(c.time_of(d(2, 1, 0, 0, 0)), Ok(SimTime::ZERO));
        assert_eq!(
            c.time_of(d(2, 2, 0, 0, 0)),
            Ok(SimTime::from_seconds(SECONDS_PER_DAY))
        );
    }

    #[test]
    fn aritmetica_sin_overflow_silencioso() {
        assert_eq!(SimTime::MAX.checked_advance(Increment::S5), None);
        assert_eq!(
            SimTime::MAX.saturating_advance(Increment::D30),
            SimTime::MAX
        );
        assert_eq!(SimTime::from_days(u64::MAX), None);
        assert_eq!(
            SimTime::from_seconds(10).checked_seconds_since(SimTime::from_seconds(4)),
            Some(6)
        );
        assert_eq!(
            SimTime::from_seconds(4).checked_seconds_since(SimTime::from_seconds(10)),
            None
        );
    }

    #[test]
    fn date_at_en_el_maximo_representable() {
        // Sin inicio desplazado, u64::MAX segundos es una fecha válida (no entra en pánico).
        let date = cal().date_at(SimTime::MAX).unwrap();
        assert!((1..=365).contains(&date.day));
        // Con inicio desplazado, sumar MAX desborda y se informa.
        let c = Calendar::new(d(1, 2, 0, 0, 0)).unwrap();
        assert_eq!(c.date_at(SimTime::MAX), Err(CalendarError::Overflow));
    }

    #[test]
    fn ida_y_vuelta_tiempo_fecha() {
        let c = Calendar::new(d(5, 123, 7, 8, 9)).unwrap();
        let mut s: u64 = 0;
        while s < 3 * SECONDS_PER_YEAR {
            let t = SimTime::from_seconds(s);
            assert_eq!(c.time_of(c.date_at(t).unwrap()), Ok(t));
            s += 7_919 * 13; // paso primo para recorrer horas, minutos y segundos variados
        }
        let t = SimTime::MAX.saturating_add_seconds(0);
        let c0 = cal();
        assert_eq!(c0.time_of(c0.date_at(t).unwrap()), Ok(t));
    }
}
