//! How long interpreting takes: typical short inputs, and the worst case of a
//! huge pasted blob with the only date at its very end.
//!
//! Run with `cargo bench`; numbers are per call, best of several rounds.

use std::hint::black_box;
use std::time::{Duration, Instant};

use epochup::parse::{Ctx, interpret};

const SHORT: &[&str] = &[
    "1791919456",
    "now",
    "next friday 3pm",
    "2026-10-13T21:24:16Z",
    "Tue, 13 Oct 2026 21:24:16 +0200",
    "13. Oktober 2026 um 15 Uhr",
    "明天下午三点半",
    "2026年10月13日 21時24分16秒",
    "03/04/2026",
    "in 3 days",
];

/// Deterministic filler text: a cheap linear congruential generator over a word list.
fn prose(words: &[&str], count: usize, sep: &str) -> String {
    let mut state = 12345u64;
    let mut out = String::new();
    for k in 0..count {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        out.push_str(words[(state >> 33) as usize % words.len()]);
        out.push_str(sep);
        if k % 13 == 0 {
            out.push_str(&((state >> 40) % 100).to_string());
            out.push_str(sep);
        }
    }
    out
}

fn time(name: &str, rounds: u32, iterations: u32, mut f: impl FnMut()) {
    let mut best = Duration::MAX;
    for _ in 0..rounds {
        let start = Instant::now();
        for _ in 0..iterations {
            f();
        }
        best = best.min(start.elapsed() / iterations);
    }
    println!("{name:<40} {best:>12.2?}");
}

fn main() {
    let ctx = Ctx::system();
    for input in SHORT {
        time(input, 5, 2000, || {
            black_box(interpret(black_box(input), &ctx));
        });
    }

    let english = "the of and to in is was for on that with as by at from this be are have it an or log error request user id value done may march sun";
    let mut blob = prose(&english.split(' ').collect::<Vec<_>>(), 100_000, " ");
    blob.push_str("finally deployed on Tue, 13 Oct 2026 21:24:16 +0200");
    time(
        &format!("english blob, {} KB", blob.len() / 1000),
        5,
        1,
        || {
            black_box(interpret(black_box(&blob), &ctx));
        },
    );

    let chinese: Vec<String> = "的一是在不了有和人这中大为上个国我以要他时来用们生到作地于出就分对成会可主发年动同工也能下过子说产种面而方后多定行学法所民得经十三之进着等部度家电力里如水化高自二理起小物现实加量都两体制机当使点从业本去把性好应开它合还因由其些然前外天政四日那社义事平形相全表间样与关各重新线内数正心反你明看原又么利比或但质气第向道命此变条只没结解问意建月公无系军很情者最立代想已通并提直题党程展五果料象员革位入常文总次品式活设及管特件长求老头基资边流路级少图山统接知较将组见计别她手角期根论运农指几九区强放决西被干做必战先回则任取据处理"
        .chars()
        .map(String::from)
        .collect();
    let refs: Vec<&str> = chinese.iter().map(String::as_str).collect();
    let mut blob = prose(&refs, 300_000, "");
    blob.push_str("会议定于2026年10月13日下午三点");
    time(
        &format!("chinese blob, {} KB", blob.len() / 1000),
        5,
        1,
        || {
            black_box(interpret(black_box(&blob), &ctx));
        },
    );
}
