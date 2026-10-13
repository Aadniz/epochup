//! Epoch number detection: either the input is "mostly a number", or a blob
//! contains an epoch-looking number somewhere.

use std::ops::Range;

use jiff::Timestamp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Seconds,
    Millis,
    Micros,
    Nanos,
}

impl Unit {
    pub fn name(self) -> &'static str {
        match self {
            Unit::Seconds => "seconds",
            Unit::Millis => "milliseconds",
            Unit::Micros => "microseconds",
            Unit::Nanos => "nanoseconds",
        }
    }

    fn nanos(self) -> i128 {
        match self {
            Unit::Seconds => 1_000_000_000,
            Unit::Millis => 1_000_000,
            Unit::Micros => 1_000,
            Unit::Nanos => 1,
        }
    }

    /// Guesses the unit from the number of integer digits.
    fn from_digits(n: usize) -> Unit {
        match n {
            0..=11 => Unit::Seconds,
            12..=14 => Unit::Millis,
            15..=17 => Unit::Micros,
            _ => Unit::Nanos,
        }
    }

    fn from_suffix(w: &str) -> Option<Unit> {
        Some(match w {
            "s" | "sec" | "secs" | "second" | "seconds" => Unit::Seconds,
            "ms" | "msec" | "millis" | "milliseconds" => Unit::Millis,
            "us" | "µs" | "μs" | "usec" | "micros" | "microseconds" => Unit::Micros,
            "ns" | "nsec" | "nanos" | "nanoseconds" => Unit::Nanos,
            _ => return None,
        })
    }
}

/// Smaller numbers (before 1970-04-26) are far more likely to be counts,
/// years or days of the month than instants.
const MIN_EPOCH: i64 = 10_000_000;

#[derive(Clone, Debug)]
pub struct Epoch {
    pub ts: Timestamp,
    pub unit: Unit,
    pub span: Range<usize>,
}

#[derive(Debug)]
struct Lit {
    span: Range<usize>,
    at: bool,
    neg: bool,
    signed: bool,
    int: String,
    frac: String,
    exp: i32,
    suffix: Option<Unit>,
}

/// Characters that don't count when judging how much of the input a number takes up.
fn weight(s: &str) -> usize {
    s.chars()
        .filter(|c| !c.is_whitespace() && !"\"'`()[]{}<>,;:=".contains(*c))
        .count()
}

fn literal_at(text: &str, start: usize) -> Option<Lit> {
    let b = text.as_bytes();
    let mut i = start;
    let at = b[i] == b'@';
    if at {
        i += 1;
    }
    let (mut neg, mut signed) = (false, false);
    if matches!(b.get(i), Some(b'-' | b'+')) {
        neg = b[i] == b'-';
        signed = true;
        i += 1;
    }
    if !b.get(i).is_some_and(u8::is_ascii_digit) {
        return None;
    }
    if let Some(prev) = text[..start].chars().next_back() {
        let glued = prev.is_alphanumeric() && !super::lexer::is_cjk(prev);
        if glued
            || matches!(prev, '.' | '_')
            || (!signed && !at && matches!(prev, '-' | '/' | ':' | '+'))
        {
            return None;
        }
    }

    let mut int = String::new();
    loop {
        while let Some(&d) = b.get(i).filter(|d| d.is_ascii_digit()) {
            int.push(d as char);
            i += 1;
        }
        if b.get(i) == Some(&b'_') && b.get(i + 1).is_some_and(u8::is_ascii_digit) {
            i += 1;
            continue;
        }
        break;
    }
    let mut frac = String::new();
    if b.get(i) == Some(&b'.') && b.get(i + 1).is_some_and(u8::is_ascii_digit) {
        i += 1;
        while let Some(&d) = b.get(i).filter(|d| d.is_ascii_digit()) {
            frac.push(d as char);
            i += 1;
        }
    }
    let mut exp = 0i32;
    if matches!(b.get(i), Some(b'e' | b'E')) {
        let mut j = i + 1;
        let eneg = b.get(j) == Some(&b'-');
        if matches!(b.get(j), Some(b'-' | b'+')) {
            j += 1;
        }
        let digits_start = j;
        while b.get(j).is_some_and(u8::is_ascii_digit) {
            j += 1;
        }
        if j > digits_start && j - digits_start <= 3 {
            exp = text[digits_start..j].parse::<i32>().ok()?;
            if eneg {
                exp = -exp;
            }
            i = j;
        }
    }

    // A unit suffix, glued or after spaces: `1760383456ms`, `1760383456 ms`.
    let mut end = i;
    let mut suffix = None;
    let after = &text[i..];
    let trimmed = after.trim_start();
    let word: String = trimmed.chars().take_while(|c| c.is_alphabetic()).collect();
    if !word.is_empty() {
        let glued = trimmed.len() == after.len();
        match Unit::from_suffix(&word.to_lowercase()) {
            Some(u) => {
                suffix = Some(u);
                end = i + (after.len() - trimmed.len()) + word.len();
            }
            None if glued => return None,
            None => {}
        }
    }
    // Reject numbers that continue as a date, time, version or IP: `13.10.`, `12:45`, `10-13`.
    let rest = &b[end..];
    if suffix.is_none()
        && rest.len() >= 2
        && matches!(rest[0], b'.' | b':' | b'-' | b'/')
        && rest[1].is_ascii_digit()
    {
        return None;
    }
    Some(Lit {
        span: start..end,
        at,
        neg,
        signed,
        int,
        frac,
        exp,
        suffix,
    })
}

fn literals(text: &str) -> impl Iterator<Item = Lit> + '_ {
    let mut i = 0;
    std::iter::from_fn(move || {
        while i < text.len() {
            if !text.is_char_boundary(i) {
                i += 1;
                continue;
            }
            if let Some(lit) = literal_at(text, i) {
                i = lit.span.end;
                return Some(lit);
            }
            i += 1;
        }
        None
    })
}

fn decode(lit: &Lit) -> Option<Epoch> {
    let digits = format!("{}{}", lit.int, lit.frac);
    let point = lit.int.len() as i64 + lit.exp as i64;
    let (int_s, frac_s) = if point <= 0 {
        (String::new(), "0".repeat((-point) as usize) + &digits)
    } else if point as usize >= digits.len() {
        (
            digits.clone() + &"0".repeat(point as usize - digits.len()),
            String::new(),
        )
    } else {
        let (a, b) = digits.split_at(point as usize);
        (a.to_string(), b.to_string())
    };
    let int_s = int_s.trim_start_matches('0');
    if int_s.len() > 21 {
        return None;
    }
    let unit = lit.suffix.unwrap_or(Unit::from_digits(int_s.len()));
    let int_v: i128 = if int_s.is_empty() {
        0
    } else {
        int_s.parse().ok()?
    };
    let frac_s = &frac_s[..frac_s.len().min(18)];
    let frac_v: i128 = if frac_s.is_empty() {
        0
    } else {
        frac_s.parse().ok()?
    };
    let mut ns =
        int_v.checked_mul(unit.nanos())? + frac_v * unit.nanos() / 10i128.pow(frac_s.len() as u32);
    if lit.neg {
        ns = -ns;
    }
    let ts = Timestamp::from_nanosecond(ns).ok()?;
    if ts.as_second() < MIN_EPOCH {
        return None;
    }
    Some(Epoch {
        ts,
        unit,
        span: lit.span.clone(),
    })
}

/// True for `YYYYMMDD`, `YYYYMMDDHHMM` and `YYYYMMDDHHMMSS` with a plausible year.
pub fn is_compact_date(s: &str) -> bool {
    if !matches!(s.len(), 8 | 12 | 14) || !s.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    let n = |r: Range<usize>| s[r].parse::<i16>().unwrap_or(-1);
    let (y, m, d) = (n(0..4), n(4..6), n(6..8));
    if !(1900..=2100).contains(&y) || jiff::civil::Date::new(y, m as i8, d as i8).is_err() {
        return false;
    }
    (s.len() < 12 || (n(8..10) < 24 && n(10..12) < 60)) && (s.len() < 14 || n(12..14) < 60)
}

/// A plain number that reads better as a date: `20261013`.
fn prefers_date(lit: &Lit) -> bool {
    let plain =
        !lit.at && !lit.signed && lit.suffix.is_none() && lit.frac.is_empty() && lit.exp == 0;
    plain && is_compact_date(&lit.int)
}

pub struct Dominant {
    pub epoch: Epoch,
    /// The number is all there is (`1791919456`, `14`), not part of a phrase (`KW 42`).
    pub alone: bool,
    /// A date reading of the same input should be tried first: a plain
    /// small number, or a number among other words that isn't epoch-sized.
    pub prefers_date: bool,
}

/// The first number that makes up at least half of the meaningful characters.
pub fn dominant(text: &str) -> Option<Dominant> {
    let total = weight(text);
    if total == 0 {
        return None;
    }
    let lit = literals(text).find(|l| weight(&text[l.span.clone()]) * 2 >= total)?;
    let alone = weight(&text[lit.span.clone()]) == total;
    let epoch_like = lit.at || lit.suffix.is_some() || lit.exp != 0 || lit.int.len() >= 9;
    let prefers_date = if alone {
        prefers_date(&lit)
    } else {
        !epoch_like
    };
    Some(Dominant {
        epoch: decode(&lit)?,
        alone,
        prefers_date,
    })
}

/// The first number in a blob that looks like an epoch in s, ms, µs or ns
/// and lands between 2000 and 2100.
pub fn in_blob(text: &str) -> Option<Epoch> {
    literals(text)
        .filter(|l| l.exp == 0 && matches!(l.int.len(), 10 | 13 | 16 | 19))
        .filter_map(|l| decode(&l))
        .find(|e| (2000..=2100).contains(&e.ts.to_zoned(jiff::tz::TimeZone::UTC).year()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dom(s: &str) -> Option<(i128, Unit, bool)> {
        dominant(s).map(|d| (d.epoch.ts.as_nanosecond(), d.epoch.unit, d.prefers_date))
    }

    const S: i128 = 1_000_000_000;

    #[test]
    fn units_by_magnitude() {
        assert_eq!(
            dom("1760383456"),
            Some((1760383456 * S, Unit::Seconds, false))
        );
        assert_eq!(
            dom("1760383456123"),
            Some((1760383456123 * 1_000_000, Unit::Millis, false))
        );
        assert_eq!(
            dom("1760383456123456"),
            Some((1760383456123456 * 1000, Unit::Micros, false))
        );
        assert_eq!(
            dom("1760383456123456789"),
            Some((1760383456123456789, Unit::Nanos, false))
        );
    }

    #[test]
    fn floats_and_notation() {
        assert_eq!(
            dom("1760383456.5"),
            Some((1760383456 * S + S / 2, Unit::Seconds, false))
        );
        assert_eq!(
            dom("1760383456123.5"),
            Some((1760383456123 * 1_000_000 + 500_000, Unit::Millis, false))
        );
        assert_eq!(
            dom("1.760383456e9"),
            Some((1760383456 * S, Unit::Seconds, false))
        );
        assert_eq!(
            dom("@1760383456"),
            Some((1760383456 * S, Unit::Seconds, false))
        );
        assert_eq!(
            dom("1_760_383_456"),
            Some((1760383456 * S, Unit::Seconds, false))
        );
    }

    #[test]
    fn suffixes() {
        assert_eq!(
            dom("1760383456123 ms"),
            Some((1760383456123 * 1_000_000, Unit::Millis, false))
        );
        assert_eq!(
            dom("1760383456 s"),
            Some((1760383456 * S, Unit::Seconds, false))
        );
        assert!(dom("90s").is_none(), "too small to be an instant");
        assert!(dom("9pm").is_none());
        assert!(dom("+3d").is_none());
    }

    #[test]
    fn mostly_a_number() {
        assert!(dom("\"created\": 1760383456,").is_some());
        assert!(dom("ts=1760383456").is_some());
        assert!(dom("Oct 13 2026").is_none());
        assert!(dom("2026 w4 fri").is_none());
    }

    #[test]
    fn not_dates_or_times() {
        assert!(dom("2026-10-13").is_none());
        assert!(dom("13.10.2026").is_none());
        assert!(dom("12:45").is_none());
        assert!(dom("10/13").is_none());
    }

    #[test]
    fn date_readings_win() {
        assert_eq!(dom("20261013").map(|d| d.2), Some(true));
        assert_eq!(dom("20261313").map(|d| d.2), Some(false));
        assert_eq!(dom("20261013212416").map(|d| d.2), Some(true));
    }

    #[test]
    fn small_numbers_are_not_epochs() {
        for input in [
            "0", "14", "45", "+4", "-86400", "2026", "9999999", "@45", "5000 ms",
        ] {
            assert!(dom(input).is_none(), "{input}");
        }
        assert!(dom("10000000").is_some());
    }

    #[test]
    fn blob() {
        let e = in_blob(r#"{"id": 42, "created_at": 1760383456789, "v": 3}"#).unwrap();
        assert_eq!(e.ts.as_millisecond(), 1760383456789);
        assert!(in_blob("call 0123456789 now").is_none());
    }
}
