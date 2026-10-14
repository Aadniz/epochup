//! egui's fonts plus fallbacks for scripts they lack: CJK (`LC_TIME` may well
//! print 年月日, and it covers Korean Hangul) and Thai.

use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

/// A fallback font: where it usually is, and what to ask fontconfig otherwise.
struct Fallback {
    name: &'static str,
    /// Common locations, with the face to use in a font collection.
    paths: &'static [(&'static str, u32)],
    fontconfig: &'static str,
}

const FALLBACKS: &[Fallback] = &[
    Fallback {
        name: "cjk",
        // Face 0 of the Noto Sans CJK collection is the Japanese one.
        paths: &[
            ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 0),
            ("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", 0),
            (
                "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
                0,
            ),
            ("/usr/share/fonts/opentype/noto/NotoSansCJK-Medium.ttc", 0),
        ],
        fontconfig: "sans-serif:lang=ja",
    },
    Fallback {
        name: "thai",
        paths: &[
            ("/usr/share/fonts/truetype/noto/NotoSansThai-Regular.ttf", 0),
            ("/usr/share/fonts/noto/NotoSansThai-Regular.ttf", 0),
            ("/usr/share/fonts/google-noto/NotoSansThai-Regular.ttf", 0),
        ],
        fontconfig: "sans-serif:lang=th",
    },
];

/// Reads the fallback fonts (the CJK one is about 20 MB, so call this on a
/// thread while the window opens).
pub fn load() -> Vec<(&'static str, FontData)> {
    FALLBACKS
        .iter()
        .filter_map(|f| {
            let (bytes, index) = f
                .paths
                .iter()
                .find_map(|&(path, index)| Some((std::fs::read(path).ok()?, index)))
                .or_else(|| fc_match(f.fontconfig))?;
            let mut data = FontData::from_owned(bytes);
            data.index = index;
            Some((f.name, data))
        })
        .collect()
}

fn fc_match(pattern: &str) -> Option<(Vec<u8>, u32)> {
    let out = std::process::Command::new("fc-match")
        .args(["-f", "%{file}\n%{index}", pattern])
        .output()
        .ok()?;
    let out = String::from_utf8(out.stdout).ok()?;
    let mut lines = out.lines();
    let path = lines.next()?;
    let index = lines.next()?.parse().ok()?;
    Some((std::fs::read(path).ok()?, index))
}

pub fn install(ctx: &egui::Context, fallbacks: Vec<(&'static str, FontData)>) {
    let mut fonts = FontDefinitions::default();
    for (name, data) in fallbacks {
        fonts.font_data.insert(name.into(), Arc::new(data));
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            fonts.families.entry(family).or_default().push(name.into());
        }
    }
    ctx.set_fonts(fonts);
}
