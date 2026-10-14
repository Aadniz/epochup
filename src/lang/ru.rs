//! Russian. Month and unit names change with their grammatical case
//! (`октябрь`, `13 октября`, `в октябре`), so each form is listed.

use jiff::civil::Weekday::*;

use super::{Lang, Meaning::*, Period::*, Unit::*};

pub const LANG: Lang = Lang {
    code: "ru",
    name: "Русский",
    label: "дата на русском",
    year_offset: 0,
    words: &[
        // Months: nominative, genitive (13 октября), prepositional (в октябре), abbreviations
        ("январь", MonthName(1)),
        ("января", MonthName(1)),
        ("январе", MonthName(1)),
        ("янв", MonthName(1)),
        ("февраль", MonthName(2)),
        ("февраля", MonthName(2)),
        ("феврале", MonthName(2)),
        ("фев", MonthName(2)),
        ("февр", MonthName(2)),
        ("март", MonthName(3)),
        ("марта", MonthName(3)),
        ("марте", MonthName(3)),
        ("мар", MonthName(3)),
        ("апрель", MonthName(4)),
        ("апреля", MonthName(4)),
        ("апреле", MonthName(4)),
        ("апр", MonthName(4)),
        ("май", MonthName(5)),
        ("мая", MonthName(5)),
        ("мае", MonthName(5)),
        ("июнь", MonthName(6)),
        ("июня", MonthName(6)),
        ("июне", MonthName(6)),
        ("июн", MonthName(6)),
        ("июль", MonthName(7)),
        ("июля", MonthName(7)),
        ("июле", MonthName(7)),
        ("июл", MonthName(7)),
        ("август", MonthName(8)),
        ("августа", MonthName(8)),
        ("августе", MonthName(8)),
        ("авг", MonthName(8)),
        ("сентябрь", MonthName(9)),
        ("сентября", MonthName(9)),
        ("сентябре", MonthName(9)),
        ("сен", MonthName(9)),
        ("сент", MonthName(9)),
        ("октябрь", MonthName(10)),
        ("октября", MonthName(10)),
        ("октябре", MonthName(10)),
        ("окт", MonthName(10)),
        ("ноябрь", MonthName(11)),
        ("ноября", MonthName(11)),
        ("ноябре", MonthName(11)),
        ("ноя", MonthName(11)),
        ("нояб", MonthName(11)),
        ("декабрь", MonthName(12)),
        ("декабря", MonthName(12)),
        ("декабре", MonthName(12)),
        ("дек", MonthName(12)),
        // Weekdays: nominative, and accusative after в (в среду)
        ("понедельник", WeekdayName(Monday)),
        ("вторник", WeekdayName(Tuesday)),
        ("среда", WeekdayName(Wednesday)),
        ("среду", WeekdayName(Wednesday)),
        ("четверг", WeekdayName(Thursday)),
        ("пятница", WeekdayName(Friday)),
        ("пятницу", WeekdayName(Friday)),
        ("суббота", WeekdayName(Saturday)),
        ("субботу", WeekdayName(Saturday)),
        ("воскресенье", WeekdayName(Sunday)),
        // Units in their case forms; the accusative singular is a count of one (через час)
        ("секунда", UnitName(Second)),
        ("секунды", UnitName(Second)),
        ("секунд", UnitName(Second)),
        ("сек", UnitName(Second)),
        ("секунду", SingleUnit(Second)),
        ("минута", UnitName(Minute)),
        ("минуты", UnitName(Minute)),
        ("минут", UnitName(Minute)),
        ("мин", UnitName(Minute)),
        ("минуту", SingleUnit(Minute)),
        ("часа", UnitName(Hour)),
        ("часов", UnitName(Hour)),
        ("ч", UnitName(Hour)),
        ("час", SingleUnit(Hour)),
        ("дня", UnitName(Day)),
        ("дней", UnitName(Day)),
        ("день", SingleUnit(Day)),
        ("неделя", UnitName(Week)),
        ("недели", UnitName(Week)),
        ("недель", UnitName(Week)),
        ("неделе", UnitName(Week)),
        ("неделю", SingleUnit(Week)),
        ("месяца", UnitName(Month)),
        ("месяцев", UnitName(Month)),
        ("месяце", UnitName(Month)),
        ("месяц", SingleUnit(Month)),
        ("года", UnitName(Year)),
        ("лет", UnitName(Year)),
        ("году", UnitName(Year)),
        ("год", SingleUnit(Year)),
        // в 3 часа: the hour words also say o'clock
        ("час", OClock),
        ("часа", OClock),
        ("часов", OClock),
        ("сегодня", DayOffset(0)),
        ("завтра", DayOffset(1)),
        ("послезавтра", DayOffset(2)),
        ("вчера", DayOffset(-1)),
        ("позавчера", DayOffset(-2)),
        ("через", In),
        ("спустя", In),
        ("назад", Ago),
        ("тому назад", Ago),
        ("один", Count(1)),
        ("одна", Count(1)),
        ("одну", Count(1)),
        ("одно", Count(1)),
        ("два", Count(2)),
        ("две", Count(2)),
        ("три", Count(3)),
        ("четыре", Count(4)),
        ("пять", Count(5)),
        ("шесть", Count(6)),
        ("семь", Count(7)),
        ("восемь", Count(8)),
        ("девять", Count(9)),
        ("десять", Count(10)),
        ("следующий", Next(1)),
        ("следующая", Next(1)),
        ("следующее", Next(1)),
        ("следующей", Next(1)),
        ("следующую", Next(1)),
        ("следующем", Next(1)),
        ("следующего", Next(1)),
        ("прошлый", Next(-1)),
        ("прошлая", Next(-1)),
        ("прошлое", Next(-1)),
        ("прошлой", Next(-1)),
        ("прошлую", Next(-1)),
        ("прошлом", Next(-1)),
        ("прошлого", Next(-1)),
        // Not это (also "it is"); этот, эту, этой, этом only come before what they point to.
        ("этот", Next(0)),
        ("эту", Next(0)),
        ("этой", Next(0)),
        ("этом", Next(0)),
        // After the hour: в 8 утра, в 3 часа дня, в 9 вечера, в 2 часа ночи
        ("утра", DayPart(Am)),
        ("дня", DayPart(Pm)),
        ("вечера", DayPart(Night)),
        ("ночи", DayPart(Night)),
        ("полдень", Clock(12)),
        ("полночь", Clock(0)),
        // 13 октября 2026 г. (года) в 15:00
        ("в", Filler),
        ("во", Filler),
        ("на", Filler),
        ("г", Filler),
        ("года", Filler),
    ],
    weak: &[
        ("пн", WeekdayName(Monday)),
        ("вт", WeekdayName(Tuesday)),
        ("ср", WeekdayName(Wednesday)),
        ("чт", WeekdayName(Thursday)),
        ("пт", WeekdayName(Friday)),
        ("сб", WeekdayName(Saturday)),
        ("вс", WeekdayName(Sunday)),
        ("сейчас", Now),
        ("теперь", Now),
        // в 9: a bare hour only as the whole input
        ("в", At),
    ],
};

#[cfg(test)]
mod tests {
    use crate::testing::{assert_no_dates, check_dates};

    #[test]
    fn locale_formats() {
        check_dates(&[
            ("Вт 13 окт 2026 21:24:16 CEST", "2026-10-13 21:24:16+02:00"),
            ("Вт 13 окт 2026 21:24:16", "2026-10-13 21:24:16+02:00"),
            ("Вт 13 окт 2026 09:05:00", "2026-10-13 09:05:00+02:00"),
        ]);
    }

    #[test]
    fn russian() {
        check_dates(&[
            ("13 октября 2026", "2026-10-13 00:00:00+02:00"),
            ("13 октября 2026 г.", "2026-10-13 00:00:00+02:00"),
            (
                "вторник, 13 октября 2026 года в 15:00",
                "2026-10-13 15:00:00+02:00",
            ),
            ("1 мая", "2027-05-01 00:00:00+02:00"),
            ("сегодня в 15:30", "2026-10-13 15:30:00+02:00"),
            ("завтра в 9", "2026-10-14 09:00:00+02:00"),
            ("завтра в 3 часа дня", "2026-10-14 15:00:00+02:00"),
            ("в 8 вечера", "2026-10-13 20:00:00+02:00"),
            ("в 2 часа ночи", "2026-10-13 02:00:00+02:00"),
            ("в полдень", "2026-10-13 12:00:00+02:00"),
            ("послезавтра", "2026-10-15 00:00:00+02:00"),
            ("через 3 дня", "2026-10-16 21:24:16+02:00"),
            ("через час", "2026-10-13 22:24:16+02:00"),
            ("через два часа", "2026-10-13 23:24:16+02:00"),
            ("3 дня назад", "2026-10-10 21:24:16+02:00"),
            ("неделю назад", "2026-10-06 21:24:16+02:00"),
            ("5 минут тому назад", "2026-10-13 21:19:16+02:00"),
            ("в следующую пятницу", "2026-10-16 00:00:00+02:00"),
            ("в прошлый понедельник", "2026-10-12 00:00:00+02:00"),
            ("на следующей неделе", "2026-10-20 00:00:00+02:00"),
            ("в среду", "2026-10-14 00:00:00+02:00"),
        ]);
    }

    #[test]
    fn in_text() {
        check_dates(&[
            (
                "Встреча назначена на 13 октября в 15:00, не опаздывайте",
                "2026-10-13 15:00:00+02:00",
            ),
            (
                "Давай увидимся завтра в 3 часа дня",
                "2026-10-14 15:00:00+02:00",
            ),
        ]);
    }

    #[test]
    fn prose_without_dates() {
        assert_no_dates(&[
            "Это был хороший день, и мы долго гуляли в парке с друзьями.",
            "Мы прожили там 3 дня, а потом уехали к морю.",
            "Я был там в прошлом, но уже не помню деталей.",
        ]);
    }
}
