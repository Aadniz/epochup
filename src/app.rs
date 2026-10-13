//! The popup: one input field, the conversion underneath.

use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use eframe::egui::text::LayoutJob;
use eframe::egui::{
    self, Color32, CornerRadius, FontId, Galley, Key, Margin, Modifiers, Rect, RichText, Sense,
    Stroke, StrokeKind, TextEdit, TextFormat, Vec2, vec2,
};
use jiff::tz::TimeZone;

use crate::output::{self, Item, Outputs};
use crate::parse::{self, Ctx};

// The palette of the sway, waybar and foot configs.
const BG: Color32 = hex(0x191b22);
const WELL: Color32 = hex(0x111111);
const BORDER: Color32 = hex(0x202936);
const TEXT: Color32 = hex(0xcccccc);
const BRIGHT: Color32 = hex(0xffffff);
const MUTED: Color32 = hex(0x6c757d);
const INFO: Color32 = hex(0x889ab1);
const ACCENT: Color32 = hex(0x7d98ec);
const LIGHT: Color32 = hex(0xc4e3ff);
const ON_ACCENT: Color32 = hex(0x041520);
const ERROR: Color32 = hex(0xe1746d);

const INPUT_SIZE: f32 = 18.0;
const PRIMARY_SIZE: f32 = 19.0;
const ALT_SIZE: f32 = 13.0;
const INFO_SIZE: f32 = 12.5;
const TAG_SIZE: f32 = 11.0;
const TAG_GAP: f32 = 5.0;

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

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    Typed,
    Clipboard,
    Now,
}

/// Where the arrow keys go: editing the input, or picking among the values.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Input,
    Values,
}

/// How a value is marked: the one Enter copies while typing, or the one picked.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    None,
    Default,
    Selected,
}

#[derive(Clone, Copy)]
enum Dir {
    Left,
    Right,
    Up,
    Down,
}

pub struct App {
    input: String,
    /// Clipboard text that has something to convert; shown as the placeholder.
    clip: Option<String>,
    clip_rx: Option<Receiver<Option<String>>>,
    /// Cleared by the first Escape, which swaps the clipboard placeholder for now.
    use_clip: bool,
    tz: TimeZone,
    out: Option<Outputs>,
    /// Bumped whenever the text to convert changes.
    version: u64,
    computed: Option<u64>,
    focus: Focus,
    /// 0 is the primary value, k the kth alternative. Always 0 while typing.
    selected: usize,
    /// Where each value was drawn last frame, for arrow-key navigation.
    rects: Vec<Rect>,
    /// The input's cursor last frame, in characters.
    cursor: Option<Range<usize>>,
    /// The last thing copied, by Enter, a click or Ctrl+C; served once the window is gone.
    copied: Rc<RefCell<Option<String>>>,
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext,
        input: String,
        clip_rx: Option<Receiver<Option<String>>>,
        copied: Rc<RefCell<Option<String>>>,
    ) -> Self {
        style(&cc.egui_ctx);
        App {
            input,
            clip: None,
            clip_rx,
            use_clip: true,
            tz: TimeZone::try_system().unwrap_or(TimeZone::UTC),
            out: None,
            version: 0,
            computed: None,
            focus: Focus::Input,
            selected: 0,
            rects: Vec::new(),
            cursor: None,
            copied,
        }
    }

    /// The text being converted: what was typed, or else the clipboard, or else now.
    fn source(&self) -> (&str, Source) {
        if !self.input.trim().is_empty() {
            (&self.input, Source::Typed)
        } else if let Some(clip) = self.clip.as_deref().filter(|_| self.use_clip) {
            (clip, Source::Clipboard)
        } else {
            ("", Source::Now)
        }
    }

    /// Recomputes on a new text, and every frame while the values move with the clock.
    fn refresh(&mut self) {
        let live = self.out.as_ref().is_some_and(|o| o.live);
        if self.computed == Some(self.version) && !live {
            return;
        }
        let ctx = Ctx::now_in(self.tz.clone());
        let out = {
            let (text, _) = self.source();
            match parse::interpret(text, &ctx) {
                Some(it) => Some(output::build(&it, text, &ctx)),
                None if text.trim().is_empty() => Some(output::now(&ctx)),
                None => None,
            }
        };
        if self.computed != Some(self.version) {
            self.focus_input();
        }
        self.out = out;
        self.computed = Some(self.version);
    }

    fn items(&self) -> impl Iterator<Item = &Item> {
        self.out
            .iter()
            .flat_map(|o| std::iter::once(&o.primary).chain(&o.alternatives))
    }

    fn choose(&mut self, ctx: &egui::Context, k: usize) {
        if let Some(item) = self.items().nth(k) {
            *self.copied.borrow_mut() = Some(item.text.clone());
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

    fn step(&mut self, dir: Dir) {
        if let Some(k) = neighbor(&self.rects, self.selected, dir) {
            self.selected = k;
        }
    }

    fn focus_input(&mut self) {
        self.focus = Focus::Input;
        self.selected = 0;
    }

    fn pick(&mut self, k: usize) {
        self.focus = Focus::Values;
        self.selected = k;
    }

    fn keys(&mut self, ctx: &egui::Context) {
        let chars = self.input.chars().count();
        let cursor = self.cursor.clone().unwrap_or(chars..chars);
        let collapsed = cursor.start == cursor.end;
        let count = self.items().count();
        // While picking, every arrow is ours. While typing, Right at the end of
        // the text and Down on its last line move down into the values.
        let picking = self.focus == Focus::Values && count > 0;
        let want_left = picking;
        let want_right = picking || (count > 0 && collapsed && cursor.end == chars);
        let want_up = picking;
        let want_down =
            picking || (count > 0 && !self.input.chars().skip(cursor.end).any(|c| c == '\n'));

        let (alt, prev, next, enter, escape, left, right, up, down) = ctx.input_mut(|i| {
            let plain = i.modifiers.is_none();
            let mut arrow = |want: bool, key| want && plain && i.consume_key(Modifiers::NONE, key);
            let arrows = (
                arrow(want_left, Key::ArrowLeft),
                arrow(want_right, Key::ArrowRight),
                arrow(want_up, Key::ArrowUp),
                arrow(want_down, Key::ArrowDown),
            );
            (
                ALT_KEYS
                    .iter()
                    .position(|&k| i.consume_key(Modifiers::ALT, k)),
                i.consume_key(Modifiers::SHIFT, Key::Tab),
                i.consume_key(Modifiers::NONE, Key::Tab),
                i.consume_key(Modifiers::NONE, Key::Enter),
                i.consume_key(Modifiers::NONE, Key::Escape),
                arrows.0,
                arrows.1,
                arrows.2,
                arrows.3,
            )
        });

        // Escape first dismisses the clipboard (showing now instead), then closes.
        if escape {
            if self.source().1 == Source::Clipboard {
                self.use_clip = false;
                self.version += 1;
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        if count > 0 {
            match (self.focus, next, prev) {
                (Focus::Input, true, _) => self.pick(1.min(count - 1)),
                (Focus::Input, _, true) => self.pick(count - 1),
                (Focus::Values, true, _) => self.selected = (self.selected + 1) % count,
                (Focus::Values, _, true) if self.selected == 0 => self.focus_input(),
                (Focus::Values, _, true) => self.selected -= 1,
                _ => {}
            }
        }
        match self.focus {
            Focus::Input if right => self.pick(1.min(count - 1)),
            // The primary value is already what Enter copies, so Down goes past it.
            Focus::Input if down => self.pick(neighbor(&self.rects, 0, Dir::Down).unwrap_or(0)),
            Focus::Input => {}
            Focus::Values => {
                // Back to the input: Left from the primary value, or Up from the top row.
                if left && self.selected == 0 {
                    self.focus_input();
                } else if left {
                    self.step(Dir::Left);
                }
                if right {
                    self.step(Dir::Right);
                }
                if down {
                    self.step(Dir::Down);
                }
                if up && neighbor(&self.rects, self.selected, Dir::Up).is_none() {
                    self.focus_input();
                } else if up {
                    self.step(Dir::Up);
                }
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
        let (_, source) = self.source();
        let span = self.out.as_ref().and_then(|o| o.span.clone());
        let hint: egui::WidgetText = match (source, &self.clip, &span) {
            (Source::Clipboard, Some(clip), Some(span)) => placeholder(clip, span.clone()).into(),
            _ if self.clip.is_some() => hint_text("now").into(),
            _ => hint_text("date or epoch").into(),
        };
        let typed_span = span.filter(|_| source == Source::Typed);
        let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
            let mut job = highlighted(buf.as_str(), typed_span.clone(), TEXT, LIGHT);
            job.wrap.max_width = wrap_width;
            ui.fonts_mut(|f| f.layout_job(job))
        };
        // Like a sway window: a bright border and a cursor while it has the focus.
        let typing = self.focus == Focus::Input;
        let edit = egui::ScrollArea::vertical()
            .max_height(INPUT_SIZE * 4.0)
            .show(ui, |ui| {
                if !typing {
                    ui.visuals_mut().text_cursor.stroke.color = Color32::TRANSPARENT;
                }
                TextEdit::multiline(&mut self.input)
                    .hint_text(hint)
                    .font(FontId::proportional(INPUT_SIZE))
                    .desired_rows(1)
                    .desired_width(f32::INFINITY)
                    .return_key(None)
                    .lock_focus(true)
                    .frame(
                        egui::Frame::new()
                            .fill(WELL)
                            .stroke(Stroke::new(1.0, if typing { ACCENT } else { BORDER }))
                            .inner_margin(Margin::symmetric(8, 6)),
                    )
                    .layouter(&mut layouter)
                    .show(ui)
            })
            .inner;
        self.cursor = edit.cursor_range.map(|r| {
            let r = r.as_sorted_char_range();
            r.start.0..r.end.0
        });
        if !edit.response.has_focus() {
            edit.response.request_focus();
        }
        if edit.response.clicked() {
            self.focus_input();
        }
        if edit.response.changed() {
            self.version += 1;
            self.refresh();
        }
    }

    fn results(&mut self, ui: &mut egui::Ui) {
        let Some(out) = &self.out else {
            self.rects.clear();
            ui.label(
                RichText::new("no date or epoch found")
                    .size(INFO_SIZE)
                    .color(ERROR),
            );
            return;
        };
        ui.label(RichText::new(&out.info).size(INFO_SIZE).color(INFO));
        ui.add_space(8.0);
        // Laid out by hand: a wrapped egui row would give every row the primary's height.
        let values: Vec<_> = std::iter::once(&out.primary)
            .chain(&out.alternatives)
            .collect();
        let laid: Vec<_> = values
            .iter()
            .enumerate()
            .map(|(k, item)| Value::layout(ui, item, k == 0))
            .collect();
        let sizes: Vec<_> = laid.iter().map(Value::size).collect();
        let width = ui.available_width();
        let (layout, height) = flow(&sizes, width, 6.0);
        let (area, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let mut clicked = None;
        let mut rects = Vec::new();
        for (k, (value, rect)) in laid.into_iter().zip(layout).enumerate() {
            let rect = rect.translate(area.min.to_vec2());
            let response = ui
                .interact(rect, ui.id().with(("value", k)), Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            let mark = match (k == self.selected, self.focus) {
                (false, _) => Mark::None,
                (true, Focus::Input) => Mark::Default,
                (true, Focus::Values) => Mark::Selected,
            };
            value.paint(ui, &response, mark);
            let hint = match k {
                1..=9 => format!("{} · Alt+{k}", values[k].label),
                _ => values[k].label.clone(),
            };
            rects.push(rect);
            if response.on_hover_text(hint).clicked() {
                clicked = Some(k);
            }
        }
        self.rects = rects;
        if let Some(k) = clicked {
            self.choose(ui.ctx(), k);
        }
    }

    /// Text copied out of the input with Ctrl+C or Ctrl+X should outlive the window too.
    fn remember_copies(&self, ctx: &egui::Context) {
        let copied = ctx.output(|o| {
            o.commands.iter().rev().find_map(|c| match c {
                egui::OutputCommand::CopyText(text) if !text.is_empty() => Some(text.clone()),
                _ => None,
            })
        });
        if let Some(text) = copied {
            *self.copied.borrow_mut() = Some(text);
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_clipboard(&ctx);
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
        self.remember_copies(&ctx);
        if self.out.as_ref().is_some_and(|o| o.live) {
            ctx.request_repaint();
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
    visuals.extreme_bg_color = WELL;
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(0x7d, 0x98, 0xec, 0x60);
    visuals.selection.stroke = Stroke::new(1.0, BRIGHT);
    visuals.text_cursor.stroke = Stroke::new(2.0, LIGHT);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.window_corner_radius = CornerRadius::ZERO;
    visuals.menu_corner_radius = CornerRadius::ZERO;
    ctx.set_theme(egui::Theme::Dark);
    ctx.set_visuals_of(egui::Theme::Dark, visuals);
}

/// The value to select when moving from `from` in a direction, by where the
/// values are drawn. Left and Right stay in the row. Up and Down go to the
/// nearest row: from the first value of a row to the first of the next, and
/// otherwise to the value most directly above or below.
fn neighbor(rects: &[Rect], from: usize, dir: Dir) -> Option<usize> {
    let r = rects.get(from)?;
    let x = r.center().x;
    let others = || rects.iter().enumerate().filter(move |&(k, _)| k != from);
    let same_row = |o: &Rect| o.top() < r.bottom() && o.bottom() > r.top();
    let closest = |a: &(usize, &Rect), b: &(usize, &Rect)| {
        (a.1.center().x - x)
            .abs()
            .total_cmp(&(b.1.center().x - x).abs())
    };
    let row: Vec<_> = others().filter(|(_, o)| same_row(o)).collect();
    match dir {
        Dir::Left => row
            .into_iter()
            .filter(|(_, o)| o.center().x < x)
            .min_by(closest),
        Dir::Right => row
            .into_iter()
            .filter(|(_, o)| o.center().x > x)
            .min_by(closest),
        Dir::Up | Dir::Down => {
            let gap = |o: &Rect| match dir {
                Dir::Up => r.top() - o.bottom(),
                _ => o.top() - r.bottom(),
            };
            let nearest = others()
                .map(|(_, o)| gap(o))
                .filter(|g| *g >= -1.0)
                .fold(f32::INFINITY, f32::min);
            let target: Vec<_> = others()
                .filter(|(_, o)| (-1.0..=nearest + 2.0).contains(&gap(o)))
                .collect();
            if row.iter().all(|(_, o)| o.center().x > x) {
                target
                    .into_iter()
                    .min_by(|a, b| a.1.left().total_cmp(&b.1.left()))
            } else {
                target.into_iter().min_by(closest)
            }
        }
    }
    .map(|(k, _)| k)
}

fn pad(primary: bool) -> egui::Vec2 {
    if primary {
        vec2(8.0, 4.0)
    } else {
        vec2(6.0, 2.0)
    }
}

/// Places boxes left to right, wrapping at `width`; each row is as tall as its
/// tallest box, and the others are centered in it. Returns the rects and the total height.
fn flow(sizes: &[egui::Vec2], width: f32, gap: f32) -> (Vec<Rect>, f32) {
    let mut rects: Vec<Rect> = Vec::with_capacity(sizes.len());
    let (mut x, mut y, mut row_height, mut row_start) = (0.0, 0.0, 0.0f32, 0);
    let center = |rects: &mut [Rect], row_height: f32| {
        for r in rects {
            *r = r.translate(vec2(0.0, (row_height - r.height()) / 2.0));
        }
    };
    for (k, &size) in sizes.iter().enumerate() {
        if x > 0.0 && x + size.x > width {
            center(&mut rects[row_start..], row_height);
            y += row_height + gap;
            (x, row_height, row_start) = (0.0, 0.0, k);
        }
        rects.push(Rect::from_min_size(egui::pos2(x, y), size));
        x += size.x + gap;
        row_height = row_height.max(size.y);
    }
    center(&mut rects[row_start..], row_height);
    (rects, y + row_height)
}

/// A copyable value and its tag, laid out. Drawn by hand so that hovering or
/// selecting never changes its size.
struct Value {
    primary: bool,
    text: Arc<Galley>,
    tag: Option<Arc<Galley>>,
}

impl Value {
    fn layout(ui: &egui::Ui, item: &Item, primary: bool) -> Self {
        let size = if primary { PRIMARY_SIZE } else { ALT_SIZE };
        let painter = ui.painter();
        let text = painter.layout_no_wrap(
            item.text.clone(),
            FontId::monospace(size),
            Color32::PLACEHOLDER,
        );
        let tag = item.tag.as_ref().map(|tag| {
            painter.layout_no_wrap(
                tag.clone(),
                FontId::proportional(TAG_SIZE),
                Color32::PLACEHOLDER,
            )
        });
        Value { primary, text, tag }
    }

    fn size(&self) -> Vec2 {
        let tag = self.tag.as_ref().map_or(0.0, |t| t.size().x + TAG_GAP);
        vec2(tag + self.text.size().x, self.text.size().y) + 2.0 * pad(self.primary)
    }

    fn paint(self, ui: &egui::Ui, response: &egui::Response, mark: Mark) {
        let (fg, bg, tag_color) = match (mark == Mark::Selected, self.primary) {
            (true, _) => (ON_ACCENT, ACCENT, BORDER),
            (false, true) => (BRIGHT, BG, INFO),
            (false, false) => (LIGHT, BORDER, INFO),
        };
        let rect = response.rect;
        if !ui.is_rect_visible(rect) {
            return;
        }
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::ZERO, bg);
        // The value Enter copies while typing gets the accent outline; hovering a dimmer one.
        let outline = match mark {
            Mark::Default => Some(ACCENT),
            Mark::None if response.hovered() => Some(INFO),
            _ => None,
        };
        if let Some(color) = outline {
            painter.rect_stroke(
                rect,
                CornerRadius::ZERO,
                Stroke::new(1.0, color),
                StrokeKind::Inside,
            );
        }
        let inner = rect.shrink2(pad(self.primary));
        let mut x = inner.min.x;
        if let Some(tag) = self.tag {
            let y = inner.center().y - tag.size().y / 2.0;
            x += tag.size().x + TAG_GAP;
            painter.galley(egui::pos2(inner.min.x, y), tag, tag_color);
        }
        painter.galley(egui::pos2(x, inner.min.y), self.text, fg);
    }
}

fn hint_text(text: &str) -> RichText {
    RichText::new(text).color(MUTED).size(INPUT_SIZE)
}

/// `text` with `span` highlighted, unless the span is all of it.
fn highlighted(
    text: &str,
    span: Option<Range<usize>>,
    color: Color32,
    matched: Color32,
) -> LayoutJob {
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
                    background: BORDER,
                    color: matched,
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
    highlighted(&text, Some(start..end), MUTED, INFO)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, w: f32) -> Rect {
        Rect::from_min_size(egui::pos2(x, y), vec2(w, 20.0))
    }

    #[test]
    fn left_edge_sticks() {
        // [ long first ] [ b ]
        // [c] [ very long d          ] [e]
        // [f]
        let rects = [
            rect(0.0, 0.0, 300.0),
            rect(306.0, 0.0, 60.0),
            rect(0.0, 26.0, 40.0),
            rect(46.0, 26.0, 400.0),
            rect(452.0, 26.0, 50.0),
            rect(0.0, 52.0, 40.0),
        ];
        assert_eq!(
            neighbor(&rects, 0, Dir::Down),
            Some(2),
            "first of a row goes to the first below"
        );
        assert_eq!(
            neighbor(&rects, 1, Dir::Down),
            Some(3),
            "the right edge goes to the closest below"
        );
        assert_eq!(neighbor(&rects, 4, Dir::Up), Some(1));
        assert_eq!(neighbor(&rects, 2, Dir::Up), Some(0));
        assert_eq!(
            neighbor(&rects, 3, Dir::Down),
            Some(5),
            "only one value below"
        );
        assert_eq!(neighbor(&rects, 5, Dir::Up), Some(2));
    }

    #[test]
    fn middle_goes_to_the_closest() {
        // [a] [ b ] [c]
        // [d    ] [ e ] [f]
        let rects = [
            rect(0.0, 0.0, 50.0),
            rect(56.0, 0.0, 80.0),
            rect(142.0, 0.0, 50.0),
            rect(0.0, 26.0, 90.0),
            rect(96.0, 26.0, 60.0),
            rect(162.0, 26.0, 50.0),
        ];
        assert_eq!(neighbor(&rects, 1, Dir::Down), Some(4));
        assert_eq!(neighbor(&rects, 4, Dir::Up), Some(1));
        assert_eq!(neighbor(&rects, 1, Dir::Right), Some(2));
        assert_eq!(neighbor(&rects, 2, Dir::Right), None);
        assert_eq!(neighbor(&rects, 0, Dir::Up), None);
    }
}
