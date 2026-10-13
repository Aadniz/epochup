//! Clipboard access. On Wayland (and X11) the copying process has to keep
//! serving the contents, so when the popup closes a detached copy of this
//! program takes over until something else is copied.

use std::io::Write;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use arboard::{GetExtLinux, LinuxClipboardKind, SetExtLinux};

use crate::parse::{self, Ctx};

/// The flag that makes this program serve its stdin as the clipboard.
pub const SERVE_FLAG: &str = "--serve-clipboard";

/// Huge pastes are cut, at a character boundary.
const MAX_LEN: usize = 1 << 20;

fn truncate(mut s: String) -> String {
    if s.len() > MAX_LEN {
        let mut end = MAX_LEN;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        s.truncate(end);
    }
    s
}

/// The clipboard text, or the primary selection when only that holds
/// something to convert.
pub fn startup_text() -> Option<String> {
    let ctx = Ctx::system();
    let readable = |t: &String| parse::interpret(t, &ctx).is_some();
    let mut cb = arboard::Clipboard::new().ok()?;
    let clip = cb.get_text().ok().map(truncate);
    if clip.as_ref().is_some_and(readable) {
        return clip;
    }
    cb.get()
        .clipboard(LinuxClipboardKind::Primary)
        .text()
        .ok()
        .map(truncate)
        .filter(readable)
}

/// Copies `text` through a detached child that owns the clipboard until it is replaced.
pub fn hand_off(text: &str) -> std::io::Result<()> {
    let mut child = Command::new(std::env::current_exe()?)
        .arg(SERVE_FLAG)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()?;
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(text.as_bytes())
}

/// Runs in the detached child: sets the clipboard and serves it until something else is copied.
pub fn serve(text: String) {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set().wait().text(text);
    }
}
