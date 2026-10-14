//! Chinese, simplified and traditional.

use super::{Lang, Meaning::*, Period::*, Unit::*};

pub const LANG: Lang = Lang {
    code: "zh",
    name: "中文",
    label: "中文日期",
    year_offset: 0,
    words: &[
        ("星期", WeekWord),
        ("礼拜", WeekWord),
        ("禮拜", WeekWord),
        ("周", WeekWord),
        ("週", WeekWord),
        ("日", WeekSunday),
        ("天", WeekSunday),
        ("秒", UnitName(Second)),
        ("秒钟", UnitName(Second)),
        ("秒鐘", UnitName(Second)),
        ("分", UnitName(Minute)),
        ("分钟", UnitName(Minute)),
        ("分鐘", UnitName(Minute)),
        ("小时", UnitName(Hour)),
        ("小時", UnitName(Hour)),
        ("钟头", UnitName(Hour)),
        ("鐘頭", UnitName(Hour)),
        ("天", UnitName(Day)),
        ("日", UnitName(Day)),
        ("周", UnitName(Week)),
        ("週", UnitName(Week)),
        ("星期", UnitName(Week)),
        ("礼拜", UnitName(Week)),
        ("禮拜", UnitName(Week)),
        ("年", UnitName(Year)),
        ("年", YearMark),
        ("月", MonthMark),
        ("日", DayMark),
        ("号", DayMark),
        ("號", DayMark),
        ("时", HourMark),
        ("時", HourMark),
        ("点", HourMark),
        ("點", HourMark),
        ("分", MinuteMark),
        ("秒", SecondMark),
        ("半", Half),
        ("刻", Quarter),
        ("今天", DayOffset(0)),
        ("明天", DayOffset(1)),
        ("昨天", DayOffset(-1)),
        ("后天", DayOffset(2)),
        ("後天", DayOffset(2)),
        ("前天", DayOffset(-2)),
        ("大后天", DayOffset(3)),
        ("大後天", DayOffset(3)),
        ("大前天", DayOffset(-3)),
        ("前", Ago),
        ("以前", Ago),
        ("之前", Ago),
        ("后", Later),
        ("後", Later),
        ("以后", Later),
        ("之后", Later),
        ("以後", Later),
        ("之後", Later),
        ("个", Measure),
        ("個", Measure),
        ("下个", NextMeasured(1)),
        ("下個", NextMeasured(1)),
        ("上个", NextMeasured(-1)),
        ("上個", NextMeasured(-1)),
        ("这个", NextMeasured(0)),
        ("這個", NextMeasured(0)),
        ("下周", Shift(Week, 1)),
        ("下週", Shift(Week, 1)),
        ("上周", Shift(Week, -1)),
        ("上週", Shift(Week, -1)),
        ("这周", Shift(Week, 0)),
        ("這週", Shift(Week, 0)),
        ("本周", Shift(Week, 0)),
        ("本週", Shift(Week, 0)),
        ("明年", Shift(Year, 1)),
        ("去年", Shift(Year, -1)),
        ("今年", Shift(Year, 0)),
        ("上午", DayPart(Am)),
        ("早上", DayPart(Am)),
        ("早晨", DayPart(Am)),
        ("清晨", DayPart(Am)),
        ("凌晨", DayPart(Am)),
        ("半夜", DayPart(Am)),
        ("下午", DayPart(Pm)),
        ("傍晚", DayPart(Pm)),
        ("中午", DayPart(Noon)),
        ("晚上", DayPart(Night)),
        ("夜里", DayPart(Night)),
        ("夜裡", DayPart(Night)),
        ("夜间", DayPart(Night)),
        ("夜間", DayPart(Night)),
        ("中午", Clock(12)),
        ("午夜", Clock(0)),
        ("民国", Era(1911)),
        ("民國", Era(1911)),
        ("西元", CommonEra),
        ("公元", CommonEra),
        ("的", Filler),
        ("在", Filler),
    ],
    weak: &[("现在", Now), ("現在", Now)],
};

#[cfg(test)]
mod tests {
    use crate::testing::{assert_no_dates, check_dates};

    #[test]
    fn locale_formats() {
        // What `date` prints in the Chinese locales.
        check_dates(&[
            (
                "2026年 10月 13日 星期二 21:24:16 CEST",
                "2026-10-13 21:24:16+02:00",
            ),
            (
                "2026年10月13日 星期二 21时24分16秒",
                "2026-10-13 21:24:16+02:00",
            ),
            (
                "西元2026年10月13日 (週二) 21時24分16秒 CEST",
                "2026-10-13 21:24:16+02:00",
            ),
            (
                "2026年10月13日 星期二 下午 09時24分16秒 CEST",
                "2026-10-13 21:24:16+02:00",
            ),
            (
                "2026年10月13日 21时24分16秒 CEST",
                "2026-10-13 21:24:16+02:00",
            ),
        ]);
    }

    #[test]
    fn chinese() {
        check_dates(&[
            // Dates
            ("2026年10月13号", "2026-10-13 00:00:00+02:00"),
            ("二〇二六年十月十三日", "2026-10-13 00:00:00+02:00"),
            ("十月十三日", "2026-10-13 00:00:00+02:00"),
            ("民國115年10月13日", "2026-10-13 00:00:00+02:00"),
            // Times
            ("明天下午3点", "2026-10-14 15:00:00+02:00"),
            ("明天下午三点半", "2026-10-14 15:30:00+02:00"),
            ("今天晚上8点", "2026-10-13 20:00:00+02:00"),
            ("晚上12点", "2026-10-13 00:00:00+02:00"),
            ("凌晨2点", "2026-10-13 02:00:00+02:00"),
            ("中午12点", "2026-10-13 12:00:00+02:00"),
            ("中午1点", "2026-10-13 13:00:00+02:00"),
            ("上午9点05分", "2026-10-13 09:05:00+02:00"),
            ("下午3点三刻", "2026-10-13 15:45:00+02:00"),
            ("3点一刻", "2026-10-13 03:15:00+02:00"),
            ("21点", "2026-10-13 21:00:00+02:00"),
            // Relative
            ("现在", "2026-10-13 21:24:16+02:00"),
            ("后天", "2026-10-15 00:00:00+02:00"),
            ("大前天", "2026-10-10 00:00:00+02:00"),
            ("3天后", "2026-10-16 21:24:16+02:00"),
            ("两个小时前", "2026-10-13 19:24:16+02:00"),
            ("兩個小時後", "2026-10-13 23:24:16+02:00"),
            ("半小时后", "2026-10-13 21:54:16+02:00"),
            ("3个月后", "2027-01-13 21:24:16+01:00"),
            ("2个星期前", "2026-09-29 21:24:16+02:00"),
            ("10分钟以后", "2026-10-13 21:34:16+02:00"),
            // Weekdays and weeks (weeks start on Monday; today is Tuesday the 13th)
            ("星期五", "2026-10-16 00:00:00+02:00"),
            ("周日", "2026-10-18 00:00:00+02:00"),
            ("礼拜天", "2026-10-18 00:00:00+02:00"),
            ("下周一", "2026-10-19 00:00:00+02:00"),
            ("下週三下午2点", "2026-10-21 14:00:00+02:00"),
            ("上周五", "2026-10-09 00:00:00+02:00"),
            ("这周五", "2026-10-16 00:00:00+02:00"),
            ("本周一", "2026-10-12 00:00:00+02:00"),
            ("下个星期一", "2026-10-19 00:00:00+02:00"),
            ("下周", "2026-10-20 00:00:00+02:00"),
            ("下个月", "2026-11-13 00:00:00+01:00"),
            ("明年", "2027-10-13 00:00:00+02:00"),
        ]);
    }

    #[test]
    fn in_text() {
        check_dates(&[
            ("我们明天下午三点在会议室见面", "2026-10-14 15:00:00+02:00"),
            (
                "发布时间：2026年10月13日 21:24",
                "2026-10-13 21:24:00+02:00",
            ),
            ("周一见！", "2026-10-19 00:00:00+02:00"),
        ]);
    }

    #[test]
    fn prose_without_dates() {
        // 一点 is "a little", not one o'clock.
        assert_no_dates(&[
            "我有一点累，想休息一下。这个问题很重要，我们以后再说。",
            "天气很好，周末我们去公园散步，十分开心。",
        ]);
    }
}
