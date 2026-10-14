//! Time zone abbreviations, which mean the same in every language.

/// Fixed offsets (seconds east of UTC) for common time zone abbreviations.
/// Ambiguous ones use the most common reading (CST is US Central, IST is India).
pub fn tz_abbrev(w: &str) -> Option<i32> {
    const H: i32 = 3600;
    Some(match w {
        "utc" | "gmt" | "ut" | "zulu" | "wet" => 0,
        "west" | "bst" | "cet" | "mez" | "wat" => H,
        "cest" | "mesz" | "eet" | "sast" | "cat" => 2 * H,
        "eest" | "msk" | "eat" => 3 * H,
        "gst" => 4 * H,
        "pkt" => 5 * H,
        "ist" => 5 * H + 1800,
        "ict" | "wib" => 7 * H,
        "hkt" | "sgt" | "awst" | "pht" | "wita" => 8 * H,
        "jst" | "kst" | "wit" => 9 * H,
        "acst" => 9 * H + 1800,
        "aest" => 10 * H,
        "acdt" => 10 * H + 1800,
        "aedt" => 11 * H,
        "nzst" => 12 * H,
        "nzdt" => 13 * H,
        "hst" => -10 * H,
        "akst" => -9 * H,
        "akdt" | "pst" => -8 * H,
        "pdt" | "mst" => -7 * H,
        "mdt" | "cst" => -6 * H,
        "cdt" | "est" => -5 * H,
        "edt" | "ast" => -4 * H,
        "adt" | "brt" | "art" => -3 * H,
        "nst" => -3 * H - 1800,
        "ndt" => -2 * H - 1800,
        _ => return None,
    })
}

/// Abbreviations that name a region rather than a fixed offset.
pub fn tz_region(w: &str) -> Option<&'static str> {
    Some(match w {
        "pt" => "America/Los_Angeles",
        "mt" => "America/Denver",
        "ct" => "America/Chicago",
        "et" => "America/New_York",
        _ => return None,
    })
}
