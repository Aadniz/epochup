//! Splits input into numbers, words and symbols, keeping byte spans into the original text.

use crate::lang::{Entry, Part, Phrase, Word, is_unspaced, lexicon};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A run of ASCII (or fullwidth) digits. `val` saturates for very long runs.
    Num { val: u64, digits: u8 },
    /// A lowercased run of letters, a known CJK word, or a known phrase.
    Word(String),
    /// Any other non-whitespace character, normalized to ASCII where a fullwidth form exists.
    Sym(char),
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: Kind,
    pub start: usize,
    pub end: usize,
    /// What a word can mean, looked up once here so the grammar never hashes.
    pub meanings: &'static [Entry],
    /// Phrases this word may start.
    phrases: &'static [Phrase],
}

/// A CJK digit: 〇 一 二 … 九, and 两 for two.
fn cjk_digit(c: char) -> Option<u64> {
    Some(match c {
        '〇' | '零' => 0,
        '一' => 1,
        '二' | '两' | '兩' => 2,
        '三' => 3,
        '四' => 4,
        '五' => 5,
        '六' => 6,
        '七' => 7,
        '八' => 8,
        '九' => 9,
        _ => return None,
    })
}

fn is_cjk_numeral(c: char) -> bool {
    cjk_digit(c).is_some() || matches!(c, '十' | '百' | '千')
}

/// The value and digit count of a CJK numeral: digit by digit (`二〇二六` is
/// 2026) or with tens and hundreds (`二十四` is 24, `十三` is 13).
fn cjk_number(s: &str) -> Option<(u64, u8)> {
    if !s.contains(['十', '百', '千']) {
        let val = s
            .chars()
            .try_fold(0u64, |v, c| Some(v.checked_mul(10)? + cjk_digit(c)?))?;
        return Some((val, s.chars().count().min(u8::MAX as usize) as u8));
    }
    let (mut total, mut current) = (0u64, 0u64);
    for c in s.chars() {
        let unit = match c {
            '十' => 10,
            '百' => 100,
            '千' => 1000,
            _ => {
                current = cjk_digit(c)?;
                continue;
            }
        };
        total += current.max(1) * unit;
        current = 0;
    }
    let val = total + current;
    Some((val, val.to_string().len() as u8))
}

/// Maps fullwidth ASCII and typographic dashes to their plain ASCII forms.
pub fn normalize(c: char) -> char {
    match c {
        '\u{FF01}'..='\u{FF5E}' => char::from_u32(c as u32 - 0xFEE0).unwrap_or(c),
        '\u{2010}'..='\u{2015}' | '\u{2212}' => '-',
        // Thai digits
        '\u{0E50}'..='\u{0E59}' => char::from_u32(c as u32 - 0x0E50 + '0' as u32).unwrap_or(c),
        _ => c,
    }
}

/// Splits text into tokens. CJK text has no spaces, so it is split by
/// longest match against the lexicon's words; runs of CJK numerals become
/// numbers, and other characters single-character words. Phrases from the
/// lexicon (`from now`) are joined into one word token.
pub fn lex(src: &str) -> Vec<Token> {
    join_phrases(split(src))
}

fn split(src: &str) -> Vec<Token> {
    let lexicon = lexicon();
    let mut out = Vec::new();
    let mut it = src.char_indices().peekable();
    while let Some(&(start, raw)) = it.peek() {
        let c = normalize(raw);
        if c.is_whitespace() {
            it.next();
        } else if c.is_ascii_digit() {
            let (mut val, mut digits, mut end) = (0u64, 0u32, start);
            while let Some(&(j, r)) = it.peek() {
                let d = normalize(r);
                if !d.is_ascii_digit() {
                    break;
                }
                val = val.saturating_mul(10).saturating_add(d as u64 - '0' as u64);
                digits += 1;
                end = j + r.len_utf8();
                it.next();
            }
            let digits = digits.min(u8::MAX as u32) as u8;
            out.push(Token {
                kind: Kind::Num { val, digits },
                start,
                end,
                meanings: &[],
                phrases: &[],
            });
        } else if is_unspaced(c) {
            let rest = &src[start..];
            let known = lexicon.unspaced_word(rest);
            let numeral_len: usize = rest
                .chars()
                .take_while(|&c| is_cjk_numeral(c))
                .map(char::len_utf8)
                .sum();
            // A known word wins over a numeral it starts with: 一昨日 is not 1 + 昨日.
            let numeral = &rest[..numeral_len];
            let number = (numeral_len > known.map_or(0, str::len))
                .then(|| cjk_number(numeral))
                .flatten();
            let (kind, text, known) = match number {
                Some((val, digits)) => (Kind::Num { val, digits }, numeral, None),
                None => {
                    let word = known.unwrap_or(&rest[..raw.len_utf8()]);
                    (Kind::Word(word.to_string()), word, lexicon.word(word))
                }
            };
            for _ in text.chars() {
                it.next();
            }
            out.push(Token {
                kind,
                start,
                end: start + text.len(),
                meanings: known.map_or(&[], |w| &w.entries),
                phrases: known.map_or(&[], |w| &w.phrases),
            });
        } else if c.is_alphabetic() {
            let (mut word, mut end) = (String::new(), start);
            while let Some(&(j, r)) = it.peek() {
                let d = normalize(r);
                if !d.is_alphabetic() || is_unspaced(d) {
                    break;
                }
                word.extend(d.to_lowercase());
                end = j + r.len_utf8();
                it.next();
            }
            let known: Option<&'static Word> = lexicon.word(&word);
            out.push(Token {
                kind: Kind::Word(word),
                start,
                end,
                meanings: known.map_or(&[], |w| &w.entries),
                phrases: known.map_or(&[], |w| &w.phrases),
            });
        } else {
            it.next();
            out.push(Token {
                kind: Kind::Sym(c),
                start,
                end: start + raw.len_utf8(),
                meanings: &[],
                phrases: &[],
            });
        }
    }
    out
}

/// Joins the tokens of a known phrase (`from now`, `il y a`, `après-demain`) into one word.
fn join_phrases(tokens: Vec<Token>) -> Vec<Token> {
    let lexicon = lexicon();
    // Most text has no phrases at all: find them first, and only then rebuild.
    let mut joins: Vec<(usize, usize, &'static str)> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let phrase = tokens[i].phrases.iter().find(|p| matches_at(&tokens, i, p));
        match phrase {
            Some(p) => {
                joins.push((i, i + p.rest.len(), p.key));
                i += p.rest.len() + 1;
            }
            None => i += 1,
        }
    }
    if joins.is_empty() {
        return tokens;
    }
    let mut out = Vec::with_capacity(tokens.len());
    let mut joins = joins.into_iter().peekable();
    let mut phrase_start = 0;
    for (k, token) in tokens.into_iter().enumerate() {
        match joins.peek() {
            Some(&(first, last, key)) if k >= first => {
                if k == first {
                    phrase_start = token.start;
                }
                if k == last {
                    let known = lexicon.word(key);
                    out.push(Token {
                        kind: Kind::Word(key.to_string()),
                        start: phrase_start,
                        end: token.end,
                        meanings: known.map_or(&[], |w| &w.entries),
                        phrases: &[],
                    });
                    joins.next();
                }
            }
            _ => out.push(token),
        }
    }
    out
}

fn matches_at(tokens: &[Token], i: usize, phrase: &Phrase) -> bool {
    phrase.rest.iter().enumerate().all(|(k, part)| {
        let Some(t) = tokens.get(i + 1 + k) else {
            return false;
        };
        let glued = t.start == tokens[i + k].end;
        match (part, &t.kind) {
            (Part::Word(w, g), Kind::Word(tw)) => w == tw && *g == glued,
            (Part::Sym(c, g), Kind::Sym(tc)) => c == tc && *g == glued,
            _ => false,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(s: &str) -> Vec<Kind> {
        lex(s).into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn splits_iso() {
        let k = kinds("2026-10-13T21:24");
        assert_eq!(k.len(), 9);
        assert_eq!(
            k[0],
            Kind::Num {
                val: 2026,
                digits: 4
            }
        );
        assert_eq!(k[5], Kind::Word("t".into()));
    }

    #[test]
    fn japanese_longest_match() {
        let k = kinds("13日火曜日 3時間後");
        assert_eq!(
            k,
            vec![
                Kind::Num { val: 13, digits: 2 },
                Kind::Word("日".into()),
                Kind::Word("火曜日".into()),
                Kind::Num { val: 3, digits: 1 },
                Kind::Word("時間".into()),
                Kind::Word("後".into()),
            ]
        );
    }

    #[test]
    fn cjk_numerals() {
        assert_eq!(cjk_number("二〇二六"), Some((2026, 4)));
        assert_eq!(cjk_number("十三"), Some((13, 2)));
        assert_eq!(cjk_number("二十四"), Some((24, 2)));
        assert_eq!(cjk_number("十"), Some((10, 2)));
        assert_eq!(cjk_number("两"), Some((2, 1)));
        assert_eq!(cjk_number("二千零二十六"), Some((2026, 4)));
        let k = kinds("十月十三日 一昨日");
        assert_eq!(
            k,
            vec![
                Kind::Num { val: 10, digits: 2 },
                Kind::Word("月".into()),
                Kind::Num { val: 13, digits: 2 },
                Kind::Word("日".into()),
                Kind::Word("一昨日".into()),
            ]
        );
    }

    #[test]
    fn fullwidth_digits() {
        let k = kinds("２０２６年");
        assert_eq!(
            k[0],
            Kind::Num {
                val: 2026,
                digits: 4
            }
        );
    }

    #[test]
    fn spans_are_bytes() {
        let t = lex("Oktober März");
        assert_eq!(t[1].kind, Kind::Word("märz".into()));
        assert_eq!(&"Oktober März"[t[1].start..t[1].end], "März");
    }
}
