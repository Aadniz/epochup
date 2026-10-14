//! Thai. Written without spaces, so split by these words. Years count from the
//! Buddhist era (2569 is 2026), and the clock names hours by part of the day:
//! ตี 2 is 2 am, บ่าย 3 โมง 3 pm, 2 ทุ่ม 8 pm.

use jiff::civil::Weekday::*;

use super::{Lang, Meaning::*, Period::*, Unit::*};

pub const LANG: Lang = Lang {
    code: "th",
    name: "ไทย",
    label: "วันที่ภาษาไทย",
    year_offset: 543,
    words: &[
        ("มกราคม", MonthName(1)),
        ("กุมภาพันธ์", MonthName(2)),
        ("มีนาคม", MonthName(3)),
        ("เมษายน", MonthName(4)),
        ("พฤษภาคม", MonthName(5)),
        ("มิถุนายน", MonthName(6)),
        ("กรกฎาคม", MonthName(7)),
        ("สิงหาคม", MonthName(8)),
        ("กันยายน", MonthName(9)),
        ("ตุลาคม", MonthName(10)),
        ("พฤศจิกายน", MonthName(11)),
        ("ธันวาคม", MonthName(12)),
        ("ม.ค.", MonthName(1)),
        ("ก.พ.", MonthName(2)),
        ("มี.ค.", MonthName(3)),
        ("เม.ย.", MonthName(4)),
        ("พ.ค.", MonthName(5)),
        ("มิ.ย.", MonthName(6)),
        ("ก.ค.", MonthName(7)),
        ("ส.ค.", MonthName(8)),
        ("ก.ย.", MonthName(9)),
        ("ต.ค.", MonthName(10)),
        ("พ.ย.", MonthName(11)),
        ("ธ.ค.", MonthName(12)),
        ("วันจันทร์", WeekdayName(Monday)),
        ("วันอังคาร", WeekdayName(Tuesday)),
        ("วันพุธ", WeekdayName(Wednesday)),
        ("วันพฤหัสบดี", WeekdayName(Thursday)),
        ("วันพฤหัส", WeekdayName(Thursday)),
        ("วันศุกร์", WeekdayName(Friday)),
        ("วันเสาร์", WeekdayName(Saturday)),
        ("วันอาทิตย์", WeekdayName(Sunday)),
        ("วินาที", UnitName(Second)),
        ("วินาที", SecondMark),
        ("นาที", UnitName(Minute)),
        ("นาที", MinuteMark),
        ("ชั่วโมง", UnitName(Hour)),
        ("วัน", UnitName(Day)),
        // Alone, these mean one of themselves: เดือนที่แล้ว (last month), ปีก่อน (a year ago)
        ("สัปดาห์", SingleUnit(Week)),
        ("อาทิตย์", UnitName(Week)),
        ("เดือน", SingleUnit(Month)),
        ("ปี", SingleUnit(Year)),
        ("วันนี้", DayOffset(0)),
        ("พรุ่งนี้", DayOffset(1)),
        ("มะรืนนี้", DayOffset(2)),
        ("มะรืน", DayOffset(2)),
        ("เมื่อวาน", DayOffset(-1)),
        ("เมื่อวานนี้", DayOffset(-1)),
        ("เมื่อวานซืน", DayOffset(-2)),
        ("อีก", In),
        ("ใน", In),
        ("ก่อน", Ago),
        ("ที่แล้ว", Ago),
        ("ข้างหน้า", Later),
        ("สัปดาห์หน้า", Shift(Week, 1)),
        ("อาทิตย์หน้า", Shift(Week, 1)),
        ("เดือนหน้า", Shift(Month, 1)),
        ("ปีหน้า", Shift(Year, 1)),
        ("สัปดาห์นี้", Shift(Week, 0)),
        ("อาทิตย์นี้", Shift(Week, 0)),
        ("เดือนนี้", Shift(Month, 0)),
        ("ปีนี้", Shift(Year, 0)),
        ("หนึ่ง", Count(1)),
        ("สอง", Count(2)),
        ("สาม", Count(3)),
        ("สี่", Count(4)),
        ("ห้า", Count(5)),
        ("หก", Count(6)),
        ("เจ็ด", Count(7)),
        ("แปด", Count(8)),
        ("เก้า", Count(9)),
        ("สิบ", Count(10)),
        ("สิบเอ็ด", Count(11)),
        ("สิบสอง", Count(12)),
        // The clock: 21 นาฬิกา, 3 โมง (hours of the day), 2 ทุ่ม (from 18:00), 15.30 น.
        ("นาฬิกา", HourMark),
        ("โมง", HourMark),
        ("ทุ่ม", HourFrom(18)),
        ("น.", OClock),
        ("ตี", DayPart(Am)),
        ("เช้า", DayPart(Am)),
        ("บ่าย", DayPart(Pm)),
        ("เย็น", DayPart(Pm)),
        ("เที่ยง", Clock(12)),
        ("เที่ยงวัน", Clock(12)),
        ("เที่ยงคืน", Clock(0)),
        // เวลา 15:00 (at 15:00), วันที่ 13 (the 13th), พ.ศ. 2569 (Buddhist era)
        ("เวลา", Filler),
        ("ที่", Filler),
        ("พ.ศ.", Filler),
        ("ค.ศ.", Filler),
    ],
    weak: &[
        // Also the planets, and อาทิตย์ the sun
        ("จันทร์", WeekdayName(Monday)),
        ("อังคาร", WeekdayName(Tuesday)),
        ("พุธ", WeekdayName(Wednesday)),
        ("พฤหัสบดี", WeekdayName(Thursday)),
        ("พฤหัส", WeekdayName(Thursday)),
        ("ศุกร์", WeekdayName(Friday)),
        ("เสาร์", WeekdayName(Saturday)),
        ("อาทิตย์", WeekdayName(Sunday)),
        ("อา.", WeekdayName(Sunday)),
        ("จ.", WeekdayName(Monday)),
        ("อ.", WeekdayName(Tuesday)),
        ("พ.", WeekdayName(Wednesday)),
        ("พฤ.", WeekdayName(Thursday)),
        ("ศ.", WeekdayName(Friday)),
        ("ส.", WeekdayName(Saturday)),
        ("ตอนนี้", Now),
        ("เดี๋ยวนี้", Now),
        ("ขณะนี้", Now),
    ],
};

#[cfg(test)]
mod tests {
    use crate::testing::{assert_no_dates, check_dates};

    #[test]
    fn locale_formats() {
        check_dates(&[
            ("อ. 13 ต.ค. 2569 21:24:16 CEST", "2026-10-13 21:24:16+02:00"),
            ("อ. 13 ต.ค. 2569, 21:24:16", "2026-10-13 21:24:16+02:00"),
            ("อ. 13 ต.ค. 2569, 09:05:00", "2026-10-13 09:05:00+02:00"),
        ]);
    }

    #[test]
    fn thai() {
        check_dates(&[
            ("13 ตุลาคม 2569", "2026-10-13 00:00:00+02:00"),
            (
                "วันที่ 13 ตุลาคม พ.ศ. 2569 เวลา 15.30 น.",
                "2026-10-13 15:30:00+02:00",
            ),
            ("วันที่ 13/10/2569", "2026-10-13 00:00:00+02:00"),
            ("๑๓ ต.ค. ๒๕๖๙", "2026-10-13 00:00:00+02:00"),
            ("13 ตุลาคม ค.ศ. 2026", "2026-10-13 00:00:00+02:00"),
            ("พรุ่งนี้ บ่าย 3 โมง", "2026-10-14 15:00:00+02:00"),
            ("บ่ายสามโมง", "2026-10-13 15:00:00+02:00"),
            ("4 โมงเย็น", "2026-10-13 16:00:00+02:00"),
            ("2 ทุ่ม", "2026-10-13 20:00:00+02:00"),
            ("ตี 2", "2026-10-13 02:00:00+02:00"),
            ("21 นาฬิกา", "2026-10-13 21:00:00+02:00"),
            ("เที่ยงคืน", "2026-10-13 00:00:00+02:00"),
            ("มะรืนนี้", "2026-10-15 00:00:00+02:00"),
            ("อีก 3 วัน", "2026-10-16 21:24:16+02:00"),
            ("3 วันก่อน", "2026-10-10 21:24:16+02:00"),
            ("2 ชั่วโมงที่แล้ว", "2026-10-13 19:24:16+02:00"),
            ("เดือนที่แล้ว", "2026-09-13 21:24:16+02:00"),
            ("สัปดาห์หน้า", "2026-10-20 00:00:00+02:00"),
            ("วันศุกร์", "2026-10-16 00:00:00+02:00"),
        ]);
    }

    #[test]
    fn in_text() {
        check_dates(&[
            (
                "ประชุมวันที่ 14 ตุลาคม 2569 เวลา 15:00 น. ที่ห้อง 5",
                "2026-10-14 15:00:00+02:00",
            ),
            ("เจอกันพรุ่งนี้ บ่าย 3 โมงนะ", "2026-10-14 15:00:00+02:00"),
        ]);
    }

    #[test]
    fn prose_without_dates() {
        assert_no_dates(&[
            "ฉันชอบอ่านหนังสือทุกเช้า แต่บางครั้งก็ไม่มีเวลา",
            "ปีศาจในนิทานน่ากลัวมาก แต่เด็กๆ ก็ชอบฟัง",
        ]);
    }
}
