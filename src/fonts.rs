//! egui's fonts plus a CJK fallback, since `LC_TIME` may well print 年月日.

use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

/// Common locations of Noto Sans CJK; face 0 of the collection is the Japanese one.
const CANDIDATES: &[&str] = &[
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Medium.ttc",
];

/// Reads a CJK font (about 20 MB, so call it on a thread while the window opens).
pub fn load_cjk() -> Option<FontData> {
    let (bytes, index) = CANDIDATES
        .iter()
        .find_map(|path| Some((std::fs::read(path).ok()?, 0)))
        .or_else(fc_match)?;
    let mut data = FontData::from_owned(bytes);
    data.index = index;
    Some(data)
}

fn fc_match() -> Option<(Vec<u8>, u32)> {
    let out = std::process::Command::new("fc-match")
        .args(["-f", "%{file}\n%{index}", "sans-serif:lang=ja"])
        .output()
        .ok()?;
    let out = String::from_utf8(out.stdout).ok()?;
    let mut lines = out.lines();
    let path = lines.next()?;
    let index = lines.next()?.parse().ok()?;
    Some((std::fs::read(path).ok()?, index))
}

pub fn install(ctx: &egui::Context, cjk: Option<FontData>) {
    let mut fonts = FontDefinitions::default();
    if let Some(data) = cjk {
        fonts.font_data.insert("cjk".into(), Arc::new(data));
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            fonts.families.entry(family).or_default().push("cjk".into());
        }
    }
    ctx.set_fonts(fonts);
}
