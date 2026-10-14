//! Dutch: so far month names.

use super::{Lang, Meaning::*};

pub const LANG: Lang = Lang {
    code: "nl",
    name: "Nederlands",
    label: "datum in het Nederlands",
    year_offset: 0,
    words: &[
        ("januari", MonthName(1)),
        ("februari", MonthName(2)),
        ("maart", MonthName(3)),
        ("april", MonthName(4)),
        ("juni", MonthName(6)),
        ("juli", MonthName(7)),
        ("augustus", MonthName(8)),
        ("september", MonthName(9)),
        ("oktober", MonthName(10)),
        ("november", MonthName(11)),
        ("december", MonthName(12)),
    ],
    weak: &[("mei", MonthName(5))],
};

#[cfg(test)]
mod tests {
    use crate::testing::{assert_no_dates, check_dates};

    #[test]
    fn dutch() {
        check_dates(&[("13 oktober 2026", "2026-10-13 00:00:00+02:00")]);
    }

    #[test]
    fn prose_without_dates() {
        assert_no_dates(&["In mei gaan we naar zee, als het weer meezit."]);
    }
}
