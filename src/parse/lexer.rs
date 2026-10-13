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

/// Japanese and Chinese words the grammar knows. CJK text has no spaces, so
/// the lexer splits it by longest match against this list. Runs of CJK
/// numerals become numbers, and other characters single-character words.
const CJK_WORDS: &[&str] = &[
    // Japanese
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
    "の",
    // Chinese, simplified and traditional
    "时",
    "点",
    "點",
    "号",
    "號",
    "刻",
    "天",
    "星期",
    "礼拜",
    "禮拜",
    "周",
    "小时",
    "小時",
    "钟头",
    "鐘頭",
    "分钟",
    "分鐘",
    "秒钟",
    "秒鐘",
    "个",
    "個",
    "今天",
    "明天",
    "昨天",
    "后天",
    "後天",
    "前天",
    "大后天",
    "大後天",
    "大前天",
    "现在",
    "現在",
    "上午",
    "下午",
    "早上",
    "早晨",
    "清晨",
    "中午",
    "晚上",
    "傍晚",
    "夜里",
    "夜裡",
    "夜间",
    "夜間",
    "凌晨",
    "半夜",
    "午夜",
    "下周",
    "下週",
    "上周",
    "上週",
    "这周",
    "這週",
    "本周",
    "本週",
    "下个",
    "下個",
    "上个",
    "上個",
    "这个",
    "這個",
    "明年",
    "后",
    "以后",
    "之后",
    "以後",
    "之後",
    "以前",
    "之前",
    "民国",
    "民國",
    "西元",
    "公元",
    "的",
    "在",
];

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
        _ => c,
    }
}

pub fn is_cjk(c: char) -> bool {
    matches!(c,
        '〇' | '\u{3040}'..='\u{30FF}' | '\u{3400}'..='\u{4DBF}' | '\u{4E00}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}')
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
            let known = CJK_WORDS
                .iter()
                .filter(|w| rest.starts_with(**w))
                .max_by_key(|w| w.len())
                .copied();
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
            let (kind, text) = match number {
                Some((val, digits)) => (Kind::Num { val, digits }, numeral),
                None => {
                    let word = known.unwrap_or(&rest[..raw.len_utf8()]);
                    (Kind::Word(word.to_string()), word)
                }
            };
            for _ in text.chars() {
                it.next();
            }
            out.push(Token {
                kind,
                start,
                end: start + text.len(),
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
