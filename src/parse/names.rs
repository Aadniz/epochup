//! Lookup tables for month and weekday names, units, keywords and time zone abbreviations.
//!
//! A name is "confident" when it is unlikely to be an ordinary word. Words like
//! "may", "sun" or the German two-letter weekdays still parse, but are not enough
//! on their own to claim that a piece of prose contains a date.

use jiff::civil::Weekday;

/// Month number and whether the name is confident.
pub fn month(w: &str) -> Option<(i8, bool)> {
    let m = match w {
        "january" | "jan" | "januar" | "jänner" | "jän" | "janvier" | "enero" | "gennaio"
        | "januari" | "janeiro" => 1,
        "february" | "feb" | "februar" | "février" | "fevrier" | "febrero" | "febbraio"
        | "februari" | "fevereiro" => 2,
        "march" | "mar" | "märz" | "maerz" | "mär" | "mrz" | "mars" | "marzo" | "maart"
        | "março" => 3,
        "april" | "apr" | "avril" | "abril" | "aprile" => 4,
        "may" | "mai" | "mayo" | "maggio" | "mei" | "maio" => 5,
        "june" | "jun" | "juni" | "juin" | "junio" | "giugno" | "junho" => 6,
        "july" | "jul" | "juli" | "juillet" | "julio" | "luglio" | "julho" => 7,
        "august" | "aug" | "août" | "aout" | "agosto" | "augustus" => 8,
        "september" | "sep" | "sept" | "septembre" | "septiembre" | "setiembre" | "settembre"
        | "setembro" => 9,
        "october" | "oct" | "oktober" | "okt" | "octobre" | "octubre" | "ottobre" | "outubro" => 10,
        "november" | "nov" | "novembre" | "noviembre" | "novembro" => 11,
        "december" | "dec" | "dezember" | "dez" | "décembre" | "decembre" | "diciembre"
        | "dicembre" | "dezembro" => 12,
        _ => return None,
    };
    let weak = matches!(
        w,
        "may" | "mar" | "mars" | "mai" | "mei" | "jan" | "dec" | "aug"
    );
    Some((m, !weak))
}

/// Weekday and whether the name is confident. Single-kanji forms (`火`) are
/// only returned when `in_parens` is set, as in `13日(火)`.
pub fn weekday(w: &str, in_parens: bool) -> Option<(Weekday, bool)> {
    use Weekday::*;
    let d = match w {
        "monday" | "mon" | "montag" | "lundi" | "lunes" | "月曜日" | "月曜" => Monday,
        "tuesday" | "tue" | "tues" | "dienstag" | "mardi" | "martes" | "火曜日" | "火曜" => {
            Tuesday
        }
        "wednesday" | "wed" | "mittwoch" | "mercredi" | "miércoles" | "miercoles" | "水曜日"
        | "水曜" => Wednesday,
        "thursday" | "thu" | "thur" | "thurs" | "donnerstag" | "jeudi" | "jueves" | "木曜日"
        | "木曜" => Thursday,
        "friday" | "fri" | "freitag" | "vendredi" | "viernes" | "金曜日" | "金曜" => Friday,
        "saturday" | "sat" | "samstag" | "sonnabend" | "samedi" | "sábado" | "sabado"
        | "土曜日" | "土曜" => Saturday,
        "sunday" | "sun" | "sonntag" | "dimanche" | "domingo" | "日曜日" | "日曜" => Sunday,
        "mo" => Monday,
        "di" => Tuesday,
        "mi" => Wednesday,
        "do" => Thursday,
        "fr" => Friday,
        "sa" => Saturday,
        "so" => Sunday,
        "月" if in_parens => Monday,
        "火" if in_parens => Tuesday,
        "水" if in_parens => Wednesday,
        "木" if in_parens => Thursday,
        "金" if in_parens => Friday,
        "土" if in_parens => Saturday,
        "日" if in_parens => Sunday,
        _ => return None,
    };
    let weak = w.len() <= 2 || matches!(w, "sun" | "sat" | "wed" | "mon");
    Some((d, !weak))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl Unit {
    pub fn is_sub_day(self) -> bool {
        matches!(self, Unit::Second | Unit::Minute | Unit::Hour)
    }
}

/// A time unit word. In org style (`+2m`) a bare `m` means months; in natural
/// language (`5m ago`) it means minutes.
pub fn unit(w: &str, org: bool) -> Option<Unit> {
    use Unit::*;
    Some(match w {
        "m" if org => Month,
        "m" => Minute,
        "s" | "sec" | "secs" | "second" | "seconds" | "sek" | "sekunde" | "sekunden" | "秒" => {
            Second
        }
        "min" | "mins" | "minute" | "minutes" | "minuten" | "分" => Minute,
        "h" | "hr" | "hrs" | "hour" | "hours" | "std" | "stunde" | "stunden" | "時間" => Hour,
        "d" | "day" | "days" | "tag" | "tage" | "tagen" | "日" => Day,
        "w" | "wk" | "wks" | "week" | "weeks" | "woche" | "wochen" | "週" | "週間" => Week,
        "mo" | "mos" | "month" | "months" | "monat" | "monate" | "monaten" | "ヶ月" | "か月"
        | "カ月" | "ケ月" | "箇月" => Month,
        "y" | "yr" | "yrs" | "year" | "years" | "jahr" | "jahre" | "jahren" | "年" => Year,
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keyword {
    Now,
    /// Days relative to today: today = 0, tomorrow = 1.
    Day(i8),
}

/// Keyword and whether it is confident enough to anchor a date in prose.
pub fn keyword(w: &str) -> Option<(Keyword, bool)> {
    use Keyword::*;
    Some(match w {
        "now" | "jetzt" | "今" => (Now, false),
        "today" | "tdy" | "heute" | "今日" | "本日" | "きょう" => (Day(0), true),
        "tomorrow" | "tmrw" | "tmr" | "明日" | "あした" | "あす" => (Day(1), true),
        "morgen" => (Day(1), false),
        "yesterday" | "yday" | "gestern" | "昨日" | "きのう" => (Day(-1), true),
        "übermorgen" | "uebermorgen" | "明後日" | "あさって" => (Day(2), true),
        "vorgestern" | "一昨日" | "おととい" => (Day(-2), true),
        _ => return None,
    })
}

/// Words that may sit between date parts without meaning anything.
pub fn is_filler(w: &str) -> bool {
    matches!(
        w,
        "at" | "on" | "the" | "of" | "um" | "am" | "den" | "der" | "dem" | "and" | "und"
    )
}

/// Fixed offsets (seconds east of UTC) for common time zone abbreviations.
/// Ambiguous ones use the most common reading (CST is US Central, IST is India).
pub fn tz_abbrev(w: &str) -> Option<i32> {
    const H: i32 = 3600;
    Some(match w {
        "utc" | "gmt" | "ut" | "zulu" | "wet" => 0,
        "west" | "bst" | "cet" | "mez" | "wat" => H,
        "cest" | "mesz" | "eet" | "sast" | "cat" => 2 * H,
        "eest" | "msk" | "eat" => 3 * H,
        "gst" => 4 * H,
        "pkt" => 5 * H,
        "ist" => 5 * H + 1800,
        "ict" | "wib" => 7 * H,
        "hkt" | "sgt" | "awst" | "pht" | "wita" => 8 * H,
        "jst" | "kst" | "wit" => 9 * H,
        "acst" => 9 * H + 1800,
        "aest" => 10 * H,
        "acdt" => 10 * H + 1800,
        "aedt" => 11 * H,
        "nzst" => 12 * H,
        "nzdt" => 13 * H,
        "hst" => -10 * H,
        "akst" => -9 * H,
        "akdt" | "pst" => -8 * H,
        "pdt" | "mst" => -7 * H,
        "mdt" | "cst" => -6 * H,
        "cdt" | "est" => -5 * H,
        "edt" | "ast" => -4 * H,
        "adt" | "brt" | "art" => -3 * H,
        "nst" => -3 * H - 1800,
        "ndt" => -2 * H - 1800,
        _ => return None,
    })
}

/// Abbreviations that name a region rather than a fixed offset.
pub fn tz_region(w: &str) -> Option<&'static str> {
    Some(match w {
        "pt" => "America/Los_Angeles",
        "mt" => "America/Denver",
        "ct" => "America/Chicago",
        "et" => "America/New_York",
        _ => return None,
    })
}

/// Japanese era name to the Gregorian year before its first year.
pub fn era(w: &str) -> Option<i16> {
    Some(match w {
        "令和" => 2018,
        "平成" => 1988,
        "昭和" => 1925,
        "大正" => 1911,
        "明治" => 1867,
        _ => return None,
    })
}
