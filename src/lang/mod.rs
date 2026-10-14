//! Vocabulary: what words mean, one module per language.
//!
//! The grammar in `parse::expr` never matches words itself. It asks the
//! [`Lexicon`] what a word can mean ("is this a month name?", "does this say
//! *ago*?"), so supporting a language means listing its words here, plus its
//! tests next to them. Words are lowercase. A key with spaces or punctuation
//! (`from now`, `après-demain`) is a phrase: the lexer joins it into one token.
//! Words in scripts written without spaces (Chinese, Japanese, Korean
//! particles, Thai) also drive how the lexer splits such text.

mod de;
mod en;
mod es;
mod fr;
mod it;
mod ja;
mod ko;
mod nl;
mod pl;
mod pt;
mod ru;
mod th;
mod zh;

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::OnceLock;

use jiff::civil::Weekday;

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

/// A part of the day said before the hour, as in 下午3点 or 午後3時.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Period {
    /// Morning: 12 is midnight.
    Am,
    /// Afternoon: 1 to 11 are after noon.
    Pm,
    /// Midday (中午): 11 and 12 as they are, 1 to 3 after noon.
    Noon,
    /// Evening and night (晚上): 6 to 11 are after noon, 12 is midnight, 1 to 5 the small hours.
    Night,
}

impl Period {
    /// The 24-hour clock hour for an hour said in this part of the day.
    pub fn hour(self, h: i64) -> i64 {
        match self {
            Period::Am if h == 12 => 0,
            Period::Pm if (1..=11).contains(&h) => h + 12,
            Period::Noon if h < 11 => h + 12,
            Period::Night if (6..=11).contains(&h) => h + 12,
            Period::Night if h == 12 => 0,
            _ => h,
        }
    }
}

/// What a word can mean to the grammar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Meaning {
    MonthName(i8),
    WeekdayName(Weekday),
    /// A weekday only inside parentheses: the 火 of 13日(火).
    ParenWeekday(Weekday),
    UnitName(Unit),
    Now,
    /// Days from today: today 0, tomorrow 1, yesterday -1.
    DayOffset(i8),
    /// Said before an hour to place it in the day: 下午, 午後.
    DayPart(Period),
    /// A time of day on its own: noon is 12, midnight 0.
    Clock(i8),
    /// A Japanese era (or Taiwan's 民國), as the Gregorian year before its first year.
    Era(i16),
    /// The 元 of 令和元年: the first year of an era.
    FirstYear,
    /// 西元: "Common Era", written before a year.
    CommonEra,
    /// Words that may sit between date parts without meaning anything: `at`, `the`, の.
    Filler,
    /// Before a count, the future: `in 3 days`.
    In,
    /// Before a count, the past: `vor 3 Tagen`.
    Before,
    /// After a count, the past: `3 days ago`, 3日前.
    Ago,
    /// After a count, the future: `3 days later`, `3 days from now`, 3日後.
    Later,
    /// A number written as a word: `a week ago` (1), `два дня` (2), 오후 세 시 (3).
    Count(u8),
    /// A unit that means one of itself: `через час` (in an hour), `tydzień temu` (a week ago).
    SingleUnit(Unit),
    /// Joins counts: `2 hours and 30 minutes`.
    And,
    /// Half: 半小时 (half an hour), 3点半 (half past three).
    Half,
    /// A Chinese measure word between a count and its unit: 3个月.
    Measure,
    /// Before a weekday or unit: next (1), last (-1), this (0), the one after next (2).
    Next(i8),
    /// Next, last or this before a Chinese measure word and unit: 下个月, 上个星期.
    NextMeasured(i8),
    /// A whole unit away: 来週 (next week), 去年 (last year).
    Shift(Unit, i8),
    /// The word before a Chinese weekday number: the 星期 of 星期二.
    WeekWord,
    /// Sunday, after a Chinese week word: the 日 of 星期日.
    WeekSunday,
    /// Before an ISO week number: `KW 42`, `week 42`.
    WeekNumber,
    /// The CJK markers after a number: 年 月 日 時 分 秒.
    YearMark,
    MonthMark,
    DayMark,
    HourMark,
    MinuteMark,
    SecondMark,
    /// A quarter hour: 3点一刻.
    Quarter,
    /// After a number, an hour counted from this one: Thai 2 ทุ่ม is 20:00.
    HourFrom(i8),
    /// A word that only matters for splitting text without spaces: listing
    /// 오늘날 ("nowadays") keeps it from being read as 오늘 ("today") + 날.
    Other,
    /// After the hour: am (false) or pm (true).
    AmPm(bool),
    /// After the hour: `15 Uhr`.
    OClock,
    /// Before a bare hour: `at 9`, `um 9`.
    At,
    /// After a day number: `13th`.
    OrdinalSuffix,
    /// Between a day and a month: `13th of October`.
    Of,
}

pub struct Lang {
    pub code: &'static str,
    /// The language's name for itself: 日本語.
    pub name: &'static str,
    /// Shown under the input when a date was read in this language, in it:
    /// 日本語の日付 ("date in Japanese"), `Datum auf Deutsch`.
    pub label: &'static str,
    /// Years written in this language may count from another era: Thai 2569
    /// is 2026, so this is 543. Applies to years past 2200 next to its words.
    pub year_offset: i16,
    pub words: &'static [(&'static str, Meaning)],
    /// Words that are also ordinary words (`may`, German `do`): they only count
    /// in text when something else confirms them.
    pub weak: &'static [(&'static str, Meaning)],
}

/// Every language the parser knows.
pub const LANGS: &[&Lang] = &[
    &en::LANG,
    &de::LANG,
    &fr::LANG,
    &es::LANG,
    &it::LANG,
    &nl::LANG,
    &pt::LANG,
    &ja::LANG,
    &zh::LANG,
    &ko::LANG,
    &ru::LANG,
    &pl::LANG,
    &th::LANG,
];

#[derive(Clone, Copy, Debug)]
pub struct Entry {
    pub meaning: Meaning,
    pub lang: &'static str,
    pub strong: bool,
    pub year_offset: i16,
}

/// A piece of a phrase key: `après-demain` is `après`, a glued `-`, a glued `demain`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Part {
    Word(String, bool),
    Sym(char, bool),
}

#[derive(Debug)]
pub struct Phrase {
    pub key: &'static str,
    /// The parts after the first word.
    pub rest: Vec<Part>,
}

/// Everything known about one word: what it means, and the phrases it starts.
#[derive(Default)]
pub struct Word {
    pub entries: Vec<Entry>,
    /// Longest first.
    pub phrases: Vec<Phrase>,
}

pub struct Lexicon {
    words: HashMap<String, Word, BuildHasherDefault<Fx>>,
    /// The words of scripts written without spaces, for splitting by longest match.
    unspaced: Trie,
    /// The era of the user's own language, if it has one (Thai).
    default_year_offset: i16,
    /// Language codes, most preferred first: the user's, then the listed order.
    ranking: Vec<&'static str>,
}

/// A character trie: finds the longest known word at the start of some text in
/// time proportional to that word, however many words there are.
#[derive(Default)]
struct Trie {
    nodes: Vec<Node>,
}

#[derive(Default)]
struct Node {
    /// Sorted by character.
    next: Vec<(char, u32)>,
    word: Option<&'static str>,
}

impl Trie {
    fn insert(&mut self, word: &'static str) {
        if self.nodes.is_empty() {
            self.nodes.push(Node::default());
        }
        let mut at = 0;
        for c in word.chars() {
            at = match self.nodes[at].next.binary_search_by_key(&c, |&(k, _)| k) {
                Ok(k) => self.nodes[at].next[k].1 as usize,
                Err(k) => {
                    let new = self.nodes.len();
                    self.nodes.push(Node::default());
                    self.nodes[at].next.insert(k, (c, new as u32));
                    new
                }
            };
        }
        self.nodes[at].word = Some(word);
    }

    fn longest(&self, text: &str) -> Option<&'static str> {
        let mut at = 0;
        let mut found = None;
        for c in text.chars() {
            let node = self.nodes.get(at)?;
            let Ok(k) = node.next.binary_search_by_key(&c, |&(k, _)| k) else {
                break;
            };
            at = node.next[k].1 as usize;
            found = self.nodes[at].word.or(found);
        }
        found
    }
}

/// FxHash (from rustc): much faster than the default hash for short keys. The
/// keys are the lexicon's own fixed words, so attack resistance buys nothing.
#[derive(Default)]
pub struct Fx(u64);

impl Fx {
    fn add(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }
}

impl Hasher for Fx {
    fn write(&mut self, bytes: &[u8]) {
        let mut chunks = bytes.chunks_exact(8);
        for chunk in &mut chunks {
            self.add(u64::from_le_bytes(chunk.try_into().expect("8 bytes")));
        }
        let mut rest = [0u8; 8];
        rest[..chunks.remainder().len()].copy_from_slice(chunks.remainder());
        self.add(u64::from_le_bytes(rest));
    }

    fn write_u8(&mut self, byte: u8) {
        self.add(byte as u64);
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

/// Scripts the lexer splits by dictionary rather than by spaces: Chinese and
/// Japanese, Korean Hangul (whose particles are glued to words), and Thai.
pub fn is_unspaced(c: char) -> bool {
    matches!(c,
        '〇' | '\u{3040}'..='\u{30FF}' | '\u{3400}'..='\u{4DBF}' | '\u{4E00}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}'
        | '\u{AC00}'..='\u{D7A3}' | '\u{1100}'..='\u{11FF}' | '\u{3130}'..='\u{318F}'
        | '\u{0E00}'..='\u{0E7F}')
}

/// Splits a phrase key into words and symbols, noting which are glued to the one before.
fn parts(key: &str) -> Vec<Part> {
    let mut parts = Vec::new();
    let mut glued = false;
    let mut chars = key.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            glued = false;
            chars.next();
        } else if c.is_alphanumeric() {
            let mut word = String::new();
            while let Some(&c) = chars.peek().filter(|c| c.is_alphanumeric()) {
                word.push(c);
                chars.next();
            }
            parts.push(Part::Word(word, glued));
            glued = true;
        } else {
            parts.push(Part::Sym(c, glued));
            chars.next();
            glued = true;
        }
    }
    parts
}

impl Lexicon {
    /// Builds the lexicon; among meanings of the same word, those of the
    /// `preferred` languages come first.
    pub fn new(langs: &[&Lang], preferred: &[String]) -> Lexicon {
        let rank = |code: &str| {
            preferred
                .iter()
                .position(|p| p == code)
                .unwrap_or(preferred.len())
        };
        let mut words: HashMap<String, Word, BuildHasherDefault<Fx>> = HashMap::default();
        let all = || {
            langs.iter().flat_map(|lang| {
                let strong = lang.words.iter().map(move |w| (*lang, *w, true));
                let weak = lang.weak.iter().map(move |w| (*lang, *w, false));
                strong.chain(weak)
            })
        };
        for (lang, (key, meaning), strong) in all() {
            let entry = Entry {
                meaning,
                lang: lang.code,
                strong,
                year_offset: lang.year_offset,
            };
            words
                .entry(key.to_string())
                .or_default()
                .entries
                .push(entry);
        }
        for word in words.values_mut() {
            word.entries.sort_by_key(|e| rank(e.lang));
        }
        let mut unspaced = Trie::default();
        for (_, (key, _), _) in all() {
            if key.starts_with(is_unspaced) {
                unspaced.insert(key);
            }
            let mut p = parts(key).into_iter();
            if let (Some(Part::Word(first, _)), rest) = (p.next(), p.collect::<Vec<_>>())
                && !rest.is_empty()
            {
                let phrases = &mut words.entry(first).or_default().phrases;
                if !phrases.iter().any(|p| p.key == key) {
                    phrases.push(Phrase { key, rest });
                    phrases.sort_by_key(|p| std::cmp::Reverse(p.rest.len()));
                }
            }
        }
        let default_year_offset = langs
            .iter()
            .filter(|l| preferred.iter().any(|p| p == l.code))
            .map(|l| l.year_offset)
            .find(|&o| o != 0)
            .unwrap_or(0);
        let mut ranking: Vec<&'static str> = langs.iter().map(|l| l.code).collect();
        ranking.sort_by_key(|code| rank(code));
        Lexicon {
            words,
            unspaced,
            default_year_offset,
            ranking,
        }
    }

    /// Everything the word can mean, preferred languages first.
    pub fn word(&self, word: &str) -> Option<&Word> {
        self.words.get(word)
    }

    pub fn meanings(&self, word: &str) -> &[Entry] {
        self.word(word).map_or(&[], |w| &w.entries)
    }

    /// The first meaning `pick` accepts, and whether that entry is strong.
    pub fn find<T>(&self, word: &str, pick: impl Fn(Meaning) -> Option<T>) -> Option<(T, bool)> {
        self.meanings(word)
            .iter()
            .find_map(|e| pick(e.meaning).map(|t| (t, e.strong)))
    }

    pub fn has(&self, word: &str, meaning: Meaning) -> bool {
        self.meanings(word).iter().any(|e| e.meaning == meaning)
    }

    /// Phrases starting with this word, longest first.
    pub fn phrases(&self, first: &str) -> &[Phrase] {
        self.word(first).map_or(&[], |w| &w.phrases)
    }

    /// Where a language stands when several fit equally: lower is preferred.
    pub fn rank(&self, code: &str) -> usize {
        self.ranking
            .iter()
            .position(|c| *c == code)
            .unwrap_or(usize::MAX)
    }

    /// The era of the user's own language: Thai users write 2569 for 2026.
    pub fn default_year_offset(&self) -> i16 {
        self.default_year_offset
    }

    /// The longest known word at the start of text written without spaces.
    pub fn unspaced_word(&self, text: &str) -> Option<&'static str> {
        self.unspaced.longest(text)
    }
}

/// The languages of the user's locale settings, most important first:
/// `LANGUAGE`, then `LC_ALL`, `LC_TIME`, `LC_MESSAGES`, `LANG` and the other `LC_*`.
pub fn preferred_languages() -> Vec<String> {
    let mut vars: Vec<String> = Vec::new();
    if let Ok(list) = std::env::var("LANGUAGE") {
        vars.extend(list.split(':').map(str::to_owned));
    }
    for name in ["LC_ALL", "LC_TIME", "LC_MESSAGES", "LANG"] {
        vars.extend(std::env::var(name).ok());
    }
    let mut others: Vec<(String, String)> = std::env::vars()
        .filter(|(k, _)| k.starts_with("LC_"))
        .collect();
    others.sort();
    vars.extend(others.into_iter().map(|(_, v)| v));
    let mut langs: Vec<String> = Vec::new();
    for v in vars {
        let code: String = v
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .collect::<String>()
            .to_lowercase();
        if code.len() >= 2 && code != "c" && code != "posix" && !langs.contains(&code) {
            langs.push(code);
        }
    }
    langs
}

/// A language's name for itself, from its code: `ja` is 日本語.
pub fn name(code: &str) -> Option<&'static str> {
    LANGS.iter().find(|l| l.code == code).map(|l| l.name)
}

/// What to show when a date was read in a language, from its code: `ja` is 日本語の日付.
pub fn label(code: &str) -> Option<&'static str> {
    LANGS.iter().find(|l| l.code == code).map(|l| l.label)
}

/// The lexicon of all languages, preferring the user's.
pub fn lexicon() -> &'static Lexicon {
    static LEXICON: OnceLock<Lexicon> = OnceLock::new();
    LEXICON.get_or_init(|| Lexicon::new(LANGS, &preferred_languages()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn languages_name_themselves() {
        for lang in LANGS {
            assert!(!lang.name.is_empty(), "{}", lang.code);
            assert!(!lang.label.is_empty(), "{}", lang.code);
            assert_eq!(
                LANGS.iter().filter(|l| l.code == lang.code).count(),
                1,
                "{}",
                lang.code
            );
        }
        assert_eq!(name("ja"), Some("日本語"));
        assert_eq!(name("xx"), None);
    }

    #[test]
    fn keys_are_lowercase_and_trimmed() {
        for lang in LANGS {
            for (key, _) in lang.words.iter().chain(lang.weak) {
                assert_eq!(*key, key.to_lowercase(), "{}: {key:?}", lang.code);
                assert_eq!(*key, key.trim(), "{}: {key:?}", lang.code);
            }
        }
    }

    #[test]
    fn same_meaning_same_strength() {
        // If two languages share a word and meaning, they must agree on whether it is weak.
        let lexicon = Lexicon::new(LANGS, &[]);
        for (word, slot) in &lexicon.words {
            for a in &slot.entries {
                for b in slot.entries.iter().filter(|b| b.meaning == a.meaning) {
                    assert_eq!(
                        a.strong, b.strong,
                        "{word:?} {:?} in {} and {}",
                        a.meaning, a.lang, b.lang
                    );
                }
            }
        }
    }

    #[test]
    fn preferred_languages_win() {
        const A: Lang = Lang {
            code: "aa",
            name: "A",
            label: "a",
            year_offset: 0,
            words: &[("mar", Meaning::MonthName(3))],
            weak: &[],
        };
        const B: Lang = Lang {
            code: "bb",
            name: "B",
            label: "b",
            year_offset: 0,
            words: &[("mar", Meaning::WeekdayName(Weekday::Tuesday))],
            weak: &[],
        };
        let first = |pref: &[&str]| {
            let pref: Vec<String> = pref.iter().map(|s| s.to_string()).collect();
            Lexicon::new(&[&A, &B], &pref).meanings("mar")[0].lang
        };
        assert_eq!(first(&["bb"]), "bb");
        assert_eq!(first(&["aa", "bb"]), "aa");
        assert_eq!(first(&[]), "aa", "without a preference, the listed order");
    }

    #[test]
    fn phrases_and_unspaced_words() {
        let lexicon = lexicon();
        assert!(lexicon.phrases("from").iter().any(|p| p.key == "from now"));
        assert_eq!(lexicon.unspaced_word("一昨日です"), Some("一昨日"));
        assert_eq!(lexicon.unspaced_word("明天下午"), Some("明天"));
        assert_eq!(lexicon.unspaced_word("내일은"), Some("내일"));
        assert_eq!(lexicon.unspaced_word("다음 주 월요일"), Some("다음 주"));
        assert_eq!(lexicon.unspaced_word("กุมภาพันธ์ 2569"), Some("กุมภาพันธ์"));
        assert_eq!(lexicon.unspaced_word("ต.ค. 2569"), Some("ต.ค."));
        assert_eq!(lexicon.unspaced_word("龍"), None);
        assert_eq!(
            parts("après-demain"),
            [
                Part::Word("après".into(), false),
                Part::Sym('-', true),
                Part::Word("demain".into(), true)
            ]
        );
    }

    #[test]
    fn thai_users_get_buddhist_years() {
        assert_eq!(
            Lexicon::new(LANGS, &["th".to_string()]).default_year_offset(),
            543
        );
        assert_eq!(
            Lexicon::new(LANGS, &["en".to_string(), "ja".to_string()]).default_year_offset(),
            0
        );
    }
}
