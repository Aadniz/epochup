//! The date expression grammar: dates, times, zones, weekdays and relative
//! offsets in any order, resolved org-mode style against "now".

use jiff::civil::{Date, ISOWeekDate, Time, Weekday};
use jiff::tz::{Offset, TimeZone};
use jiff::{Span, ToSpan, Zoned};

use super::Ctx;
use super::epoch::is_compact_date;
use super::lexer::{Kind, Token, normalize};
use super::names::{self, Keyword, Unit};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// The whole input is the expression; partial forms (`14`, `fri`, `+3d`) are fine.
    Whole,
    /// Looking for a complete date inside arbitrary text.
    Strong,
    /// Looking for anything date-like inside arbitrary text.
    Weak,
}

#[derive(Clone, Debug, Default)]
struct DatePart {
    year: Option<i16>,
    month: Option<i8>,
    day: Option<i8>,
    /// Other valid readings of an ambiguous numeric date.
    alts: Vec<(Option<i16>, i8, i8)>,
    /// Complete enough to be found in prose on its own.
    strong: bool,
    /// Written with a month name (or Japanese 月), so it reads as a date even without a year.
    named: bool,
}

#[derive(Clone, Copy, Debug)]
struct TimePart {
    h: i8,
    m: i8,
    s: i8,
    ns: i32,
    seconds: bool,
    iso_t: bool,
}

impl TimePart {
    fn hm(h: i8, m: i8) -> Self {
        TimePart {
            h,
            m,
            s: 0,
            ns: 0,
            seconds: false,
            iso_t: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ZoneKind {
    Offset,
    Abbrev,
    Named,
}

#[derive(Clone, Copy, Debug)]
enum Rel {
    /// `natural` relatives (`in 3 days`, `2h ago`) keep the current clock time.
    Units { n: i64, unit: Unit, natural: bool },
    /// org-mode `+2tue`, `-wed`, `next fri`.
    Weekday { n: i64, wd: Weekday },
    /// A weekday in the week `weeks` from this one, weeks starting on Monday: 下周一.
    WeekOf { weeks: i64, wd: Weekday },
}

enum Comp {
    Filler,
    Open,
    Close,
    Ignored,
    Date(DatePart, Option<TimePart>),
    Time(TimePart),
    Zone(TimeZone, ZoneKind),
    Weekday(Weekday, bool),
    Year(i16),
    Day(i8, bool),
    IsoWeek(Option<i16>, i8, Option<Weekday>),
    Ordinal(i16, i16),
    Rel(Vec<Rel>),
    Keyword(Keyword, bool),
}

#[derive(Clone, Debug, Default)]
pub struct Parsed {
    date: Option<DatePart>,
    time: Option<TimePart>,
    zone: Option<(TimeZone, ZoneKind)>,
    weekday: Option<Weekday>,
    iso_week: Option<(Option<i16>, i8)>,
    ordinal: Option<(i16, i16)>,
    rels: Vec<Rel>,
    day_offset: Option<i8>,
    now: bool,
    /// Components that make a piece of prose look like it names a date.
    anchors: u32,
}

impl Parsed {
    fn merge(&mut self, comp: Comp) -> bool {
        match comp {
            Comp::Filler | Comp::Open | Comp::Close | Comp::Ignored => {}
            Comp::Date(d, t) => {
                if self.iso_week.is_some() || self.ordinal.is_some() {
                    return false;
                }
                match &mut self.date {
                    // A lone year seen earlier (`2026 Oct 13`) joins a yearless date.
                    Some(e) if e.month.is_none() && e.day.is_none() && d.year.is_none() => {
                        let year = e.year;
                        *e = d;
                        e.year = year;
                    }
                    Some(_) => return false,
                    None => {
                        if d.strong || (d.named && d.day.is_some()) {
                            self.anchors += 1;
                        }
                        self.date = Some(d);
                    }
                }
                if let Some(t) = t {
                    if self.time.is_some() {
                        return false;
                    }
                    self.time = Some(t);
                }
            }
            Comp::Time(t) => {
                if self.time.replace(t).is_some() {
                    return false;
                }
                self.anchors += 1;
            }
            Comp::Zone(tz, kind) => match &self.zone {
                // A second zone is an annotation, like `+0200 CEST` or `+02:00[Europe/Berlin]`.
                Some((_, ZoneKind::Offset)) if kind == ZoneKind::Offset => return false,
                Some(_) => {}
                None => self.zone = Some((tz, kind)),
            },
            Comp::Weekday(wd, confident) => {
                if self.weekday.replace(wd).is_some() {
                    return false;
                }
                self.anchors += confident as u32;
            }
            Comp::Year(y) => match &mut self.date {
                Some(d) if d.year.is_none() => {
                    d.year = Some(y);
                    if d.named && d.day.is_some() {
                        d.strong = true;
                    }
                }
                Some(_) => return false,
                None => {
                    self.date = Some(DatePart {
                        year: Some(y),
                        ..Default::default()
                    })
                }
            },
            Comp::Day(day, ordinal) => {
                match &mut self.date {
                    Some(d) if d.day.is_none() => d.day = Some(day),
                    Some(_) => return false,
                    None => {
                        self.date = Some(DatePart {
                            day: Some(day),
                            ..Default::default()
                        })
                    }
                }
                self.anchors += ordinal as u32;
            }
            Comp::IsoWeek(year, week, wd) => {
                if self.iso_week.is_some()
                    || self
                        .date
                        .as_ref()
                        .is_some_and(|d| d.month.is_some() || d.day.is_some())
                {
                    return false;
                }
                if let Some(wd) = wd
                    && self.weekday.replace(wd).is_some()
                {
                    return false;
                }
                self.iso_week = Some((year, week));
                self.anchors += 1;
            }
            Comp::Ordinal(year, doy) => {
                if self.ordinal.is_some() || self.date.is_some() || self.iso_week.is_some() {
                    return false;
                }
                self.ordinal = Some((year, doy));
            }
            Comp::Rel(rels) => {
                self.rels.extend(rels);
                self.anchors += 1;
            }
            Comp::Keyword(Keyword::Now, _) => {
                if std::mem::replace(&mut self.now, true) {
                    return false;
                }
            }
            Comp::Keyword(Keyword::Day(n), confident) => {
                if self.day_offset.replace(n).is_some() {
                    return false;
                }
                self.anchors += confident as u32;
            }
        }
        true
    }

    /// A full calendar date, safe to pick out of arbitrary text.
    pub fn is_strong(&self) -> bool {
        let date = self.date.as_ref().is_some_and(|d| {
            d.strong || (d.named && d.month.is_some() && d.day.is_some() && self.time.is_some())
        });
        date || matches!(self.iso_week, Some((Some(_), _)))
    }

    /// Something date-like enough to pick out of prose when no full date exists.
    pub fn is_anchored(&self) -> bool {
        self.is_strong() || self.anchors > 0
    }
}

pub struct Expr {
    pub parsed: Parsed,
    /// Token range of the expression, without leading or trailing fillers.
    pub start: usize,
    pub end: usize,
    /// Every token was consumed.
    pub complete: bool,
}

/// Parses the longest expression starting at token `start`.
pub fn parse(toks: &[Token], src: &str, start: usize, mode: Mode) -> Option<Expr> {
    let cur = Cur { t: toks, src, mode };
    let mut p = Parsed::default();
    let (mut i, mut first, mut end, mut depth, mut fillers) = (start, None, start, 0i32, 0);
    while i < toks.len() {
        let Some((next, comp)) = cur.component(i, &p, depth) else {
            break;
        };
        match comp {
            Comp::Filler | Comp::Open | Comp::Close => {
                if i == start && mode != Mode::Whole {
                    return None;
                }
                fillers += 1;
                if mode != Mode::Whole && fillers > 2 {
                    break;
                }
                depth += match comp {
                    Comp::Open => 1,
                    Comp::Close => -1,
                    _ => 0,
                };
            }
            comp => {
                let ignored = matches!(comp, Comp::Ignored);
                if !p.merge(comp) {
                    break;
                }
                if !ignored {
                    first.get_or_insert(i);
                    fillers = 0;
                    end = next;
                }
            }
        }
        i = next;
    }
    let first = first?;
    Some(Expr {
        parsed: p,
        start: first,
        end,
        complete: i == toks.len(),
    })
}

fn pivot(yy: u64) -> i16 {
    if yy < 69 {
        2000 + yy as i16
    } else {
        1900 + yy as i16
    }
}

fn ymd(y: i16, m: u64, d: u64) -> Option<(Option<i16>, i8, i8)> {
    if m > 12 || d > 31 {
        return None;
    }
    Date::new(y, m as i8, d as i8)
        .ok()
        .map(|_| (Some(y), m as i8, d as i8))
}

/// Month and day without a year (February 29 allowed).
fn md(m: u64, d: u64) -> Option<(Option<i16>, i8, i8)> {
    ymd(2000, m, d).map(|(_, m, d)| (None, m, d))
}

fn is_ordinal_suffix(w: &str) -> bool {
    matches!(w, "st" | "nd" | "rd" | "th")
}

struct Cur<'a> {
    t: &'a [Token],
    src: &'a str,
    mode: Mode,
}

impl Cur<'_> {
    fn num(&self, i: usize) -> Option<(u64, u8)> {
        match self.t.get(i)?.kind {
            Kind::Num { val, digits } => Some((val, digits)),
            _ => None,
        }
    }

    fn word(&self, i: usize) -> Option<&str> {
        match &self.t.get(i)?.kind {
            Kind::Word(w) => Some(w),
            _ => None,
        }
    }

    fn sym(&self, i: usize) -> Option<char> {
        match self.t.get(i)?.kind {
            Kind::Sym(c) => Some(c),
            _ => None,
        }
    }

    fn is_sym(&self, i: usize, c: char) -> bool {
        self.sym(i) == Some(c)
    }

    /// Token `i` directly follows token `i - 1` with no space between.
    fn adj(&self, i: usize) -> bool {
        i > 0 && i < self.t.len() && self.t[i].start == self.t[i - 1].end
    }

    /// A number glued to the previous token.
    fn adj_num(&self, i: usize) -> Option<(u64, u8)> {
        self.num(i).filter(|_| self.adj(i))
    }

    fn raw(&self, i: usize) -> &str {
        &self.src[self.t[i].start..self.t[i].end]
    }

    fn component(&self, i: usize, p: &Parsed, depth: i32) -> Option<(usize, Comp)> {
        // org-mode: a lone `.` is today.
        if self.mode == Mode::Whole && self.t.len() == 1 && self.is_sym(i, '.') {
            return Some((i + 1, Comp::Keyword(Keyword::Day(0), true)));
        }
        if let Some(r) = self.filler(i) {
            return Some(r);
        }
        // After a time, `+0200` is a zone, not a relative offset.
        if p.time.is_some()
            && let Some(r) = self.zone(i, p)
        {
            return Some(r);
        }
        if self.mode != Mode::Strong {
            if let Some(r) = self.relative(i, p, depth) {
                return Some(r);
            }
            if let Some(r) = self.keyword(i) {
                return Some(r);
            }
        }
        self.iso_week(i)
            .or_else(|| self.tuple_date(i))
            .or_else(|| self.numeric_date(i))
            .or_else(|| self.cjk_date(i))
            .or_else(|| self.month_name_date(i))
            .or_else(|| self.time(i))
            .or_else(|| self.zone(i, p))
            .or_else(|| self.weekday(i))
            .or_else(|| self.lone_year(i, p))
            .or_else(|| self.lone_day(i))
    }

    fn filler(&self, i: usize) -> Option<(usize, Comp)> {
        let comp = match &self.t[i].kind {
            Kind::Sym('<' | '[' | '(' | '{') => Comp::Open,
            Kind::Sym('>' | ']' | ')' | '}') => Comp::Close,
            Kind::Sym(
                ',' | ';' | '"' | '\'' | '`' | '|' | '、' | '。' | '・' | '@' | '=' | '*' | '~'
                | '«' | '»',
            ) => Comp::Filler,
            Kind::Sym('.') if self.adj_num(i + 1).is_none() => Comp::Filler,
            Kind::Word(w) if names::is_filler(w) => Comp::Filler,
            _ => return None,
        };
        Some((i + 1, comp))
    }

    fn keyword(&self, i: usize) -> Option<(usize, Comp)> {
        let (k, confident) = names::keyword(self.word(i)?)?;
        Some((i + 1, Comp::Keyword(k, confident)))
    }

    /// `3 days`, `2h 30m`, `an hour`, `3日`: counts with units, for natural relatives.
    fn unit_list(&self, mut j: usize) -> Option<(usize, Vec<Rel>)> {
        let mut rels = Vec::new();
        loop {
            let (n, k, half) = match (self.num(j), self.word(j)) {
                (Some((v, d)), _) if d <= 6 => (v as i64, j + 1, false),
                (_, Some("a" | "an" | "one" | "ein" | "einem" | "einer" | "einen")) => {
                    (1, j + 1, false)
                }
                (_, Some("半")) => (1, j + 1, true),
                _ => break,
            };
            // Chinese counts months and weeks with a measure word: 3个月, 两个星期.
            let measured = matches!(self.word(k), Some("个" | "個"));
            let k = k + measured as usize;
            let unit = match self.word(k) {
                Some("月") if measured => Some(Unit::Month),
                w => w.and_then(|w| names::unit(w, false)),
            };
            let Some(unit) = unit else {
                break;
            };
            // 半小时 is half an hour.
            let (n, unit) = match (half, unit) {
                (false, unit) => (n, unit),
                (true, Unit::Hour) => (30, Unit::Minute),
                (true, Unit::Minute) => (30, Unit::Second),
                (true, Unit::Day) => (12, Unit::Hour),
                (true, Unit::Week) => (84, Unit::Hour),
                (true, Unit::Month) => (15, Unit::Day),
                (true, Unit::Year) => (6, Unit::Month),
                (true, Unit::Second) => break,
            };
            rels.push(Rel::Units {
                n,
                unit,
                natural: true,
            });
            j = k + 1;
            let joined = matches!(self.word(j), Some("and" | "und")) || self.is_sym(j, ',');
            if joined && (self.num(j + 1).is_some() || matches!(self.word(j + 1), Some("a" | "an")))
            {
                j += 1;
            }
        }
        (!rels.is_empty()).then_some((j, rels))
    }

    /// The day after a Chinese week word: the 一 of 星期一 (or 下周一), 日 or 天 for Sunday.
    fn chinese_day(&self, j: usize) -> Option<(usize, Weekday)> {
        if !self.adj(j) {
            return None;
        }
        if let Some((v, _)) = self.num(j)
            && (1..=6).contains(&v)
        {
            return Some((j + 1, Weekday::from_monday_one_offset(v as i8).ok()?));
        }
        matches!(self.word(j), Some("日" | "天")).then_some((j + 1, Weekday::Sunday))
    }

    /// 星期二, 周二, 週二, 礼拜天.
    fn chinese_weekday(&self, i: usize) -> Option<(usize, Weekday)> {
        match self.word(i)? {
            "星期" | "礼拜" | "禮拜" | "周" | "週" => self.chinese_day(i + 1),
            _ => None,
        }
    }

    /// The day named after a week word: 下周一, 下周的星期一, 来週の金曜日.
    fn day_of_that_week(&self, j: usize) -> Option<(usize, Weekday)> {
        if let Some(day) = self.chinese_day(j) {
            return Some(day);
        }
        let j = if matches!(self.word(j), Some("の" | "的")) {
            j + 1
        } else {
            j
        };
        if let Some(day) = self.chinese_weekday(j) {
            return Some(day);
        }
        let (wd, _) = names::weekday(self.word(j)?, false)?;
        Some((j + 1, wd))
    }

    fn relative(&self, i: usize, p: &Parsed, depth: i32) -> Option<(usize, Comp)> {
        fn negate(rels: Vec<Rel>) -> Vec<Rel> {
            rels.into_iter()
                .map(|r| match r {
                    Rel::Units { n, unit, natural } => Rel::Units {
                        n: -n,
                        unit,
                        natural,
                    },
                    r => r,
                })
                .collect()
        }

        // org-mode: +4, +4d, ++5, +2tue, -wed, +3 days
        if let Some(sign @ ('+' | '-')) = self.sym(i) {
            let mut j = i + 1;
            if self.is_sym(j, sign) && self.adj(j) {
                j += 1;
            }
            let sgn = if sign == '-' { -1 } else { 1 };
            let mut count = None;
            if let Some((v, d)) = self.adj_num(j)
                && d <= 6
            {
                // `-10-13` or `+10:00` continue as something else.
                if self
                    .sym(j + 1)
                    .is_some_and(|c| matches!(c, ':' | '.' | '/' | '-'))
                    && self.adj(j + 1)
                {
                    return None;
                }
                count = Some((v as i64 * sgn, d));
                j += 1;
            } else if !self.adj(j) {
                return None;
            }
            let word = self
                .word(j)
                .filter(|w| self.adj(j) || (count.is_some() && w.chars().count() > 1));
            let unit = word.and_then(|w| names::unit(w, true));
            let weekday = word
                .filter(|_| self.adj(j))
                .and_then(|w| names::weekday(w, false));
            let whole = self.mode == Mode::Whole;
            let (rel, takes_word) = match (count, unit, weekday) {
                // Units always need a count: `-s` is a command-line flag, not a second ago.
                (Some((n, _)), Some(unit), _) => (
                    Rel::Units {
                        n,
                        unit,
                        natural: false,
                    },
                    true,
                ),
                // `-wed` needs no count, but only as the whole input: `ls -sat` is not about Saturday.
                (_, _, Some((wd, _))) if count.is_some() || whole => (
                    Rel::Weekday {
                        n: count.map_or(sgn, |(n, _)| n),
                        wd,
                    },
                    true,
                ),
                // A bare `+4` means days, but only as the whole input; in text it is
                // more likely a phone number.
                (Some((n, digits)), None, None) if whole && digits <= 4 => (
                    Rel::Units {
                        n,
                        unit: Unit::Day,
                        natural: false,
                    },
                    false,
                ),
                _ => return None,
            };
            if takes_word {
                j += 1;
            }
            // Repeaters and warnings inside an org timestamp: <2026-10-13 Tue +1w>.
            if depth > 0
                && p.date
                    .as_ref()
                    .is_some_and(|d| d.year.is_some() && d.day.is_some())
            {
                return Some((j, Comp::Ignored));
            }
            return Some((j, Comp::Rel(vec![rel])));
        }

        // 3 days ago, 2h 30m ago, 3 days from now, 3日後, 2週間前, 3天后, 半小时前
        if (self.num(i).is_some() || self.word(i) == Some("半"))
            && let Some((j, rels)) = self.unit_list(i)
        {
            return match (self.word(j), self.word(j + 1)) {
                (Some("ago" | "前" | "以前" | "之前"), _) => {
                    Some((j + 1, Comp::Rel(negate(rels))))
                }
                (Some("later" | "hence" | "後" | "后" | "以后" | "之后" | "以後" | "之後"), _) => {
                    Some((j + 1, Comp::Rel(rels)))
                }
                (Some("from"), Some("now")) => Some((j + 2, Comp::Rel(rels))),
                _ => None,
            };
        }

        let w = self.word(i)?;
        match w {
            "in" => return self.unit_list(i + 1).map(|(j, rels)| (j, Comp::Rel(rels))),
            "vor" => {
                return self
                    .unit_list(i + 1)
                    .map(|(j, rels)| (j, Comp::Rel(negate(rels))));
            }
            _ => {}
        }
        // A week, month or year away, and perhaps a day of that week:
        // 下周一, 上周五, 来週の金曜日, 下个星期一, 下个月, 来年
        let shift = match w {
            "来週" | "下周" | "下週" => Some((Unit::Week, 1, i + 1)),
            "先週" | "上周" | "上週" => Some((Unit::Week, -1, i + 1)),
            "今週" | "这周" | "這週" | "本周" | "本週" => Some((Unit::Week, 0, i + 1)),
            "来月" => Some((Unit::Month, 1, i + 1)),
            "先月" => Some((Unit::Month, -1, i + 1)),
            "今月" => Some((Unit::Month, 0, i + 1)),
            "来年" | "明年" => Some((Unit::Year, 1, i + 1)),
            "去年" | "昨年" => Some((Unit::Year, -1, i + 1)),
            "今年" => Some((Unit::Year, 0, i + 1)),
            "下个" | "下個" | "上个" | "上個" | "这个" | "這個" => {
                let n = match w {
                    "下个" | "下個" => 1,
                    "上个" | "上個" => -1,
                    _ => 0,
                };
                match self.word(i + 1) {
                    Some("月") => Some((Unit::Month, n, i + 2)),
                    Some("星期" | "礼拜" | "禮拜" | "周" | "週") => {
                        Some((Unit::Week, n, i + 2))
                    }
                    _ => return None,
                }
            }
            _ => None,
        };
        if let Some((unit, n, j)) = shift {
            if unit == Unit::Week
                && let Some((k, wd)) = self.day_of_that_week(j)
            {
                return Some((k, Comp::Rel(vec![Rel::WeekOf { weeks: n, wd }])));
            }
            let rel = Rel::Units {
                n,
                unit,
                natural: false,
            };
            return Some((j, Comp::Rel(vec![rel])));
        }
        let dir = match w {
            "next" | "coming" | "nächste" | "nächsten" | "nächster" | "nächstes" | "kommende"
            | "kommenden" => 1,
            "last" | "previous" | "past" | "letzte" | "letzten" | "letzter" | "letztes"
            | "vorige" | "vorigen" => -1,
            "this" | "diese" | "diesen" | "dieser" | "dieses" => 0,
            "übernächste" | "übernächsten" => 2,
            _ => return None,
        };
        let next = self.word(i + 1)?;
        if let Some((wd, _)) = names::weekday(next, false) {
            let comp = if dir == 0 {
                Comp::Weekday(wd, true)
            } else {
                Comp::Rel(vec![Rel::Weekday { n: dir, wd }])
            };
            return Some((i + 2, comp));
        }
        let unit = names::unit(next, false).filter(|_| next.chars().count() > 1)?;
        Some((
            i + 2,
            Comp::Rel(vec![Rel::Units {
                n: dir,
                unit,
                natural: false,
            }]),
        ))
    }

    /// `w42`, `KW42`, `W42-2`, `week 42`. ISO weeks with a year are parsed by [`Self::numeric_date`].
    fn iso_week(&self, i: usize) -> Option<(usize, Comp)> {
        let w = self.word(i)?;
        let spaced = matches!(w, "kw" | "cw" | "wk" | "week" | "woche");
        if !(spaced || w == "w") || self.mode == Mode::Strong {
            return None;
        }
        let (n, d) = self.num(i + 1)?;
        if d > 2 || !(1..=53).contains(&n) || !(self.adj(i + 1) || spaced) {
            return None;
        }
        let (j, wd) = self.week_day_suffix(i + 2);
        Some((j, Comp::IsoWeek(None, n as i8, wd)))
    }

    /// The `-2` in `W42-2`.
    fn week_day_suffix(&self, j: usize) -> (usize, Option<Weekday>) {
        if self.is_sym(j, '-')
            && self.adj(j)
            && let Some((v, 1)) = self.adj_num(j + 1)
            && let Ok(wd) = Weekday::from_monday_one_offset(v as i8)
        {
            return (j + 2, Some(wd));
        }
        (j, None)
    }

    /// Python-style `(2026, 10, 13, 21, 24, 16)`.
    fn tuple_date(&self, i: usize) -> Option<(usize, Comp)> {
        let (y, 4) = self.num(i)? else { return None };
        if i == 0 || !self.is_sym(i - 1, '(') {
            return None;
        }
        let mut vals = vec![y];
        let mut j = i + 1;
        while vals.len() < 7
            && self.is_sym(j, ',')
            && let Some((v, _)) = self.num(j + 1)
        {
            vals.push(v);
            j += 2;
        }
        if vals.len() < 3 {
            return None;
        }
        let (year, month, day) = ymd(y as i16, vals[1], vals[2])?;
        let time = if vals.len() >= 5 {
            let get = |k: usize| vals.get(k).copied().unwrap_or(0);
            if get(3) > 23 || get(4) > 59 || get(5) > 59 || get(6) > 999_999 {
                return None;
            }
            let ns = (get(6) * 1000) as i32;
            Some(TimePart {
                h: get(3) as i8,
                m: get(4) as i8,
                s: get(5) as i8,
                ns,
                seconds: vals.len() > 5,
                iso_t: false,
            })
        } else {
            None
        };
        let date = DatePart {
            year,
            month: Some(month),
            day: Some(day),
            strong: true,
            ..Default::default()
        };
        Some((j, Comp::Date(date, time)))
    }

    fn numeric_date(&self, i: usize) -> Option<(usize, Comp)> {
        let (a, ad) = self.num(i)?;
        let whole = self.mode == Mode::Whole;
        let raw: String = self.raw(i).chars().map(normalize).collect();

        // Compact 20261013, 202610132124, 20261013212416
        if is_compact_date(&raw) {
            let n = |r: std::ops::Range<usize>| raw[r].parse::<i8>().unwrap_or(0);
            let date = DatePart {
                year: raw[0..4].parse().ok(),
                month: Some(n(4..6)),
                day: Some(n(6..8)),
                // Basic ISO 8601 with a time: 20261013T212416Z
                strong: ad == 8
                    && self.word(i + 1) == Some("t")
                    && self.adj(i + 1)
                    && self.adj_num(i + 2).is_some(),
                ..Default::default()
            };
            let time = (ad >= 12).then(|| TimePart {
                s: if ad == 14 { n(12..14) } else { 0 },
                seconds: ad == 14,
                iso_t: true,
                ..TimePart::hm(n(8..10), n(10..12))
            });
            return Some((i + 1, Comp::Date(date, time)));
        }

        let sep = self
            .sym(i + 1)
            .filter(|c| matches!(c, '-' | '/' | '.'))
            .filter(|_| self.adj(i + 1));
        if ad == 4 {
            // ISO week with year: 2026-W42-2, 2026W42
            let w = if sep == Some('-') { i + 2 } else { i + 1 };
            if self.word(w) == Some("w")
                && self.adj(w)
                && let Some((week, 2)) = self.adj_num(w + 1)
                && (1..=53).contains(&week)
            {
                let (j, wd) = self.week_day_suffix(w + 2);
                return Some((j, Comp::IsoWeek(Some(a as i16), week as i8, wd)));
            }
            let sep = sep?;
            let (b, bd) = self.adj_num(i + 2)?;
            // Ordinal date: 2026-286
            if bd == 3 && sep == '-' {
                if !(1..=366).contains(&b) || self.is_sym(i + 3, '-') {
                    return None;
                }
                return Some((i + 3, Comp::Ordinal(a as i16, b as i16)));
            }
            if bd > 2 {
                return None;
            }
            if self.is_sym(i + 3, sep) && self.adj(i + 3) {
                let (c, cd) = self.adj_num(i + 4)?;
                if cd > 2 {
                    return None;
                }
                let (year, month, day) = ymd(a as i16, b, c)?;
                let date = DatePart {
                    year,
                    month: Some(month),
                    day: Some(day),
                    strong: true,
                    ..Default::default()
                };
                return Some((i + 5, Comp::Date(date, None)));
            }
            // Year and month: 2026-10
            if whole && (1..=12).contains(&b) {
                let date = DatePart {
                    year: Some(a as i16),
                    month: Some(b as i8),
                    ..Default::default()
                };
                return Some((i + 3, Comp::Date(date, None)));
            }
            return None;
        }
        if ad > 2 {
            return None;
        }
        let sep = sep?;
        let (b, bd) = self.adj_num(i + 2)?;
        if bd > 2 {
            return None;
        }
        if self.is_sym(i + 3, sep)
            && self.adj(i + 3)
            && let Some((c, cd)) = self.adj_num(i + 4)
        {
            if cd != 2 && cd != 4 {
                return None;
            }
            let year = if cd == 4 { c as i16 } else { pivot(c) };
            // Slashes read month-first, dots and dashes day-first; the other order is an alternative.
            let (mdy, dmy) = (ymd(year, a, b), ymd(year, b, a));
            let mut readings: Vec<_> = if sep == '/' { [mdy, dmy] } else { [dmy, mdy] }
                .into_iter()
                .flatten()
                .collect();
            if ad == 2
                && bd == 2
                && cd == 2
                && let Some(r) = ymd(pivot(a), b, c)
            {
                readings.push(r);
            }
            let mut unique = Vec::new();
            for r in readings {
                if !unique.contains(&r) {
                    unique.push(r);
                }
            }
            if unique.is_empty() {
                return None;
            }
            let (year, month, day) = unique.remove(0);
            let strong = cd == 4 || (ad == 2 && bd == 2);
            let date = DatePart {
                year,
                month: Some(month),
                day: Some(day),
                alts: unique,
                strong,
                named: false,
            };
            return Some((i + 5, Comp::Date(date, None)));
        }
        if !whole {
            return None;
        }
        // Without a year: 10/13 (month first), 13.10. (day first)
        let (first, second) = match sep {
            '/' => (md(a, b), md(b, a)),
            '.' => (md(b, a), md(a, b)),
            _ => return None,
        };
        let mut j = i + 3;
        if sep == '.' && self.is_sym(j, '.') && self.adj(j) {
            j += 1;
        }
        let (year, month, day) = first.or(second)?;
        let alts = second
            .filter(|s| first.is_some() && Some(*s) != first)
            .into_iter()
            .collect();
        let date = DatePart {
            year,
            month: Some(month),
            day: Some(day),
            alts,
            ..Default::default()
        };
        Some((j, Comp::Date(date, None)))
    }

    /// `2026年10月13日`, `令和8年10月13日`, `民國115年10月13日`, `10月13号`, `二〇二六年十月十三日`.
    fn cjk_date(&self, i: usize) -> Option<(usize, Comp)> {
        let whole = self.mode == Mode::Whole;
        let mut j = i;
        // zh_TW writes the Common Era out: 西元2026年
        if matches!(self.word(j), Some("西元" | "公元")) && self.num(j + 1).is_some() {
            j += 1;
        }
        let mut year = None;
        if let Some(base) = self.word(j).and_then(names::era) {
            let n = match (self.num(j + 1), self.word(j + 1)) {
                (Some((v, _)), _) => v as i16,
                (_, Some("元")) => 1,
                _ => return None,
            };
            if self.word(j + 2) != Some("年") {
                return None;
            }
            year = Some(base + n);
            j += 3;
        } else if let Some((v, d)) = self.num(j)
            && self.word(j + 1) == Some("年")
        {
            year = Some(if d == 2 { pivot(v) } else { v as i16 });
            j += 2;
        }
        let mut month = None;
        if let Some((v, _)) = self.num(j)
            && self.word(j + 1) == Some("月")
            && (1..=12).contains(&v)
        {
            month = Some(v as i8);
            j += 2;
        }
        let mut day = None;
        if let Some((v, _)) = self.num(j)
            && matches!(self.word(j + 1), Some("日" | "号" | "號"))
            && (1..=31).contains(&v)
        {
            day = Some(v as i8);
            j += 2;
        }
        if j == i || !(month.is_some() || whole) {
            return None;
        }
        if let (Some(m), Some(d)) = (month, day)
            && Date::new(year.unwrap_or(2000), m, d).is_err()
        {
            return None;
        }
        let strong = month.is_some() && day.is_some();
        let date = DatePart {
            year,
            month,
            day,
            strong,
            named: true,
            ..Default::default()
        };
        Some((j, Comp::Date(date, None)))
    }

    /// A year after a day: `, 2026`, ` 2026`, ` '26`, or ` 26` (whole input only).
    fn trailing_year(&self, j: usize) -> Option<(usize, i16, bool)> {
        let k = if self.is_sym(j, ',') { j + 1 } else { j };
        if matches!(self.sym(k), Some('\'' | '’'))
            && let Some((y, 2)) = self.adj_num(k + 1)
        {
            return Some((k + 2, pivot(y), true));
        }
        let (y, yd) = self.num(k)?;
        // In `Oct 13 21:24:16` the number after the day is the hour.
        if self
            .sym(k + 1)
            .is_some_and(|c| matches!(c, ':' | '.' | '-' | '/'))
            && self.adj_num(k + 2).is_some()
        {
            return None;
        }
        match yd {
            4 => Some((k + 1, y as i16, true)),
            2 if self.mode == Mode::Whole => Some((k + 1, pivot(y), false)),
            _ => None,
        }
    }

    /// A year that must use the same separator: the `-2026` of `13-Oct-2026`.
    fn separated_year(&self, j: usize, sep: char) -> Option<(usize, i16, u8)> {
        if !self.is_sym(j, sep) || !self.adj(j) {
            return None;
        }
        match self.adj_num(j + 1)? {
            (y, 4) => Some((j + 2, y as i16, 4)),
            (y, 2) => Some((j + 2, pivot(y), 2)),
            _ => None,
        }
    }

    fn month_name_date(&self, i: usize) -> Option<(usize, Comp)> {
        let whole = self.mode == Mode::Whole;
        let valid = |d: &DatePart| match (d.month, d.day) {
            (Some(m), Some(day)) => Date::new(d.year.unwrap_or(2000), m, day).is_ok(),
            _ => true,
        };

        // Day first: 13 Oct 2026, 13. Oktober, 13th of October, 13-Oct-26, 13/Oct/2026:21:24:16
        if let Some((d, dd)) = self.num(i) {
            if dd > 2 || !(1..=31).contains(&d) {
                return None;
            }
            let mut j = i + 1;
            if (self.word(j).is_some_and(is_ordinal_suffix) || self.is_sym(j, '.')) && self.adj(j) {
                j += 1;
            }
            let mut sep = None;
            if let Some(c @ ('-' | '/')) = self.sym(j)
                && self.adj(j)
            {
                sep = Some(c);
                j += 1;
            } else if self.word(j) == Some("of") {
                j += 1;
            }
            let (month, _) = names::month(self.word(j)?)?;
            if sep.is_some() && !self.adj(j) {
                return None;
            }
            j += 1;
            if sep.is_none() && self.is_sym(j, '.') && self.adj(j) {
                j += 1;
            }
            let mut date = DatePart {
                month: Some(month),
                day: Some(d as i8),
                named: true,
                ..Default::default()
            };
            let mut time = None;
            if let Some(sep) = sep {
                if let Some((k, y, yd)) = self.separated_year(j, sep) {
                    date.year = Some(y);
                    date.strong = true;
                    j = k;
                    // Apache log: 13/Oct/2026:21:24:16
                    if yd == 4
                        && self.is_sym(j, ':')
                        && self.adj(j)
                        && let Some((k, Comp::Time(t))) = self.time(j + 1)
                    {
                        time = Some(t);
                        j = k;
                    }
                }
            } else if let Some((k, y, strong)) = self.trailing_year(j) {
                date.year = Some(y);
                date.strong = strong;
                j = k;
            }
            return valid(&date).then_some((j, Comp::Date(date, time)));
        }

        // Month first: Oct 13, 2026 / October 13th / Oct-13-2026 / Oct 2026 / Oct
        let (month, confident) = names::month(self.word(i)?)?;
        let mut j = i + 1;
        if self.is_sym(j, '.') && self.adj(j) {
            j += 1;
        }
        let mut sep = None;
        if let Some(c @ ('-' | '/')) = self.sym(j)
            && self.adj(j)
            && self.adj_num(j + 1).is_some()
        {
            sep = Some(c);
            j += 1;
        }
        let mut date = DatePart {
            month: Some(month),
            named: true,
            ..Default::default()
        };
        if let Some((v, vd)) = self.num(j) {
            let is_time = self.is_sym(j + 1, ':') && self.adj(j + 1);
            if vd == 4 && !is_time {
                date.year = Some(v as i16);
                j += 1;
            } else if vd <= 2 && (1..=31).contains(&v) && !is_time {
                date.day = Some(v as i8);
                j += 1;
                if self.word(j).is_some_and(is_ordinal_suffix) && self.adj(j) {
                    j += 1;
                }
                if let Some(sep) = sep {
                    if let Some((k, y, _)) = self.separated_year(j, sep) {
                        date.year = Some(y);
                        date.strong = true;
                        j = k;
                    }
                } else if let Some((k, y, strong)) = self.trailing_year(j) {
                    date.year = Some(y);
                    date.strong = strong;
                    j = k;
                }
            }
        }
        // A bare month name only counts as the whole input; "may" needs a day in prose.
        if !whole && (date.day.is_none() || (!confident && date.year.is_none())) {
            return None;
        }
        valid(&date).then_some((j, Comp::Date(date, None)))
    }

    /// `.123`, `,123456789` after seconds.
    fn fraction(&self, j: usize) -> Option<(usize, i32)> {
        if !matches!(self.sym(j), Some('.' | ',')) || !self.adj(j) {
            return None;
        }
        self.adj_num(j + 1)?;
        let digits: String = self.raw(j + 1).chars().map(normalize).take(9).collect();
        Some((j + 2, format!("{digits:0<9}").parse().ok()?))
    }

    /// `pm`, `PM`, `p.m.`, and a glued `9p`.
    fn ampm(&self, j: usize) -> Option<(usize, bool)> {
        let w = self.word(j)?;
        match w {
            "am" | "pm" => Some((j + 1, w == "pm")),
            "a" | "p" => {
                if self.is_sym(j + 1, '.')
                    && self.adj(j + 1)
                    && self.word(j + 2) == Some("m")
                    && self.adj(j + 2)
                {
                    let k = if self.is_sym(j + 3, '.') && self.adj(j + 3) {
                        j + 4
                    } else {
                        j + 3
                    };
                    Some((k, w == "p"))
                } else if self.adj(j) {
                    Some((j + 1, w == "p"))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn time(&self, i: usize) -> Option<(usize, Comp)> {
        if let Some(w) = self.word(i) {
            let fixed = match w {
                "noon" | "mittag" | "mittags" | "正午" => Some(12),
                "中午" if self.num(i + 1).is_none() => Some(12),
                "midnight" | "mitternacht" | "午夜" => Some(0),
                _ => None,
            };
            if let Some(h) = fixed {
                return Some((i + 1, Comp::Time(TimePart::hm(h, 0))));
            }
        }
        let mut j = i;
        let iso_t = self.word(j) == Some("t") && self.adj_num(j + 1).is_some();
        if iso_t {
            j += 1;
        }
        // 午後3時, 下午3点, 晚上8点
        let period = self.word(j).and_then(names::period);
        if period.is_some() {
            j += 1;
        }
        let mut pm = None;
        let (h, hd) = self.num(j)?;
        let hour_at = j;
        let (mut hh, mut mm, mut ss, mut ns, mut seconds) = (h as i64, 0i64, 0i64, 0i32, false);
        let mut clock = false;
        if iso_t && (hd == 4 || hd == 6) {
            // Basic format: T2124, T212416
            hh = (h / 10u64.pow(hd as u32 - 2)) as i64;
            mm = (h / 10u64.pow(hd as u32 - 4) % 100) as i64;
            if hd == 6 {
                ss = (h % 100) as i64;
                seconds = true;
            }
            j += 1;
            clock = true;
            if let Some((k, f)) = self.fraction(j) {
                ns = f;
                j = k;
            }
        } else if hd <= 2 {
            j += 1;
            if self.is_sym(j, ':')
                && self.adj(j)
                && let Some((m, 2)) = self.adj_num(j + 1)
            {
                mm = m as i64;
                j += 2;
                clock = true;
                if self.is_sym(j, ':')
                    && self.adj(j)
                    && let Some((s, 2)) = self.adj_num(j + 1)
                {
                    ss = s as i64;
                    seconds = true;
                    j += 2;
                    if let Some((k, f)) = self.fraction(j) {
                        ns = f;
                        j = k;
                    }
                }
            } else if self.word(j) == Some("h") && self.adj(j) {
                // French/German style: 21h30, 21h
                if let Some((m, 2)) = self.adj_num(j + 1) {
                    mm = m as i64;
                    j += 2;
                } else if self.mode == Mode::Whole {
                    j += 1;
                } else {
                    return None;
                }
                clock = true;
            } else if matches!(self.word(j), Some("時" | "时" | "点" | "點")) {
                j += 1;
                clock = true;
                let mut minutes = true;
                if let Some((m, _)) = self.num(j)
                    && self.word(j + 1) == Some("分")
                {
                    mm = m as i64;
                    j += 2;
                    if let Some((s, _)) = self.num(j)
                        && self.word(j + 1) == Some("秒")
                    {
                        ss = s as i64;
                        seconds = true;
                        j += 2;
                    }
                } else if self.word(j) == Some("半") {
                    mm = 30;
                    j += 1;
                } else if let Some((q, _)) = self.num(j)
                    && (1..=3).contains(&q)
                    && self.word(j + 1) == Some("刻")
                {
                    // 3点一刻 is a quarter past three.
                    mm = 15 * q as i64;
                    j += 2;
                } else {
                    minutes = false;
                }
                // In text, 一点 is more often "a little" and 一時 "for a while" than an hour.
                let cjk_numeral = !self
                    .raw(hour_at)
                    .chars()
                    .all(|c| normalize(c).is_ascii_digit());
                if cjk_numeral && !minutes && period.is_none() && self.mode != Mode::Whole {
                    return None;
                }
            } else if self.is_sym(j, '.')
                && self.adj(j)
                && let Some((m, 2)) = self.adj_num(j + 1)
                && self.word(j + 2) == Some("uhr")
            {
                // German: 21.30 Uhr
                mm = m as i64;
                j += 2;
            }
        } else {
            return None;
        }
        if (1..=12).contains(&hh)
            && let Some((k, is_pm)) = self.ampm(j)
        {
            pm = Some(is_pm);
            j = k;
            clock = true;
        }
        if self.word(j) == Some("uhr") {
            j += 1;
            clock = true;
        }
        if period.is_some() {
            clock = true;
        }
        if !clock {
            // A bare hour after "at": `tomorrow at 9`
            let after_at = hour_at > 0
                && (matches!(self.word(hour_at - 1), Some("at" | "um" | "gegen"))
                    || self.is_sym(hour_at - 1, '@'));
            if !after_at || hd > 2 {
                return None;
            }
        }
        if let Some(pm) = pm {
            if !(1..=12).contains(&hh) {
                return None;
            }
            hh = hh % 12 + if pm { 12 } else { 0 };
        }
        if let Some(period) = period {
            if hh > 23 {
                return None;
            }
            hh = period.hour(hh);
        }
        if hh > 24 || mm > 59 || ss > 60 || (hh == 24 && (mm > 0 || ss > 0)) {
            return None;
        }
        let t = TimePart {
            h: hh as i8,
            m: mm as i8,
            s: ss.min(59) as i8,
            ns,
            seconds,
            iso_t,
        };
        Some((j, Comp::Time(t)))
    }

    /// `+02:00`, `+0200`, `+02`, and `+9` right after UTC/GMT. Returns the end, seconds and digit count.
    fn offset(&self, i: usize, after_utc: bool) -> Option<(usize, i32, u8)> {
        let sign = match self.sym(i)? {
            '+' => 1,
            '-' => -1,
            _ => return None,
        };
        if after_utc && !self.adj(i) {
            return None;
        }
        let (v, d) = self.adj_num(i + 1)?;
        let (h, mut m) = match d {
            1 if after_utc => (v, 0),
            2 => (v, 0),
            4 => (v / 100, v % 100),
            _ => return None,
        };
        let mut k = i + 2;
        if d <= 2
            && self.is_sym(k, ':')
            && self.adj(k)
            && let Some((mv, 2)) = self.adj_num(k + 1)
        {
            m = mv;
            k += 2;
        }
        // `+3d` is a relative offset.
        if self.word(k).is_some() && self.adj(k) {
            return None;
        }
        if h > 14 || m > 59 {
            return None;
        }
        Some((k, sign * (h * 3600 + m * 60) as i32, d))
    }

    fn zone(&self, i: usize, p: &Parsed) -> Option<(usize, Comp)> {
        let fixed = |secs: i32| Offset::from_seconds(secs).ok().map(TimeZone::fixed);
        if self.word(i) == Some("z") && self.adj(i) && p.time.is_some() {
            return Some((i + 1, Comp::Zone(TimeZone::UTC, ZoneKind::Offset)));
        }
        if let Some("utc" | "gmt" | "ut") = self.word(i) {
            if let Some((k, secs, _)) = self.offset(i + 1, true) {
                return Some((k, Comp::Zone(fixed(secs)?, ZoneKind::Offset)));
            }
            return Some((i + 1, Comp::Zone(TimeZone::UTC, ZoneKind::Abbrev)));
        }
        if let Some(t) = p.time
            && let Some((k, secs, digits)) = self.offset(i, false)
        {
            // `10:00-11:00` is a range; an offset follows seconds, a `T` time, or is `+0200`.
            if !(t.seconds || t.iso_t || digits == 4) {
                return None;
            }
            return Some((k, Comp::Zone(fixed(secs)?, ZoneKind::Offset)));
        }
        if let Some(w) = self.word(i) {
            // In prose only shouting counts: `CEST`, not the German word "ist".
            let upper = !self.raw(i).chars().any(char::is_lowercase);
            if self.mode == Mode::Whole || upper {
                if let Some(secs) = names::tz_abbrev(w) {
                    return Some((i + 1, Comp::Zone(fixed(secs)?, ZoneKind::Abbrev)));
                }
                if let Some(tz) = names::tz_region(w).and_then(|n| TimeZone::get(n).ok()) {
                    return Some((i + 1, Comp::Zone(tz, ZoneKind::Named)));
                }
            }
        }
        // IANA names: Europe/Berlin, America/Argentina/Buenos_Aires, Etc/GMT+5
        if self.word(i).is_some()
            && self.is_sym(i + 1, '/')
            && self.adj(i + 1)
            && self.word(i + 2).is_some()
            && self.adj(i + 2)
        {
            let mut k = i + 3;
            while k < self.t.len()
                && self.adj(k)
                && matches!(
                    self.t[k].kind,
                    Kind::Word(_) | Kind::Num { .. } | Kind::Sym('/' | '_' | '-' | '+')
                )
            {
                k += 1;
            }
            while k > i + 2 {
                if let Ok(tz) = TimeZone::get(&self.src[self.t[i].start..self.t[k - 1].end]) {
                    return Some((k, Comp::Zone(tz, ZoneKind::Named)));
                }
                k -= 1;
            }
        }
        None
    }

    fn weekday(&self, i: usize) -> Option<(usize, Comp)> {
        if let Some((j, wd)) = self.chinese_weekday(i) {
            return Some((j, Comp::Weekday(wd, true)));
        }
        let w = self.word(i)?;
        let in_parens = i > 0 && self.is_sym(i - 1, '(') && self.is_sym(i + 1, ')');
        let (wd, confident) = names::weekday(w, in_parens)?;
        // In prose, the German "Do"/"So" and English "sun"/"sat" must be capitalized.
        if !confident && self.mode != Mode::Whole && !self.raw(i).starts_with(char::is_uppercase) {
            return None;
        }
        let j = if self.is_sym(i + 1, '.') && self.adj(i + 1) {
            i + 2
        } else {
            i + 1
        };
        Some((j, Comp::Weekday(wd, confident)))
    }

    /// The `2026` in `Tue Oct 13 21:24:16 CEST 2026` or `2026 w4 fri`.
    fn lone_year(&self, i: usize, p: &Parsed) -> Option<(usize, Comp)> {
        let (y, 4) = self.num(i)? else { return None };
        if !(1000..=2999).contains(&y)
            || (self.sym(i + 1).is_some() && self.adj(i + 1) && self.adj_num(i + 2).is_some())
        {
            return None;
        }
        let fits = match &p.date {
            Some(d) => d.year.is_none() && (d.month.is_some() || self.mode == Mode::Whole),
            None => self.mode == Mode::Whole,
        };
        fits.then_some((i + 1, Comp::Year(y as i16)))
    }

    /// `14`, `14th`, `the 14th`, `14.`: a day of the month.
    fn lone_day(&self, i: usize) -> Option<(usize, Comp)> {
        let (v, d) = self.num(i)?;
        if d > 2 || !(1..=31).contains(&v) {
            return None;
        }
        let mut j = i + 1;
        let ordinal = self.word(j).is_some_and(is_ordinal_suffix) && self.adj(j);
        let german_dot = self.mode == Mode::Whole && self.is_sym(j, '.') && self.adj(j);
        if ordinal || german_dot {
            j += 1;
        }
        match self.mode {
            Mode::Whole => {}
            Mode::Weak if ordinal => {}
            _ => return None,
        }
        Some((j, Comp::Day(v as i8, ordinal)))
    }
}

pub struct Resolved {
    pub zoned: Zoned,
    pub explicit_zone: bool,
    /// Other plausible readings: the other day/month order, the other year.
    pub readings: Vec<Zoned>,
    /// The result moves with the clock (`now`, `in 3 hours`).
    pub live: bool,
    pub note: Option<String>,
}

fn span(unit: Unit, n: i64) -> Option<Span> {
    let s = Span::new();
    match unit {
        Unit::Year => s.try_years(n),
        Unit::Month => s.try_months(n),
        Unit::Week => s.try_weeks(n),
        Unit::Day => s.try_days(n),
        Unit::Hour => s.try_hours(n),
        Unit::Minute => s.try_minutes(n),
        Unit::Second => s.try_seconds(n),
    }
    .ok()
}

fn add_days(d: Date, n: i64) -> Option<Date> {
    d.checked_add(span(Unit::Day, n)?).ok()
}

/// The weekday on or after the date.
fn next_weekday(d: Date, wd: Weekday) -> Date {
    let delta = (wd.to_monday_zero_offset() - d.weekday().to_monday_zero_offset()).rem_euclid(7);
    add_days(d, delta as i64).unwrap_or(d)
}

/// org-mode `+2tue` / `-wed`: the nth weekday strictly after (or before) the date.
fn org_weekday(d: Date, n: i64, wd: Weekday) -> Option<Date> {
    let delta =
        (wd.to_monday_zero_offset() - d.weekday().to_monday_zero_offset()).rem_euclid(7) as i64;
    let delta = if n > 0 {
        (if delta == 0 { 7 } else { delta }) + 7 * (n - 1)
    } else {
        delta - 7 + 7 * (n + 1)
    };
    add_days(d, delta)
}

/// A date for the month and day, in the first year from `year` on where it exists (Feb 29).
fn in_year(year: i16, m: i8, d: i8, step: i16) -> Option<Date> {
    (0..8).find_map(|k| Date::new(year + k * step, m, d).ok())
}

/// Fills in a missing year or month. Whole inputs prefer the future (org-mode);
/// dates found in text prefer the current year. Also returns the other candidate.
fn fill(
    y: Option<i16>,
    m: Option<i8>,
    d: Option<i8>,
    today: Date,
    future: bool,
) -> Option<(Date, Vec<Date>)> {
    match (y, m, d) {
        (Some(y), Some(m), d) => Some((Date::new(y, m, d.unwrap_or(1)).ok()?, vec![])),
        (Some(y), None, None) => Some((Date::new(y, 1, 1).ok()?, vec![])),
        (None, Some(m), d) => {
            let day = d.unwrap_or(1);
            let this = in_year(today.year(), m, day, 1)?;
            let passed = if d.is_some() {
                this < today
            } else {
                m < today.month()
            };
            if future && passed {
                Some((in_year(today.year() + 1, m, day, 1)?, vec![this]))
            } else if !future && this > today {
                Some((
                    this,
                    in_year(today.year() - 1, m, day, -1).into_iter().collect(),
                ))
            } else {
                Some((this, vec![]))
            }
        }
        (None, None, Some(d)) => {
            let this = today.with().day(d).build().ok();
            if let Some(t) = this
                && (t >= today || !future)
            {
                return Some((t, vec![]));
            }
            let mut month = today.first_of_month();
            for _ in 0..12 {
                month = month.checked_add(1.month()).ok()?;
                if let Ok(t) = month.with().day(d).build() {
                    return Some((t, this.into_iter().collect()));
                }
            }
            None
        }
        _ => None,
    }
}

/// The next date (or, in text, the previous one) that keeps the given parts of
/// a partial date and falls on the weekday.
fn matching_weekday(d: &DatePart, from: Date, wd: Weekday, future: bool) -> Option<Date> {
    let step = if future { 1 } else { -1 };
    let day = d.day?;
    match (d.year, d.month) {
        (None, Some(m)) => (1..=28).find_map(|k| {
            Date::new(from.year() + k * step, m, day)
                .ok()
                .filter(|c| c.weekday() == wd)
        }),
        (None, None) => {
            let mut month = from.first_of_month();
            for _ in 0..12 * 28 {
                month = month.checked_add(step.months()).ok()?;
                if let Ok(c) = month.with().day(day).build()
                    && c.weekday() == wd
                {
                    return Some(c);
                }
            }
            None
        }
        _ => None,
    }
}

fn weekday_name(wd: Weekday) -> &'static str {
    [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ][wd.to_monday_zero_offset() as usize]
}

pub fn resolve(p: &Parsed, ctx: &Ctx, mode: Mode) -> Option<Resolved> {
    let tz = p
        .zone
        .as_ref()
        .map_or_else(|| ctx.tz.clone(), |(tz, _)| tz.clone());
    let now = ctx.now.with_time_zone(tz.clone());
    let future = mode == Mode::Whole;

    // Relatives without a date ("in 3 hours", "2 days ago") count from this very moment;
    // anything naming a day ("fri", "+3d", "tomorrow") means the start of that day.
    let names_day = p.date.is_some()
        || p.iso_week.is_some()
        || p.ordinal.is_some()
        || p.weekday.is_some()
        || p.day_offset.is_some()
        || p.rels
            .iter()
            .any(|r| matches!(r, Rel::Weekday { .. } | Rel::WeekOf { .. }));
    let from_now = p.rels.iter().any(|r| match r {
        Rel::Units { natural, unit, .. } => *natural || unit.is_sub_day(),
        Rel::Weekday { .. } | Rel::WeekOf { .. } => false,
    });
    let keep_time = p.now || (from_now && !names_day);
    // Days are the user's own days, even for a time given in another zone.
    let today = if keep_time {
        now.date()
    } else {
        ctx.now.date()
    };

    let (mut date, mut alts) = if let Some((y, doy)) = p.ordinal {
        (
            Date::new(y, 1, 1)
                .ok()?
                .with()
                .day_of_year(doy)
                .build()
                .ok()?,
            vec![],
        )
    } else if let Some((y, week)) = p.iso_week {
        let year = y
            .or(p.date.as_ref().and_then(|d| d.year))
            .unwrap_or_else(|| today.iso_week_date().year());
        let wd = p.weekday.unwrap_or(Weekday::Monday);
        (ISOWeekDate::new(year, week, wd).ok()?.date(), vec![])
    } else if let Some(d) = &p.date {
        let (date, mut alts) = fill(d.year, d.month, d.day, today, future)?;
        let ambiguous = d
            .alts
            .iter()
            .filter_map(|&(y, m, day)| fill(y, Some(m), Some(day), today, future));
        alts.splice(0..0, ambiguous.map(|(date, _)| date));
        (date, alts)
    } else if let Some(wd) = p.weekday {
        (next_weekday(today, wd), vec![])
    } else {
        (today, vec![])
    };

    let mut note = None;
    if let (Some(d), Some(wd)) = (&p.date, p.weekday)
        && p.iso_week.is_none()
        && d.day.is_some()
        && date.weekday() != wd
    {
        // A weekday with a partial date picks a date that falls on it: "friday the 13th".
        if let Some(found) = matching_weekday(d, date, wd, future) {
            date = found;
            alts.clear();
        } else {
            note = Some(format!(
                "{date} is a {}, not a {}",
                weekday_name(date.weekday()),
                weekday_name(wd)
            ));
        }
    }

    let shift = |mut d: Date| -> Option<Date> {
        if let Some(n) = p.day_offset {
            d = add_days(d, n as i64)?;
        }
        for r in &p.rels {
            d = match *r {
                Rel::Weekday { n, wd } => org_weekday(d, n, wd)?,
                Rel::WeekOf { weeks, wd } => {
                    let monday = add_days(d, -(d.weekday().to_monday_zero_offset() as i64))?;
                    add_days(monday, 7 * weeks + wd.to_monday_zero_offset() as i64)?
                }
                Rel::Units { n, unit, .. } if !unit.is_sub_day() => {
                    d.checked_add(span(unit, n)?).ok()?
                }
                Rel::Units { .. } => d,
            };
        }
        Some(d)
    };
    let date = shift(date)?;
    let alts: Vec<Date> = alts.into_iter().filter_map(shift).collect();

    let mut sub_day = 0i64;
    for r in &p.rels {
        if let Rel::Units { n, unit, .. } = *r {
            let secs = match unit {
                Unit::Hour => 3600,
                Unit::Minute => 60,
                Unit::Second => 1,
                _ => 0,
            };
            sub_day = sub_day.checked_add(n.checked_mul(secs)?)?;
        }
    }
    let (time, next_day) = match p.time {
        Some(t) => (Time::new(t.h % 24, t.m, t.s, t.ns).ok()?, t.h == 24),
        None if keep_time => (now.time(), false),
        None => (Time::midnight(), false),
    };
    let to_zoned = |d: Date| -> Option<Zoned> {
        let d = if next_day { d.tomorrow().ok()? } else { d };
        let z = d.to_datetime(time).to_zoned(tz.clone()).ok()?;
        z.checked_add(Span::new().try_seconds(sub_day).ok()?).ok()
    };
    let zoned = to_zoned(date)?;
    let mut readings: Vec<Zoned> = Vec::new();
    for z in alts.into_iter().filter_map(to_zoned) {
        if z.timestamp() != zoned.timestamp()
            && !readings.iter().any(|r| r.timestamp() == z.timestamp())
        {
            readings.push(z);
        }
    }
    Some(Resolved {
        zoned,
        explicit_zone: p.zone.is_some(),
        readings,
        live: keep_time && p.time.is_none(),
        note,
    })
}
