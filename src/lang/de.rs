//! German.

use jiff::civil::Weekday::*;

use super::{Lang, Meaning::*, Unit::*};

pub const LANG: Lang = Lang {
    code: "de",
    name: "Deutsch",
    label: "Datum auf Deutsch",
    year_offset: 0,
    words: &[
        ("januar", MonthName(1)),
        ("jänner", MonthName(1)),
        ("jän", MonthName(1)),
        ("februar", MonthName(2)),
        ("feb", MonthName(2)),
        ("märz", MonthName(3)),
        ("maerz", MonthName(3)),
        ("mär", MonthName(3)),
        ("mrz", MonthName(3)),
        ("april", MonthName(4)),
        ("apr", MonthName(4)),
        ("juni", MonthName(6)),
        ("jun", MonthName(6)),
        ("juli", MonthName(7)),
        ("jul", MonthName(7)),
        ("august", MonthName(8)),
        ("september", MonthName(9)),
        ("sep", MonthName(9)),
        ("oktober", MonthName(10)),
        ("okt", MonthName(10)),
        ("november", MonthName(11)),
        ("nov", MonthName(11)),
        ("dezember", MonthName(12)),
        ("dez", MonthName(12)),
        ("montag", WeekdayName(Monday)),
        ("dienstag", WeekdayName(Tuesday)),
        ("mittwoch", WeekdayName(Wednesday)),
        ("donnerstag", WeekdayName(Thursday)),
        ("freitag", WeekdayName(Friday)),
        ("samstag", WeekdayName(Saturday)),
        ("sonnabend", WeekdayName(Saturday)),
        ("sonntag", WeekdayName(Sunday)),
        ("sek", UnitName(Second)),
        ("sekunde", UnitName(Second)),
        ("sekunden", UnitName(Second)),
        ("minuten", UnitName(Minute)),
        ("std", UnitName(Hour)),
        ("stunde", UnitName(Hour)),
        ("stunden", UnitName(Hour)),
        ("tag", UnitName(Day)),
        ("tage", UnitName(Day)),
        ("tagen", UnitName(Day)),
        ("woche", UnitName(Week)),
        ("wochen", UnitName(Week)),
        ("monat", UnitName(Month)),
        ("monate", UnitName(Month)),
        ("monaten", UnitName(Month)),
        ("jahr", UnitName(Year)),
        ("jahre", UnitName(Year)),
        ("jahren", UnitName(Year)),
        ("heute", DayOffset(0)),
        ("gestern", DayOffset(-1)),
        ("übermorgen", DayOffset(2)),
        ("uebermorgen", DayOffset(2)),
        ("vorgestern", DayOffset(-2)),
        ("in", In),
        ("vor", Before),
        ("ein", Count(1)),
        ("einem", Count(1)),
        ("einer", Count(1)),
        ("einen", Count(1)),
        ("und", And),
        ("nächste", Next(1)),
        ("nächsten", Next(1)),
        ("nächster", Next(1)),
        ("nächstes", Next(1)),
        ("kommende", Next(1)),
        ("kommenden", Next(1)),
        ("letzte", Next(-1)),
        ("letzten", Next(-1)),
        ("letzter", Next(-1)),
        ("letztes", Next(-1)),
        ("vorige", Next(-1)),
        ("vorigen", Next(-1)),
        ("diese", Next(0)),
        ("diesen", Next(0)),
        ("dieser", Next(0)),
        ("dieses", Next(0)),
        ("übernächste", Next(2)),
        ("übernächsten", Next(2)),
        ("mittag", Clock(12)),
        ("mittags", Clock(12)),
        ("mitternacht", Clock(0)),
        ("uhr", OClock),
        ("um", At),
        ("gegen", At),
        ("kw", WeekNumber),
        ("woche", WeekNumber),
        ("um", Filler),
        ("am", Filler),
        ("den", Filler),
        ("der", Filler),
        ("dem", Filler),
        ("und", Filler),
    ],
    weak: &[
        ("jan", MonthName(1)),
        ("mai", MonthName(5)),
        ("aug", MonthName(8)),
        ("mo", WeekdayName(Monday)),
        ("di", WeekdayName(Tuesday)),
        ("mi", WeekdayName(Wednesday)),
        ("do", WeekdayName(Thursday)),
        ("fr", WeekdayName(Friday)),
        ("sa", WeekdayName(Saturday)),
        ("so", WeekdayName(Sunday)),
        // "morgen" is also "morning": `heute morgen`, `Guten Morgen`.
        ("morgen", DayOffset(1)),
        ("jetzt", Now),
    ],
};

#[cfg(test)]
mod tests {
    use crate::testing::{assert_no_dates, check_dates};

    #[test]
    fn german() {
        check_dates(&[
            ("13. Oktober 2026", "2026-10-13 00:00:00+02:00"),
            ("Di, 13. Okt. 2026", "2026-10-13 00:00:00+02:00"),
            ("morgen um 15 Uhr", "2026-10-14 15:00:00+02:00"),
            ("21.30 Uhr", "2026-10-13 21:30:00+02:00"),
            ("21h30", "2026-10-13 21:30:00+02:00"),
            ("übermorgen", "2026-10-15 00:00:00+02:00"),
            ("vor 3 Tagen", "2026-10-10 21:24:16+02:00"),
            ("in 2 Stunden", "2026-10-13 23:24:16+02:00"),
            ("nächsten Freitag", "2026-10-16 00:00:00+02:00"),
            ("KW 42", "2026-10-12 00:00:00+02:00"),
            ("März 3", "2027-03-03 00:00:00+01:00"),
            ("13.10.2026 15:00 MESZ", "2026-10-13 15:00:00+02:00"),
        ]);
    }

    #[test]
    fn in_text() {
        check_dates(&[
            (
                "Am 13. Oktober um 15 Uhr ist das Treffen",
                "2026-10-13 15:00:00+02:00",
            ),
            (
                "Das Meeting um 21:24 ist wichtig",
                "2026-10-13 21:24:00+02:00",
            ),
        ]);
    }

    #[test]
    fn prose_without_dates() {
        assert_no_dates(&[
            "Guten Morgen! Das ist so, und so bleibt es: wir sind jetzt da, wo die Mi-Linie endet.",
            "Do it yourself, sagte er, und dann gingen wir in den Wald, der vor uns lag.",
            // Polish `sie` (August) and `o` (at)
            "Sie haben recht, sagte sie, o weh, das wird teuer.",
        ]);
    }
}
