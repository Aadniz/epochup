//! Splits input into numbers, words and symbols, keeping byte spans into the original text.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A run of ASCII (or fullwidth) digits. `val` saturates for very long runs.
    Num { val: u64, digits: u8 },
    /// A lowercased run of letters, or a known Japanese word.
    Word(String),
    /// Any other non-whitespace character, normalized to ASCII where a fullwidth form exists.
    Sym(char),
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: Kind,
    pub start: usize,
    pub end: usize,
}

/// Japanese words the grammar knows. CJK text has no spaces, so the lexer
/// splits it by longest match against this list (unknown characters become
/// single-character words).
const JA_WORDS: &[&str] = &[
    "年",
    "月",
    "日",
    "時",
    "分",
    "秒",
    "半",
    "午前",
    "午後",
    "正午",
    "曜日",
    "月曜日",
    "火曜日",
    "水曜日",
    "木曜日",
    "金曜日",
    "土曜日",
    "日曜日",
    "月曜",
    "火曜",
    "水曜",
    "木曜",
    "金曜",
    "土曜",
    "日曜",
    "今日",
    "本日",
    "明日",
    "昨日",
    "明後日",
    "一昨日",
    "今",
    "後",
    "前",
    "来週",
    "先週",
    "今週",
    "来月",
    "先月",
    "今月",
    "来年",
    "去年",
    "昨年",
    "今年",
    "週間",
    "週",
    "ヶ月",
    "か月",
    "カ月",
    "ケ月",
    "箇月",
    "時間",
    "令和",
    "平成",
    "昭和",
    "大正",
    "明治",
    "元",
    "あした",
    "あす",
    "きのう",
    "きょう",
    "あさって",
    "おととい",
];

/// Maps fullwidth ASCII and typographic dashes to their plain ASCII forms.
pub fn normalize(c: char) -> char {
    match c {
        '\u{FF01}'..='\u{FF5E}' => char::from_u32(c as u32 - 0xFEE0).unwrap_or(c),
        '\u{2010}'..='\u{2015}' | '\u{2212}' => '-',
        _ => c,
    }
}

pub fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{3040}'..='\u{30FF}' | '\u{3400}'..='\u{4DBF}' | '\u{4E00}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}')
}

pub fn lex(src: &str) -> Vec<Token> {
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
            });
        } else if is_cjk(c) {
            let rest = &src[start..];
            let word = JA_WORDS
                .iter()
                .filter(|w| rest.starts_with(**w))
                .max_by_key(|w| w.len())
                .copied()
                .unwrap_or(&rest[..raw.len_utf8()]);
            for _ in word.chars() {
                it.next();
            }
            out.push(Token {
                kind: Kind::Word(word.to_string()),
                start,
                end: start + word.len(),
            });
        } else if c.is_alphabetic() {
            let (mut word, mut end) = (String::new(), start);
            while let Some(&(j, r)) = it.peek() {
                let d = normalize(r);
                if !d.is_alphabetic() || is_cjk(d) {
                    break;
                }
                word.extend(d.to_lowercase());
                end = j + r.len_utf8();
                it.next();
            }
            out.push(Token {
                kind: Kind::Word(word),
                start,
                end,
            });
        } else {
            it.next();
            out.push(Token {
                kind: Kind::Sym(c),
                start,
                end: start + raw.len_utf8(),
            });
        }
    }
    out
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
