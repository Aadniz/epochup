//! Italian: so far month names.

use super::{Lang, Meaning::*};

pub const LANG: Lang = Lang {
    code: "it",
    name: "Italiano",
    label: "data in italiano",
    year_offset: 0,
    words: &[
        ("gennaio", MonthName(1)),
        ("febbraio", MonthName(2)),
        ("marzo", MonthName(3)),
        ("aprile", MonthName(4)),
        ("maggio", MonthName(5)),
        ("giugno", MonthName(6)),
        ("luglio", MonthName(7)),
        ("agosto", MonthName(8)),
        ("settembre", MonthName(9)),
        ("ottobre", MonthName(10)),
        ("novembre", MonthName(11)),
        ("dicembre", MonthName(12)),
    ],
    weak: &[],
};

#[cfg(test)]
mod tests {
    use crate::testing::{assert_no_dates, check_dates};

    #[test]
    fn italian() {
        check_dates(&[("13 ottobre 2026", "2026-10-13 00:00:00+02:00")]);
    }

    #[test]
    fn prose_without_dates() {
        assert_no_dates(&["A maggio andiamo al mare, se il tempo lo permette."]);
    }
}
