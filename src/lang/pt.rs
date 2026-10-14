//! Portuguese: so far month names.

use super::{Lang, Meaning::*};

pub const LANG: Lang = Lang {
    code: "pt",
    name: "Português",
    label: "data em português",
    year_offset: 0,
    words: &[
        ("janeiro", MonthName(1)),
        ("fevereiro", MonthName(2)),
        ("março", MonthName(3)),
        ("abril", MonthName(4)),
        ("maio", MonthName(5)),
        ("junho", MonthName(6)),
        ("julho", MonthName(7)),
        ("agosto", MonthName(8)),
        ("setembro", MonthName(9)),
        ("outubro", MonthName(10)),
        ("novembro", MonthName(11)),
        ("dezembro", MonthName(12)),
    ],
    weak: &[],
};

#[cfg(test)]
mod tests {
    use crate::testing::{assert_no_dates, check_dates};

    #[test]
    fn portuguese() {
        check_dates(&[("13 outubro 2026", "2026-10-13 00:00:00+02:00")]);
    }

    #[test]
    fn prose_without_dates() {
        assert_no_dates(&["Em maio vamos à praia, se o tempo deixar."]);
    }
}
