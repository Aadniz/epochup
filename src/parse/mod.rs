//! Turns arbitrary text into either an epoch (to show as a date) or a date
//! (to show as an epoch).

pub mod epoch;
pub mod expr;
pub mod lexer;
mod zones;

#[cfg(test)]
mod tests;

use std::ops::Range;

use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};

use crate::lang::lexicon;
use epoch::Epoch;
use expr::{Mode, Resolved};
use lexer::{Kind as Tok, Token};

/// The reference point for relative dates and the zone for local times.
#[derive(Clone)]
pub struct Ctx {
    pub now: Zoned,
    pub tz: TimeZone,
}

impl Ctx {
    pub fn system() -> Ctx {
        Ctx::now_in(TimeZone::try_system().unwrap_or(TimeZone::UTC))
    }

    /// The current time to the millisecond; finer digits would only flicker past.
    pub fn now_in(tz: TimeZone) -> Ctx {
        let now = Timestamp::now();
        let now = Timestamp::from_millisecond(now.as_millisecond()).unwrap_or(now);
        Ctx {
            now: now.to_zoned(tz.clone()),
            tz,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// The input is mostly an epoch number.
    Epoch,
    /// The whole input is a date expression.
    Expression,
    /// A full date found inside text.
    Text,
    /// Something date-like found inside text.
    LooseText,
    /// An epoch-looking number found inside text.
    EpochInText,
}

pub enum Kind {
    Epoch(Epoch),
    Date(Resolved),
}

pub struct Interpretation {
    pub kind: Kind,
    /// Byte range of the input that was understood.
    pub span: Range<usize>,
    pub stage: Stage,
    /// The same input read as an epoch, when a date reading won (`20261013`, `14`).
    pub as_epoch: Option<Epoch>,
    /// The language of the understood words, like `ja`; none without words (`2026-10-13`).
    pub lang: Option<&'static str>,
}

impl Interpretation {
    fn epoch(e: Epoch, stage: Stage) -> Self {
        Interpretation {
            span: e.span.clone(),
            kind: Kind::Epoch(e),
            stage,
            as_epoch: None,
            lang: None,
        }
    }
}

pub fn interpret(text: &str, ctx: &Ctx) -> Option<Interpretation> {
    let mut fallback = None;
    let mut as_epoch = None;
    if let Some(d) = epoch::dominant(text) {
        if !d.prefers_date {
            return Some(Interpretation::epoch(d.epoch, Stage::Epoch));
        }
        if d.alone {
            as_epoch = Some(d.epoch.clone());
        }
        fallback = Some(d.epoch);
    }
    let toks = lexer::lex(text);
    // Thai text counts years in the Buddhist era, as do users with a Thai locale.
    let era = toks
        .iter()
        .flat_map(|t| t.meanings)
        .map(|e| e.year_offset)
        .find(|&o| o != 0)
        .unwrap_or_else(|| lexicon().default_year_offset());
    let span = |start: usize, end: usize| toks[start].start..toks[end - 1].end;

    if let Some(e) = expr::parse(&toks, text, 0, Mode::Whole)
        && e.complete
        && let Some(r) = expr::resolve(&e.parsed, ctx, Mode::Whole, era)
    {
        let span = span(e.start, e.end);
        return Some(Interpretation {
            kind: Kind::Date(r),
            lang: language(&toks, &span),
            span,
            stage: Stage::Expression,
            as_epoch,
        });
    }
    if let Some(e) = fallback {
        return Some(Interpretation::epoch(e, Stage::Epoch));
    }
    for (mode, stage) in [(Mode::Strong, Stage::Text), (Mode::Weak, Stage::LooseText)] {
        if let Some((r, span)) = scan(&toks, text, ctx, mode, era) {
            return Some(Interpretation {
                kind: Kind::Date(r),
                lang: language(&toks, &span),
                span,
                stage,
                as_epoch: None,
            });
        }
    }
    epoch::in_blob(text).map(|e| Interpretation::epoch(e, Stage::EpochInText))
}

/// The language the understood words are in. Each word votes for the
/// languages it belongs to, a shared word splitting its vote (`oktober` is
/// German and Dutch); ties go to the user's languages, then the listed order.
fn language(toks: &[Token], span: &Range<usize>) -> Option<&'static str> {
    let lexicon = lexicon();
    let mut votes: Vec<(&'static str, f32)> = Vec::new();
    for t in toks
        .iter()
        .filter(|t| t.start >= span.start && t.end <= span.end)
    {
        let mut langs: Vec<&'static str> = t.meanings.iter().map(|e| e.lang).collect();
        langs.sort_unstable();
        langs.dedup();
        for &lang in &langs {
            let share = 1.0 / langs.len() as f32;
            match votes.iter_mut().find(|(l, _)| *l == lang) {
                Some((_, v)) => *v += share,
                None => votes.push((lang, share)),
            }
        }
    }
    let (lang, _) = votes.into_iter().max_by(|a, b| {
        let by_votes = a.1.total_cmp(&b.1);
        by_votes.then(lexicon.rank(b.0).cmp(&lexicon.rank(a.0)))
    })?;
    Some(lang)
}

/// The earliest date expression in the text that is convincing for the mode.
fn scan(
    toks: &[Token],
    src: &str,
    ctx: &Ctx,
    mode: Mode,
    era: i16,
) -> Option<(Resolved, Range<usize>)> {
    (0..toks.len())
        .filter(|&i| expr::could_start(toks, i))
        .find_map(|i| {
            let e = expr::parse(toks, src, i, mode)?;
            let convincing = match mode {
                Mode::Strong => e.parsed.is_strong(),
                _ => e.parsed.is_anchored(),
            };
            if !convincing || !standalone(toks, e.start, e.end) {
                return None;
            }
            let r = expr::resolve(&e.parsed, ctx, mode, era)?;
            Some((r, toks[e.start].start..toks[e.end - 1].end))
        })
}

/// Rejects matches that are part of a longer number-ish run: versions
/// (`v1.2.10`), IP addresses (`10.10.10.10`), longer digit strings.
fn standalone(toks: &[Token], start: usize, end: usize) -> bool {
    let glued = |i: usize| i > 0 && i < toks.len() && toks[i].start == toks[i - 1].end;
    let is_num = |i: usize| matches!(toks.get(i).map(|t| &t.kind), Some(Tok::Num { .. }));
    let is_sep = |i: usize| {
        matches!(
            toks.get(i).map(|t| &t.kind),
            Some(Tok::Sym('.' | ':' | '-' | '/' | ','))
        )
    };
    if glued(start) {
        let prev = start - 1;
        let ascii_word =
            matches!(&toks[prev].kind, Tok::Word(w) if w.chars().all(|c| c.is_ascii_alphabetic()));
        if is_num(prev)
            || (ascii_word && is_num(start))
            || (is_sep(prev) && prev > 0 && glued(prev) && is_num(prev - 1))
        {
            return false;
        }
    }
    if glued(end) && (is_num(end) || (is_sep(end) && glued(end + 1) && is_num(end + 1))) {
        return false;
    }
    true
}
