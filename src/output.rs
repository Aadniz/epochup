//! What to show for an interpretation: the primary value (copied on Enter),
//! the alternatives next to it, and a line saying what was understood.

use std::ops::Range;

use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};

use crate::locale;
use crate::parse::epoch::Epoch;
use crate::parse::expr::Resolved;
use crate::parse::{Ctx, Interpretation, Kind, Stage};

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub text: String,
    /// A short hint shown with the value, like "UTC" or "ms"; never copied.
    pub tag: Option<String>,
    /// What the format is, shown on hover.
    pub label: String,
}

impl Item {
    fn new(text: impl Into<String>, label: impl Into<String>) -> Self {
        Item {
            text: text.into(),
            tag: None,
            label: label.into(),
        }
    }

    fn tagged(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }
}

#[derive(Clone, Debug)]
pub struct Outputs {
    pub primary: Item,
    pub alternatives: Vec<Item>,
    pub info: String,
    /// Byte range of the input that was understood.
    pub span: Option<Range<usize>>,
    /// Depends on the clock, so it should be refreshed while shown.
    pub live: bool,
    /// The language the date was read in, said in it (日本語の日付); none for English.
    pub language: Option<&'static str>,
}

pub fn build(it: &Interpretation, input: &str, ctx: &Ctx) -> Outputs {
    let mut out = match &it.kind {
        Kind::Epoch(e) => from_epoch(e, it.stage, ctx),
        Kind::Date(r) => {
            let what = format!("“{}”", snippet(&input[it.span.clone()]));
            let mut out = from_date(r, &what, ctx);
            if let Some(e) = &it.as_epoch {
                let z = e.ts.to_zoned(ctx.tz.clone());
                out.alternatives.push(
                    Item::new(locale::date(&z), format!("read as epoch {}", e.unit.name()))
                        .tagged("epoch"),
                );
            }
            out
        }
    };
    out.span = Some(it.span.clone());
    out.language = it
        .lang
        .filter(|&code| code != "en")
        .and_then(crate::lang::label);
    dedupe(&mut out);
    out
}

/// What to show with nothing to read: the current time.
pub fn now(ctx: &Ctx) -> Outputs {
    let r = Resolved {
        zoned: ctx.now.clone(),
        explicit_zone: true,
        readings: vec![],
        live: true,
        note: None,
    };
    let mut out = from_date(&r, "now", ctx);
    dedupe(&mut out);
    out
}

fn from_epoch(e: &Epoch, stage: Stage, ctx: &Ctx) -> Outputs {
    let z = e.ts.to_zoned(ctx.tz.clone());
    let found = if stage == Stage::EpochInText {
        " found in text"
    } else {
        ""
    };
    Outputs {
        primary: Item::new(locale::date(&z), format!("{} date", locale::name())),
        alternatives: date_formats(&z, ctx),
        info: format!("epoch {}{found} → {}", e.unit.name(), zone_desc(&z)),
        span: None,
        live: false,
        language: None,
    }
}

fn from_date(r: &Resolved, what: &str, ctx: &Ctx) -> Outputs {
    let ts = r.zoned.timestamp();
    let mut primary = Item::new(seconds(ts).to_string(), "epoch seconds");
    let mut alternatives =
        vec![Item::new(ts.as_millisecond().to_string(), "epoch milliseconds").tagged("ms")];
    // A live value keeps its fraction even on a whole second, so nothing after it jumps.
    if r.live || ts.subsec_nanosecond() != 0 {
        alternatives.push(Item::new(
            fractional_seconds(ts),
            "epoch seconds with fraction",
        ));
    }
    // Only a written wall-clock time can be reread as UTC; "now" or "in 3 hours" are instants.
    if !r.explicit_zone
        && !r.live
        && r.zoned.offset().seconds() != 0
        && let Ok(utc) = r.zoned.datetime().to_zoned(TimeZone::UTC)
    {
        // Two epochs a few hours apart look alike, so both say which zone they assume.
        primary = primary.tagged("Local");
        alternatives.push(
            Item::new(
                seconds(utc.timestamp()).to_string(),
                "epoch if that time were UTC",
            )
            .tagged("UTC"),
        );
    }
    for z in &r.readings {
        let date = z.strftime("%Y-%m-%d").to_string();
        alternatives.push(
            Item::new(
                seconds(z.timestamp()).to_string(),
                format!("epoch reading it as {date}"),
            )
            .tagged(date),
        );
    }
    // Shown in local time; a time written for another zone also says how it was written.
    let local = ts.to_zoned(ctx.tz.clone());
    let when = locale::strftime(&local, "%Y-%m-%d (%a) %H:%M:%S %Z");
    let mut info = format!("{what} → {when}");
    let ago = relative(ts, ctx.now.timestamp());
    if ago != "now" {
        info.push_str(&format!(" · {ago}"));
    }
    if r.zoned.offset() != local.offset() {
        info.push_str(&format!(
            " · written as {}",
            r.zoned.strftime("%H:%M:%S %:z")
        ));
    }
    if let Some(note) = &r.note {
        info.push_str(&format!(" · {note}"));
    }
    Outputs {
        primary,
        alternatives,
        info,
        span: None,
        live: r.live,
        language: None,
    }
}

/// Formats that don't name their zone are tagged as local time.
fn date_formats(z: &Zoned, ctx: &Ctx) -> Vec<Item> {
    let utc = z.with_time_zone(TimeZone::UTC);
    let locale_c = locale::strftime(z, "%c");
    let mut locale_c = Item::new(locale_c, format!("{} %c", locale::name()));
    if !locale_c.text.contains(&z.strftime("%Z").to_string()) {
        locale_c = locale_c.tagged("Local");
    }
    vec![
        Item::new(
            z.strftime("%Y-%m-%dT%H:%M:%S%.f%:z").to_string(),
            "ISO 8601",
        ),
        Item::new(
            utc.strftime("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
            "ISO 8601, UTC",
        ),
        Item::new(
            z.strftime("%Y-%m-%d %H:%M:%S%.f").to_string(),
            "local time, SQL style",
        )
        .tagged("Local"),
        Item::new(
            jiff::fmt::rfc2822::to_string(z).unwrap_or_default(),
            "RFC 2822",
        ),
        Item::new(
            z.strftime("%a %b %e %H:%M:%S %Z %Y").to_string(),
            "date(1) in the C locale",
        ),
        Item::new(z.strftime("%d.%m.%Y %H:%M:%S").to_string(), "German").tagged("Local"),
        Item::new(z.strftime("%m/%d/%Y %I:%M:%S %p").to_string(), "US").tagged("Local"),
        locale_c,
        Item::new(
            relative(z.timestamp(), ctx.now.timestamp()),
            "relative to now",
        ),
    ]
}

/// Whole epoch seconds, rounded down so fractions of negative epochs stay consistent.
fn seconds(ts: Timestamp) -> i64 {
    ts.as_nanosecond().div_euclid(1_000_000_000) as i64
}

/// Seconds with the fraction in whole groups of three digits (`.120`, `.120500`),
/// so that a value ticking by keeps its width.
fn fractional_seconds(ts: Timestamp) -> String {
    let ns = ts.as_nanosecond();
    let sign = if ns < 0 { "-" } else { "" };
    let (whole, frac) = (
        ns.unsigned_abs() / 1_000_000_000,
        ns.unsigned_abs() % 1_000_000_000,
    );
    let mut frac = format!("{frac:09}");
    while frac.len() > 3 && frac.ends_with("000") {
        frac.truncate(frac.len() - 3);
    }
    format!("{sign}{whole}.{frac}")
}

fn zone_desc(z: &Zoned) -> String {
    let offset = z.strftime("%Z %:z").to_string();
    match z.time_zone().iana_name() {
        Some(name) => format!("{name} ({offset})"),
        None => offset,
    }
}

/// `3 hours ago`, `in 2 days`.
pub fn relative(ts: Timestamp, now: Timestamp) -> String {
    let diff = ts.as_second() - now.as_second();
    let a = diff.unsigned_abs();
    const MIN: u64 = 60;
    const HOUR: u64 = 3600;
    const DAY: u64 = 86400;
    let (div, unit) = match a {
        0 => return "now".into(),
        1..MIN => (1, "second"),
        MIN..HOUR => (MIN, "minute"),
        HOUR..DAY => (HOUR, "hour"),
        _ if a < 45 * DAY => (DAY, "day"),
        _ if a < 365 * DAY => (2_629_746, "month"),
        _ => (31_556_952, "year"),
    };
    let n = (a + div / 2) / div;
    let s = if n == 1 { "" } else { "s" };
    if diff < 0 {
        format!("{n} {unit}{s} ago")
    } else {
        format!("in {n} {unit}{s}")
    }
}

/// The matched text on one line, shortened.
fn snippet(s: &str) -> String {
    let flat = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() > 40 {
        format!("{}…", flat.chars().take(39).collect::<String>())
    } else {
        flat
    }
}

fn dedupe(out: &mut Outputs) {
    let mut seen = vec![out.primary.text.clone()];
    out.alternatives.retain(|item| {
        let new = !item.text.is_empty() && !seen.contains(&item.text);
        seen.push(item.text.clone());
        new
    });
}

#[cfg(test)]
mod tests {
    use jiff::tz::TimeZone;

    use super::*;
    use crate::parse::interpret;

    fn ctx() -> Ctx {
        Ctx {
            now: "2026-10-13T21:24:16+02:00[Europe/Berlin]".parse().unwrap(),
            tz: TimeZone::get("Europe/Berlin").unwrap(),
        }
    }

    fn outputs(input: &str) -> Outputs {
        let ctx = ctx();
        build(&interpret(input, &ctx).unwrap(), input, &ctx)
    }

    #[test]
    fn every_date_format_reads_back() {
        let ctx = ctx();
        for input in [
            "1791919456",
            "1791919456789",
            "946684800",
            "1234567890",
            "10000000",
            "4102444799",
        ] {
            let Kind::Epoch(e) = interpret(input, &ctx).unwrap().kind else {
                panic!("{input} not an epoch")
            };
            let out = outputs(input);
            for item in std::iter::once(&out.primary).chain(&out.alternatives) {
                if item.label == "relative to now" {
                    continue;
                }
                let back = interpret(&item.text, &ctx)
                    .unwrap_or_else(|| panic!("{:?} ({}) not understood", item.text, item.label));
                let Kind::Date(r) = back.kind else {
                    panic!("{:?} ({}) read as an epoch", item.text, item.label)
                };
                assert_eq!(
                    r.zoned.timestamp().as_second(),
                    e.ts.as_second(),
                    "{:?} ({})",
                    item.text,
                    item.label
                );
            }
        }
    }

    #[test]
    fn date_to_epoch() {
        let out = outputs("2026-10-13 21:24:16");
        assert_eq!(out.primary.text, "1791919456");
        let alts: Vec<&str> = out.alternatives.iter().map(|a| a.text.as_str()).collect();
        assert_eq!(alts, ["1791919456000", "1791926656"]);
        assert!(out.info.contains("2026-10-13"), "{}", out.info);
    }

    fn tags(out: &Outputs) -> Vec<Option<&str>> {
        std::iter::once(&out.primary)
            .chain(&out.alternatives)
            .map(|i| i.tag.as_deref())
            .collect()
    }

    #[test]
    fn tags_say_which_zone() {
        // A local wall-clock time and its UTC rereading are a few hours apart: both say which.
        let out = outputs("2026-10-13 21:24:16");
        assert_eq!(tags(&out), [Some("Local"), Some("ms"), Some("UTC")]);
        // An instant has no such pair.
        assert_eq!(tags(&outputs("now")), [None, Some("ms"), None]);
        assert_eq!(tags(&outputs("2026-10-13T19:24:16Z")), [None, Some("ms")]);
        // The other reading of an ambiguous date names the date it assumes.
        assert_eq!(tags(&outputs("03/04/2026"))[3], Some("2026-04-03"));
        // Dates without a zone in them are local.
        let out = outputs("1791919456");
        let local: Vec<&str> = out
            .alternatives
            .iter()
            .filter(|i| i.tag.as_deref() == Some("Local"))
            .map(|i| i.label.as_str())
            .collect();
        assert!(
            local.contains(&"local time, SQL style")
                && local.contains(&"German")
                && local.contains(&"US"),
            "{local:?}"
        );
        assert!(
            !local.contains(&"ISO 8601") && !local.contains(&"RFC 2822"),
            "{local:?}"
        );
    }

    #[test]
    fn explicit_zone_has_no_utc_alternative() {
        let out = outputs("2026-10-13T19:24:16.5Z");
        assert_eq!(out.primary.text, "1791919456");
        let alts: Vec<&str> = out.alternatives.iter().map(|a| a.text.as_str()).collect();
        assert_eq!(alts, ["1791919456500", "1791919456.500"]);
    }

    #[test]
    fn live_values_keep_their_shape() {
        // "now" is an instant: no UTC rereading, and the fraction is always there.
        for input in ["now", "in 3 hours", "2h ago"] {
            let out = outputs(input);
            let labels: Vec<&str> = out.alternatives.iter().map(|a| a.label.as_str()).collect();
            assert_eq!(
                labels,
                ["epoch milliseconds", "epoch seconds with fraction"],
                "{input}"
            );
            let fraction = &out.alternatives[1].text;
            assert_eq!(
                fraction.split('.').nth(1).map(str::len),
                Some(3),
                "{input}: {fraction}"
            );
        }
    }

    #[test]
    fn language_in_its_own_words() {
        assert_eq!(outputs("2026年10月13日").language, Some("日本語の日付"));
        assert_eq!(
            outputs("13. Oktober 2026").language,
            Some("Datum auf Deutsch")
        );
        assert_eq!(
            outputs("13 October 2026").language,
            None,
            "English needs no label"
        );
        assert_eq!(outputs("1791919456").language, None);
    }

    #[test]
    fn epoch_to_date() {
        let out = outputs("1791919456");
        let alt = |label: &str| {
            out.alternatives
                .iter()
                .find(|a| a.label == label)
                .map(|a| a.text.as_str())
        };
        assert_eq!(alt("ISO 8601"), Some("2026-10-13T21:24:16+02:00"));
        assert_eq!(alt("ISO 8601, UTC"), Some("2026-10-13T19:24:16Z"));
        assert_eq!(alt("RFC 2822"), Some("Tue, 13 Oct 2026 21:24:16 +0200"));
        assert_eq!(alt("relative to now"), Some("now"));
        assert!(out.info.contains("Europe/Berlin"), "{}", out.info);
    }

    #[test]
    fn relative_wording() {
        let now: Timestamp = "2026-10-13T19:24:16Z".parse().unwrap();
        let at = |s: i64| relative(Timestamp::from_second(now.as_second() + s).unwrap(), now);
        assert_eq!(at(-3 * 3600), "3 hours ago");
        assert_eq!(at(90), "in 2 minutes");
        assert_eq!(at(86400), "in 1 day");
        assert_eq!(at(-400 * 86400), "1 year ago");
    }
}
