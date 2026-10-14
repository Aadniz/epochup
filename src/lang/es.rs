//! Spanish: so far month and weekday names.

use jiff::civil::Weekday::*;

use super::{Lang, Meaning::*};

pub const LANG: Lang = Lang {
    code: "es",
    name: "Español",
    label: "fecha en español",
    year_offset: 0,
    words: &[
        ("enero", MonthName(1)),
        ("febrero", MonthName(2)),
        ("marzo", MonthName(3)),
        ("abril", MonthName(4)),
        ("mayo", MonthName(5)),
        ("junio", MonthName(6)),
        ("julio", MonthName(7)),
        ("agosto", MonthName(8)),
        ("septiembre", MonthName(9)),
        ("setiembre", MonthName(9)),
        ("octubre", MonthName(10)),
        ("noviembre", MonthName(11)),
        ("diciembre", MonthName(12)),
        ("lunes", WeekdayName(Monday)),
        ("martes", WeekdayName(Tuesday)),
        ("miércoles", WeekdayName(Wednesday)),
        ("miercoles", WeekdayName(Wednesday)),
        ("jueves", WeekdayName(Thursday)),
        ("viernes", WeekdayName(Friday)),
        ("sábado", WeekdayName(Saturday)),
        ("sabado", WeekdayName(Saturday)),
        ("domingo", WeekdayName(Sunday)),
    ],
    weak: &[],
};

#[cfg(test)]
mod tests {
    use crate::testing::{assert_no_dates, check_dates};

    #[test]
    fn spanish() {
        check_dates(&[
            ("13 octubre 2026", "2026-10-13 00:00:00+02:00"),
            ("martes, 13 octubre 2026 21:24", "2026-10-13 21:24:00+02:00"),
        ]);
    }

    #[test]
    fn prose_without_dates() {
        assert_no_dates(&["Me gusta el mar, y en mayo hace buen tiempo en la costa."]);
    }
}
