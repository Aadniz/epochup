//! Helpers for the interpretation tests, with "now" fixed at Tuesday
//! 2026-10-13 21:24:16 in Berlin.

use jiff::tz::TimeZone;

use crate::parse::{Ctx, Interpretation, Kind, interpret};

pub fn ctx() -> Ctx {
    Ctx {
        now: "2026-10-13T21:24:16+02:00[Europe/Berlin]".parse().unwrap(),
        tz: TimeZone::get("Europe/Berlin").unwrap(),
    }
}

pub fn run(input: &str) -> Option<Interpretation> {
    interpret(input, &ctx())
}

/// The resolved date as `YYYY-MM-DD HH:MM:SS±HH:MM`, or what went wrong.
pub fn date_of(input: &str) -> String {
    match run(input).map(|i| i.kind) {
        Some(Kind::Date(r)) => r.zoned.strftime("%Y-%m-%d %H:%M:%S%:z").to_string(),
        Some(Kind::Epoch(e)) => format!("epoch {}", e.ts),
        None => "nothing".into(),
    }
}

/// Checks every case and reports all failures at once.
#[track_caller]
pub fn check_dates(cases: &[(&str, &str)]) {
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|(input, want)| {
            let got = date_of(input);
            (got != *want).then(|| format!("{input:?}: got {got}, want {want}"))
        })
        .collect();
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// Ordinary text that must not be mistaken for a date.
#[track_caller]
pub fn assert_no_dates(texts: &[&str]) {
    let failures: Vec<String> = texts
        .iter()
        .filter_map(|text| {
            let it = run(text)?;
            Some(format!(
                "{text:?}: found {:?} as {}",
                &text[it.span],
                date_of(text)
            ))
        })
        .collect();
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}
