//! The user's settings, from `$XDG_CONFIG_HOME/epochup/config.toml` (usually
//! `~/.config/epochup/config.toml`). Everything in it is optional; see
//! `config.toml.example` in the repository.

use std::path::PathBuf;

use eframe::egui::Color32;

/// The colors of the popup. The defaults are the palette of the author's
/// sway, waybar and foot configs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    /// The window.
    pub background: Color32,
    /// The input field.
    pub input: Color32,
    /// What you type.
    pub text: Color32,
    /// The clipboard, shown in the empty input.
    pub placeholder: Color32,
    pub cursor: Color32,
    /// The input's border while picking among the values.
    pub input_border: Color32,
    /// The input's border while typing, and the picked value.
    pub accent: Color32,
    /// The line under the input, the tags on values, outlines on hover.
    pub info: Color32,
    /// The main value.
    pub primary: Color32,
    /// The other values, the language label, the date found in what you typed.
    pub value: Color32,
    /// Behind the other values and the date found in the input.
    pub value_background: Color32,
    /// The picked value's text.
    pub selected_text: Color32,
    /// "no date or epoch found".
    pub error: Color32,
}

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            background: hex(0x191b22),
            input: hex(0x111111),
            text: hex(0xcccccc),
            placeholder: hex(0x6c757d),
            cursor: hex(0xc4e3ff),
            input_border: hex(0x202936),
            accent: hex(0x7d98ec),
            info: hex(0x889ab1),
            primary: hex(0xffffff),
            value: hex(0xc4e3ff),
            value_background: hex(0x202936),
            selected_text: hex(0x041520),
            error: hex(0xe1746d),
        }
    }
}

impl Theme {
    fn color(&mut self, name: &str) -> Option<&mut Color32> {
        Some(match name {
            "background" => &mut self.background,
            "input" => &mut self.input,
            "text" => &mut self.text,
            "placeholder" => &mut self.placeholder,
            "cursor" => &mut self.cursor,
            "input_border" => &mut self.input_border,
            "accent" => &mut self.accent,
            "info" => &mut self.info,
            "primary" => &mut self.primary,
            "value" => &mut self.value,
            "value_background" => &mut self.value_background,
            "selected_text" => &mut self.selected_text,
            "error" => &mut self.error,
            _ => return None,
        })
    }
}

fn path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("epochup").join("config.toml"))
}

/// Reads the config file. Problems are reported on stderr and never stop the
/// popup: whatever can't be used keeps its default.
pub fn load() -> Theme {
    let Some(path) = path() else {
        return Theme::default();
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Theme::default(),
        Err(e) => {
            eprintln!("epochup: {}: {e}", path.display());
            return Theme::default();
        }
    };
    let (theme, problems) = parse(&text);
    for problem in problems {
        eprintln!("epochup: {}: {problem}", path.display());
    }
    theme
}

/// The theme a config file describes, and what was wrong with it.
pub fn parse(text: &str) -> (Theme, Vec<String>) {
    let mut theme = Theme::default();
    let mut problems = Vec::new();
    let table = match text.parse::<toml::Table>() {
        Ok(table) => table,
        Err(e) => {
            problems.push(e.to_string().trim().to_owned());
            return (theme, problems);
        }
    };
    for (key, value) in table {
        let toml::Value::Table(colors) = value else {
            problems.push(format!("unknown setting `{key}`"));
            continue;
        };
        if key != "theme" {
            problems.push(format!("unknown section `[{key}]`"));
            continue;
        }
        for (name, value) in colors {
            match (theme.color(&name), value.as_str().and_then(parse_color)) {
                (Some(slot), Some(color)) => *slot = color,
                (Some(_), None) => problems.push(format!(
                    "`{name}`: expected a color like \"#7d98ec\" or \"#7d98ec80\""
                )),
                (None, _) => problems.push(format!("unknown color `{name}`")),
            }
        }
    }
    (theme, problems)
}

/// `#rrggbb`, or `#rrggbbaa` with transparency.
fn parse_color(s: &str) -> Option<Color32> {
    let hex = s.strip_prefix('#')?;
    if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    match hex.len() {
        6 => Some(Color32::from_rgb(byte(0)?, byte(2)?, byte(4)?)),
        8 => Some(Color32::from_rgba_unmultiplied(
            byte(0)?,
            byte(2)?,
            byte(4)?,
            byte(6)?,
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_is_the_default() {
        let (theme, problems) = parse(include_str!("../config.toml.example"));
        assert_eq!(problems, Vec::<String>::new());
        assert_eq!(theme, Theme::default());
    }

    #[test]
    fn empty_is_the_default() {
        assert_eq!(parse(""), (Theme::default(), vec![]));
    }

    #[test]
    fn partial_themes_keep_the_rest() {
        let (theme, problems) = parse("[theme]\naccent = \"#ff8800\"\n");
        assert!(problems.is_empty());
        assert_eq!(theme.accent, Color32::from_rgb(0xff, 0x88, 0x00));
        assert_eq!(theme.background, Theme::default().background);
    }

    #[test]
    fn mistakes_are_reported_and_skipped() {
        let (theme, problems) = parse(
            "[theme]\naccent = \"orange\"\nbackgruond = \"#000000\"\ntext = \"#123456\"\n[colors]\nx = 1\n",
        );
        assert_eq!(theme.accent, Theme::default().accent);
        assert_eq!(theme.text, Color32::from_rgb(0x12, 0x34, 0x56));
        assert_eq!(problems.len(), 3, "{problems:?}");
        let (theme, problems) = parse("[theme\n");
        assert_eq!(theme, Theme::default());
        assert_eq!(problems.len(), 1);
    }

    #[test]
    fn colors() {
        assert_eq!(
            parse_color("#7d98ec"),
            Some(Color32::from_rgb(0x7d, 0x98, 0xec))
        );
        assert_eq!(
            parse_color("#7d98ec80"),
            Some(Color32::from_rgba_unmultiplied(0x7d, 0x98, 0xec, 0x80))
        );
        for bad in ["7d98ec", "#7d98e", "#gggggg", "", "#"] {
            assert_eq!(parse_color(bad), None, "{bad}");
        }
    }
}
