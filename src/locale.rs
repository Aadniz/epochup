//! Formatting through the C library, so dates come out exactly as `LC_TIME` renders them.

use std::ffi::{CStr, CString};
use std::sync::OnceLock;

use jiff::Zoned;

/// glibc's `_DATE_FMT`: the format `date` prints with no arguments. Not exported by the libc crate.
const DATE_FMT: libc::nl_item = 0x2006C;
const C_DATE_FMT: &str = "%a %b %e %H:%M:%S %Z %Y";

static FORMATS: OnceLock<(String, String)> = OnceLock::new();

/// Applies the user's `LC_TIME`. Must run before other threads start.
pub fn init() {
    FORMATS.get_or_init(|| {
        let langinfo = |item| {
            // SAFETY: nl_langinfo returns a pointer to a NUL-terminated static string, copied at once.
            let ptr = unsafe { libc::nl_langinfo(item) };
            (!ptr.is_null()).then(|| {
                unsafe { CStr::from_ptr(ptr) }
                    .to_string_lossy()
                    .into_owned()
            })
        };
        // SAFETY: called once at startup, before any threads read the locale.
        unsafe { libc::setlocale(libc::LC_TIME, c"".as_ptr()) };
        let date_fmt = langinfo(DATE_FMT)
            .filter(|f| !f.is_empty())
            .unwrap_or_else(|| C_DATE_FMT.into());
        let name = unsafe { libc::setlocale(libc::LC_TIME, std::ptr::null()) };
        let name = if name.is_null() {
            "C".into()
        } else {
            unsafe { CStr::from_ptr(name) }
                .to_string_lossy()
                .into_owned()
        };
        (date_fmt, name)
    });
}

/// The locale's `date(1)` format, e.g. `%Y年 %b %e日 %A %H:%M:%S %Z` for ja_JP.
pub fn date_fmt() -> &'static str {
    init();
    &FORMATS.get().expect("initialized").0
}

/// The `LC_TIME` locale name, e.g. `ja_JP.UTF-8`.
pub fn name() -> &'static str {
    init();
    &FORMATS.get().expect("initialized").1
}

/// `strftime(3)` in the current `LC_TIME`, with the zone abbreviation and offset of `z`.
pub fn strftime(z: &Zoned, fmt: &str) -> String {
    init();
    let info = z.time_zone().to_offset_info(z.timestamp());
    let abbr = CString::new(info.abbreviation()).unwrap_or_default();
    let Ok(cfmt) = CString::new(fmt) else {
        return String::new();
    };
    // SAFETY: tm is plain old data; every field strftime reads is set below.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    tm.tm_year = z.year() as i32 - 1900;
    tm.tm_mon = z.month() as i32 - 1;
    tm.tm_mday = z.day() as i32;
    tm.tm_hour = z.hour() as i32;
    tm.tm_min = z.minute() as i32;
    tm.tm_sec = z.second() as i32;
    tm.tm_wday = z.weekday().to_sunday_zero_offset() as i32;
    tm.tm_yday = z.day_of_year() as i32 - 1;
    tm.tm_isdst = info.dst().is_dst() as i32;
    tm.tm_gmtoff = info.offset().seconds() as libc::c_long;
    tm.tm_zone = abbr.as_ptr();
    let mut buf = vec![0u8; 256];
    loop {
        // SAFETY: buf is writable for buf.len() bytes; cfmt and tm (and abbr, which tm points to) outlive the call.
        let n = unsafe { libc::strftime(buf.as_mut_ptr().cast(), buf.len(), cfmt.as_ptr(), &tm) };
        if n > 0 || buf.len() >= 1 << 16 {
            buf.truncate(n);
            return String::from_utf8_lossy(&buf).into_owned();
        }
        buf.resize(buf.len() * 4, 0);
    }
}

/// The date the way `date` prints it in this locale.
pub fn date(z: &Zoned) -> String {
    strftime(z, date_fmt())
}
