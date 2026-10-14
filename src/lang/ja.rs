//! Japanese.

use jiff::civil::Weekday::*;

use super::{Lang, Meaning::*, Period::*, Unit::*};

pub const LANG: Lang = Lang {
    code: "ja",
    name: "日本語",
    label: "日本語の日付",
    year_offset: 0,
    words: &[
        ("月曜日", WeekdayName(Monday)),
        ("月曜", WeekdayName(Monday)),
        ("火曜日", WeekdayName(Tuesday)),
        ("火曜", WeekdayName(Tuesday)),
        ("水曜日", WeekdayName(Wednesday)),
        ("水曜", WeekdayName(Wednesday)),
        ("木曜日", WeekdayName(Thursday)),
        ("木曜", WeekdayName(Thursday)),
        ("金曜日", WeekdayName(Friday)),
        ("金曜", WeekdayName(Friday)),
        ("土曜日", WeekdayName(Saturday)),
        ("土曜", WeekdayName(Saturday)),
        ("日曜日", WeekdayName(Sunday)),
        ("日曜", WeekdayName(Sunday)),
        ("月", ParenWeekday(Monday)),
        ("火", ParenWeekday(Tuesday)),
        ("水", ParenWeekday(Wednesday)),
        ("木", ParenWeekday(Thursday)),
        ("金", ParenWeekday(Friday)),
        ("土", ParenWeekday(Saturday)),
        ("日", ParenWeekday(Sunday)),
        ("秒", UnitName(Second)),
        ("分", UnitName(Minute)),
        ("時間", UnitName(Hour)),
        ("日", UnitName(Day)),
        ("週", UnitName(Week)),
        ("週間", UnitName(Week)),
        ("ヶ月", UnitName(Month)),
        ("か月", UnitName(Month)),
        ("カ月", UnitName(Month)),
        ("ケ月", UnitName(Month)),
        ("箇月", UnitName(Month)),
        ("年", UnitName(Year)),
        ("年", YearMark),
        ("月", MonthMark),
        ("日", DayMark),
        ("時", HourMark),
        ("分", MinuteMark),
        ("秒", SecondMark),
        ("半", Half),
        ("今日", DayOffset(0)),
        ("本日", DayOffset(0)),
        ("きょう", DayOffset(0)),
        ("明日", DayOffset(1)),
        ("あした", DayOffset(1)),
        ("あす", DayOffset(1)),
        ("昨日", DayOffset(-1)),
        ("きのう", DayOffset(-1)),
        ("明後日", DayOffset(2)),
        ("あさって", DayOffset(2)),
        ("一昨日", DayOffset(-2)),
        ("おととい", DayOffset(-2)),
        ("前", Ago),
        ("後", Later),
        ("来週", Shift(Week, 1)),
        ("先週", Shift(Week, -1)),
        ("今週", Shift(Week, 0)),
        ("来月", Shift(Month, 1)),
        ("先月", Shift(Month, -1)),
        ("今月", Shift(Month, 0)),
        ("来年", Shift(Year, 1)),
        ("去年", Shift(Year, -1)),
        ("昨年", Shift(Year, -1)),
        ("今年", Shift(Year, 0)),
        ("午前", DayPart(Am)),
        ("午後", DayPart(Pm)),
        ("正午", Clock(12)),
        ("令和", Era(2018)),
        ("平成", Era(1988)),
        ("昭和", Era(1925)),
        ("大正", Era(1911)),
        ("明治", Era(1867)),
        ("元", FirstYear),
        ("の", Filler),
    ],
    weak: &[("今", Now)],
};

#[cfg(test)]
mod tests {
    use crate::testing::{assert_no_dates, check_dates};

    #[test]
    fn japanese() {
        check_dates(&[
            ("2026年10月13日", "2026-10-13 00:00:00+02:00"),
            ("令和8年10月13日", "2026-10-13 00:00:00+02:00"),
            ("平成元年1月8日", "1989-01-08 00:00:00+01:00"),
            ("2026年10月13日 21時24分16秒", "2026-10-13 21:24:16+02:00"),
            (
                "2026年 10月 13日 火曜日 21:24:16 CEST",
                "2026-10-13 21:24:16+02:00",
            ),
            ("10月13日(火) 午後3時半", "2026-10-13 15:30:00+02:00"),
            ("２０２６年１０月１３日", "2026-10-13 00:00:00+02:00"),
            ("十月十三日(火)", "2026-10-13 00:00:00+02:00"),
            ("明日 15時", "2026-10-14 15:00:00+02:00"),
            ("3日後", "2026-10-16 21:24:16+02:00"),
            ("2週間前", "2026-09-29 21:24:16+02:00"),
            ("来週", "2026-10-20 00:00:00+02:00"),
            ("来週の金曜日", "2026-10-23 00:00:00+02:00"),
            ("金曜日", "2026-10-16 00:00:00+02:00"),
            ("一昨日", "2026-10-11 00:00:00+02:00"),
        ]);
    }

    #[test]
    fn in_text() {
        check_dates(&[(
            "会議は10月14日(水)の15時からです",
            "2026-10-14 15:00:00+02:00",
        )]);
    }

    #[test]
    fn prose_without_dates() {
        assert_no_dates(&[
            "一時停止してください。今は月が綺麗ですね、金も木も水も大事です。",
            "日本語の文章には日付がありません。分かりましたか？",
        ]);
    }
}
