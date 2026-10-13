//! The popup: one input field, the conversion underneath.

use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

use eframe::egui::text::LayoutJob;
use eframe::egui::{
    self, Color32, CornerRadius, FontId, Key, Margin, Modifiers, RichText, Stroke, TextEdit,
    TextFormat,
};
use jiff::Zoned;
use jiff::tz::TimeZone;

use crate::output::{self, Item, Outputs};
use crate::parse::{self, Ctx};

const BG: Color32 = Color32::from_rgb(0x19, 0x1b, 0x22);
const FIELD: Color32 = Color32::from_rgb(0x22, 0x25, 0x2f);
const FG: Color32 = Color32::from_rgb(0xe4, 0xe7, 0xf0);
const DIM: Color32 = Color32::from_rgb(0x6c, 0x72, 0x84);
const ACCENT: Color32 = Color32::from_rgb(0x7d, 0x98, 0xec);
const CHIP: Color32 = Color32::from_rgb(0x24, 0x2a, 0x3d);
const MATCH: Color32 = Color32::from_rgb(0x30, 0x3c, 0x66);

const INPUT_SIZE: f32 = 18.0;
const PRIMARY_SIZE: f32 = 22.0;
const ALT_SIZE: f32 = 14.0;
const INFO_SIZE: f32 = 12.5;

const ALT_KEYS: [Key; 9] = [
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
];

pub struct App {
    input: String,
    /// Clipboard text that has something to convert; shown as the placeholder.
    clip: Option<String>,
    clip_rx: Option<Receiver<Option<String>>>,
    tz: TimeZone,
    out: Option<Outputs>,
    /// Bumped whenever the text to convert changes.
    version: u64,
    computed: Option<(u64, Instant)>,
    /// 0 is the primary value, k the kth alternative.
    selected: usize,
    /// What to copy once the window is gone.
    chosen: Rc<RefCell<Option<String>>>,
    focused_once: bool,
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext,
        input: String,
        clip_rx: Option<Receiver<Option<String>>>,
        chosen: Rc<RefCell<Option<String>>>,
    ) -> Self {
        style(&cc.egui_ctx);
        App {
            input,
            clip: None,
            clip_rx,
            tz: TimeZone::try_system().unwrap_or(TimeZone::UTC),
            out: None,
            version: 0,
            computed: None,
            selected: 0,
            chosen,
            focused_once: false,
        }
    }

    /// The text being converted: what was typed, or else the clipboard.
    fn source(&self) -> (&str, bool) {
        if self.input.trim().is_empty() {
            (self.clip.as_deref().unwrap_or(""), true)
        } else {
            (&self.input, false)
        }
    }

    fn refresh(&mut self) {
        let live = self.out.as_ref().is_some_and(|o| o.live);
        let stale = match self.computed {
            Some((version, at)) => {
                version != self.version || (live && at.elapsed() >= Duration::from_secs(1))
            }
            None => true,
        };
        if !stale {
            return;
        }
        let ctx = Ctx {
            now: Zoned::now().with_time_zone(self.tz.clone()),
            tz: self.tz.clone(),
        };
        let out = {
            let (text, _) = self.source();
            match parse::interpret(text, &ctx) {
                Some(it) => Some(output::build(&it, text, &ctx)),
                None if text.trim().is_empty() => Some(output::now(&ctx)),
                None => None,
            }
        };
        if self
            .computed
            .is_none_or(|(version, _)| version != self.version)
        {
            self.selected = 0;
        }
        self.out = out;
        self.computed = Some((self.version, Instant::now()));
    }

    fn items(&self) -> impl Iterator<Item = &Item> {
        self.out
            .iter()
            .flat_map(|o| std::iter::once(&o.primary).chain(&o.alternatives))
    }

    fn choose(&mut self, ctx: &egui::Context, k: usize) {
        if let Some(item) = self.items().nth(k) {
            *self.chosen.borrow_mut() = Some(item.text.clone());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn poll_clipboard(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.clip_rx else { return };
        match rx.try_recv() {
            Ok(clip) => {
                self.clip = clip;
                self.clip_rx = None;
                self.version += 1;
            }
            Err(TryRecvError::Empty) => ctx.request_repaint_after(Duration::from_millis(5)),
            Err(TryRecvError::Disconnected) => self.clip_rx = None,
        }
    }

    /// Popups go away when you look elsewhere.
    fn watch_focus(&mut self, ctx: &egui::Context) {
        match ctx.input(|i| i.viewport().focused) {
            Some(true) => self.focused_once = true,
            Some(false) if self.focused_once => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            _ => {}
        }
    }

    fn keys(&mut self, ctx: &egui::Context) {
        let count = self.items().count();
        let single_line = !self.input.contains('\n');
        let (enter, escape, next, prev, alt) = ctx.input_mut(|i| {
            let alt = ALT_KEYS
                .iter()
                .position(|&k| i.consume_key(Modifiers::ALT, k));
            let prev = i.consume_key(Modifiers::SHIFT, Key::Tab)
                || (single_line && i.consume_key(Modifiers::NONE, Key::ArrowUp));
            let next = i.consume_key(Modifiers::NONE, Key::Tab)
                || (single_line && i.consume_key(Modifiers::NONE, Key::ArrowDown));
            (
                i.consume_key(Modifiers::NONE, Key::Enter),
                i.consume_key(Modifiers::NONE, Key::Escape),
                next,
                prev,
                alt,
            )
        });
        if escape {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if count > 0 {
            if next {
                self.selected = (self.selected + 1) % count;
            }
            if prev {
                self.selected = (self.selected + count - 1) % count;
            }
        }
        if enter {
            self.choose(ctx, self.selected);
        }
        if let Some(n) = alt {
            self.choose(ctx, n + 1);
        }
    }

    fn input_field(&mut self, ui: &mut egui::Ui) {
        let (_, from_clip) = self.source();
        let span = self.out.as_ref().and_then(|o| o.span.clone());
        let hint: egui::WidgetText = match (&self.clip, span.clone().filter(|_| from_clip)) {
            (Some(clip), Some(span)) => placeholder(clip, span).into(),
            _ => RichText::new("date or epoch…")
                .color(DIM)
                .size(INPUT_SIZE)
                .into(),
        };
        let typed_span = span.filter(|_| !from_clip);
        let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
            let mut job = highlighted(buf.as_str(), typed_span.clone(), FG);
            job.wrap.max_width = wrap_width;
            ui.fonts_mut(|f| f.layout_job(job))
        };
        let edit = egui::ScrollArea::vertical()
            .max_height(INPUT_SIZE * 4.0)
            .show(ui, |ui| {
                TextEdit::multiline(&mut self.input)
                    .hint_text(hint)
                    .font(FontId::proportional(INPUT_SIZE))
                    .desired_rows(1)
                    .desired_width(f32::INFINITY)
                    .return_key(None)
                    .lock_focus(true)
                    .frame(
                        egui::Frame::new()
                            .fill(FIELD)
                            .inner_margin(Margin::same(8))
                            .corner_radius(CornerRadius::same(6)),
                    )
                    .layouter(&mut layouter)
                    .show(ui)
            })
            .inner;
        if !edit.response.has_focus() {
            edit.response.request_focus();
        }
        if edit.response.changed() {
            self.version += 1;
            self.refresh();
        }
    }

    fn results(&mut self, ui: &mut egui::Ui) {
        let Some(out) = &self.out else {
            ui.label(
                RichText::new("no date or epoch found")
                    .size(INFO_SIZE)
                    .color(DIM),
            );
            return;
        };
        ui.label(RichText::new(&out.info).size(INFO_SIZE).color(DIM));
        ui.add_space(6.0);
        let mut clicked = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
            for (k, item) in std::iter::once(&out.primary)
                .chain(&out.alternatives)
                .enumerate()
            {
                let (text, fill, hint) = if k == 0 {
                    (
                        RichText::new(&item.text)
                            .size(PRIMARY_SIZE)
                            .color(FG)
                            .strong(),
                        BG,
                        format!("{} · Enter", item.label),
                    )
                } else {
                    let key = if k <= 9 {
                        format!(" · Alt+{k}")
                    } else {
                        String::new()
                    };
                    (
                        RichText::new(&item.text).size(ALT_SIZE).color(ACCENT),
                        CHIP,
                        format!("{}{key}", item.label),
                    )
                };
                let stroke = if k == self.selected {
                    Stroke::new(1.5, ACCENT)
                } else {
                    Stroke::NONE
                };
                let button = egui::Button::new(text)
                    .fill(fill)
                    .stroke(stroke)
                    .corner_radius(CornerRadius::same(5));
                if ui.add(button).on_hover_text(hint).clicked() {
                    clicked = Some(k);
                }
            }
        });
        if let Some(k) = clicked {
            self.choose(ui.ctx(), k);
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_clipboard(&ctx);
        self.watch_focus(&ctx);
        self.keys(&ctx);
        self.refresh();
        // sway sizes floating windows itself, so everything fits a fixed size and scrolls past it.
        egui::Frame::new()
            .fill(BG)
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                self.input_field(ui);
                ui.add_space(6.0);
                // Until the clipboard arrives there is nothing worth showing.
                if self.clip_rx.is_none() || !self.input.trim().is_empty() {
                    egui::ScrollArea::vertical()
                        .id_salt("results")
                        .show(ui, |ui| self.results(ui));
                }
            });
        if self.out.as_ref().is_some_and(|o| o.live) {
            ctx.request_repaint_after(Duration::from_secs(1));
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        BG.to_normalized_gamma_f32()
    }
}

fn style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = BG;
    visuals.extreme_bg_color = FIELD;
    visuals.selection.bg_fill = MATCH;
    visuals.selection.stroke = Stroke::new(1.0, FG);
    visuals.text_cursor.stroke = Stroke::new(2.0, ACCENT);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    ctx.set_theme(egui::Theme::Dark);
    ctx.set_visuals_of(egui::Theme::Dark, visuals);
}

/// `text` with `span` highlighted, unless the span is all of it.
fn highlighted(text: &str, span: Option<Range<usize>>, color: Color32) -> LayoutJob {
    let plain = TextFormat {
        font_id: FontId::proportional(INPUT_SIZE),
        color,
        ..Default::default()
    };
    let mut job = LayoutJob::default();
    let span = span.filter(|s| {
        s.start < s.end
            && s.end <= text.len()
            && text.is_char_boundary(s.start)
            && text.is_char_boundary(s.end)
            && !(text[..s.start].trim().is_empty() && text[s.end..].trim().is_empty())
    });
    match span {
        Some(s) => {
            job.append(&text[..s.start], 0.0, plain.clone());
            job.append(
                &text[s.clone()],
                0.0,
                TextFormat {
                    background: MATCH,
                    color: FG,
                    ..plain.clone()
                },
            );
            job.append(&text[s.end..], 0.0, plain);
        }
        None => job.append(text, 0.0, plain),
    }
    job
}

/// The clipboard on one line, cut around the match: `…Date: Tue Oct 13 21:24:16 2026 +0200 fix the…`
fn placeholder(clip: &str, span: Range<usize>) -> LayoutJob {
    fn flat(s: &str) -> String {
        s.chars()
            .map(|c| if c.is_whitespace() { ' ' } else { c })
            .collect()
    }
    let before_all = &clip[..span.start];
    let after_all = &clip[span.end..];
    let before: String = before_all
        .chars()
        .rev()
        .take(24)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let after: String = after_all.chars().take(60).collect();
    let mut text = String::new();
    if before.len() < before_all.len() {
        text.push('…');
    }
    text.push_str(&flat(&before));
    let start = text.len();
    text.push_str(&flat(&clip[span.clone()]));
    let end = text.len();
    text.push_str(&flat(&after));
    if after.len() < after_all.len() {
        text.push('…');
    }
    highlighted(&text, Some(start..end), DIM)
}
