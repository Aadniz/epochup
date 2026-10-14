//! Korean. Particles are written glued to words (`내일은`, `3시에`), so Hangul
//! is split by these words like Chinese and Japanese are.

use jiff::civil::Weekday::*;

use super::{Lang, Meaning::*, Period::*, Unit::*};

pub const LANG: Lang = Lang {
    code: "ko",
    name: "한국어",
    label: "한국어 날짜",
    year_offset: 0,
    words: &[
        ("년", YearMark),
        ("년", UnitName(Year)),
        ("월", MonthMark),
        ("일", DayMark),
        ("일", UnitName(Day)),
        ("시", HourMark),
        ("분", MinuteMark),
        ("분", UnitName(Minute)),
        ("초", SecondMark),
        ("초", UnitName(Second)),
        ("반", Half),
        ("월요일", WeekdayName(Monday)),
        ("화요일", WeekdayName(Tuesday)),
        ("수요일", WeekdayName(Wednesday)),
        ("목요일", WeekdayName(Thursday)),
        ("금요일", WeekdayName(Friday)),
        ("토요일", WeekdayName(Saturday)),
        ("일요일", WeekdayName(Sunday)),
        // 2026년 10월 13일 (화)
        ("월", ParenWeekday(Monday)),
        ("화", ParenWeekday(Tuesday)),
        ("수", ParenWeekday(Wednesday)),
        ("목", ParenWeekday(Thursday)),
        ("금", ParenWeekday(Friday)),
        ("토", ParenWeekday(Saturday)),
        ("일", ParenWeekday(Sunday)),
        ("시간", UnitName(Hour)),
        ("주", UnitName(Week)),
        ("주일", UnitName(Week)),
        ("개월", UnitName(Month)),
        ("달", UnitName(Month)),
        ("하루", SingleUnit(Day)),
        ("오늘", DayOffset(0)),
        ("내일", DayOffset(1)),
        ("모레", DayOffset(2)),
        ("어제", DayOffset(-1)),
        ("그저께", DayOffset(-2)),
        ("그제", DayOffset(-2)),
        ("후", Later),
        ("뒤", Later),
        ("이후", Later),
        ("전", Ago),
        ("이전", Ago),
        ("다음", Next(1)),
        ("지난", Next(-1)),
        ("저번", Next(-1)),
        ("이번", Next(0)),
        ("다음 주", Shift(Week, 1)),
        ("다음주", Shift(Week, 1)),
        ("지난 주", Shift(Week, -1)),
        ("지난주", Shift(Week, -1)),
        ("저번 주", Shift(Week, -1)),
        ("저번주", Shift(Week, -1)),
        ("이번 주", Shift(Week, 0)),
        ("이번주", Shift(Week, 0)),
        ("다음 달", Shift(Month, 1)),
        ("다음달", Shift(Month, 1)),
        ("지난 달", Shift(Month, -1)),
        ("지난달", Shift(Month, -1)),
        ("이번 달", Shift(Month, 0)),
        ("이번달", Shift(Month, 0)),
        ("내년", Shift(Year, 1)),
        ("작년", Shift(Year, -1)),
        ("올해", Shift(Year, 0)),
        // Native numbers, for hours and counts: 오후 세 시, 두 시간 후
        ("한", Count(1)),
        ("두", Count(2)),
        ("세", Count(3)),
        ("네", Count(4)),
        ("다섯", Count(5)),
        ("여섯", Count(6)),
        ("일곱", Count(7)),
        ("여덟", Count(8)),
        ("아홉", Count(9)),
        ("열", Count(10)),
        ("열한", Count(11)),
        ("열두", Count(12)),
        ("오전", DayPart(Am)),
        ("오후", DayPart(Pm)),
        ("새벽", DayPart(Am)),
        ("아침", DayPart(Am)),
        ("낮", DayPart(Noon)),
        ("저녁", DayPart(Night)),
        ("밤", DayPart(Night)),
        ("정오", Clock(12)),
        ("자정", Clock(0)),
        // Particles: 3시에 (at 3), 내일은, 3시쯤 (around 3)
        ("에", Filler),
        ("의", Filler),
        ("은", Filler),
        ("는", Filler),
        ("부터", Filler),
        ("까지", Filler),
        ("쯤", Filler),
        ("경", Filler),
        // So that "nowadays" is not read as "today" + 날
        ("오늘날", Other),
    ],
    weak: &[("지금", Now)],
};

#[cfg(test)]
mod tests {
    use crate::testing::{assert_no_dates, check_dates};

    #[test]
    fn locale_formats() {
        check_dates(&[
            (
                "2026. 10. 13. (화) 21:24:16 CEST",
                "2026-10-13 21:24:16+02:00",
            ),
            (
                "2026년 10월 13일 (화) 오후 09시 24분 16초",
                "2026-10-13 21:24:16+02:00",
            ),
            (
                "2026년 10월 13일 (화) 오전 09시 05분 00초",
                "2026-10-13 09:05:00+02:00",
            ),
            ("2026년 10월 13일", "2026-10-13 00:00:00+02:00"),
        ]);
    }

    #[test]
    fn korean() {
        check_dates(&[
            ("10월 13일", "2026-10-13 00:00:00+02:00"),
            ("2026.10.13", "2026-10-13 00:00:00+02:00"),
            ("내일 오후 3시", "2026-10-14 15:00:00+02:00"),
            ("내일은 오후 3시 30분에", "2026-10-14 15:30:00+02:00"),
            ("오후 3시 반", "2026-10-13 15:30:00+02:00"),
            ("오후 세 시", "2026-10-13 15:00:00+02:00"),
            ("저녁 8시", "2026-10-13 20:00:00+02:00"),
            ("새벽 2시", "2026-10-13 02:00:00+02:00"),
            ("모레", "2026-10-15 00:00:00+02:00"),
            ("3일 후", "2026-10-16 21:24:16+02:00"),
            ("2시간 전", "2026-10-13 19:24:16+02:00"),
            ("두 시간 후", "2026-10-13 23:24:16+02:00"),
            ("하루 전", "2026-10-12 21:24:16+02:00"),
            ("3개월 후", "2027-01-13 21:24:16+01:00"),
            ("다음 주 월요일", "2026-10-19 00:00:00+02:00"),
            ("다음주 금요일 오후 2시", "2026-10-23 14:00:00+02:00"),
            ("지난주", "2026-10-06 00:00:00+02:00"),
            ("다음 달", "2026-11-13 00:00:00+01:00"),
            ("금요일", "2026-10-16 00:00:00+02:00"),
            ("다음 금요일", "2026-10-16 00:00:00+02:00"),
        ]);
    }

    #[test]
    fn in_text() {
        check_dates(&[
            (
                "회의는 10월 14일 오후 3시에 시작합니다",
                "2026-10-14 15:00:00+02:00",
            ),
            ("내일 오후 3시에 만나요!", "2026-10-14 15:00:00+02:00"),
        ]);
    }

    #[test]
    fn prose_without_dates() {
        assert_no_dates(&[
            "일이 너무 많아서 주말에도 회사에 갑니다.",
            "네, 알겠습니다. 한국어는 정말 재미있어요.",
            "오늘날 한국 사회는 빠르게 변하고 있다.",
        ]);
    }
}
