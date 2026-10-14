//! French: so far month and weekday names.

use jiff::civil::Weekday::*;

use super::{Lang, Meaning::*};

pub const LANG: Lang = Lang {
    code: "fr",
    name: "Français",
    label: "date en français",
    year_offset: 0,
    words: &[
        ("janvier", MonthName(1)),
        ("février", MonthName(2)),
        ("fevrier", MonthName(2)),
        ("avril", MonthName(4)),
        ("juin", MonthName(6)),
        ("juillet", MonthName(7)),
        ("août", MonthName(8)),
        ("aout", MonthName(8)),
        ("septembre", MonthName(9)),
        ("octobre", MonthName(10)),
        ("novembre", MonthName(11)),
        ("décembre", MonthName(12)),
        ("decembre", MonthName(12)),
        ("lundi", WeekdayName(Monday)),
        ("mardi", WeekdayName(Tuesday)),
        ("mercredi", WeekdayName(Wednesday)),
        ("jeudi", WeekdayName(Thursday)),
        ("vendredi", WeekdayName(Friday)),
        ("samedi", WeekdayName(Saturday)),
        ("dimanche", WeekdayName(Sunday)),
    ],
    weak: &[("mars", MonthName(3)), ("mai", MonthName(5))],
};

#[cfg(test)]
mod tests {
    use crate::testing::{assert_no_dates, check_dates};

    #[test]
    fn french() {
        check_dates(&[
            ("13 octobre 2026", "2026-10-13 00:00:00+02:00"),
            (
                "vendredi 16 octobre 2026 15:00",
                "2026-10-16 15:00:00+02:00",
            ),
            ("3 mars 2026", "2026-03-03 00:00:00+01:00"),
        ]);
    }

    #[test]
    fn prose_without_dates() {
        assert_no_dates(&["Il est venu en mars pour voir la planète Mars, mais il pleuvait."]);
    }
}
