//! The interpretation corpus, with "now" fixed at Tuesday 2026-10-13 21:24:16 in Berlin.

use jiff::Timestamp;

use super::{Kind, Stage};
use crate::testing::{check_dates, date_of, run};

fn epoch_of(input: &str) -> Option<Timestamp> {
    match run(input)?.kind {
        Kind::Epoch(e) => Some(e.ts),
        Kind::Date(_) => None,
    }
}

const NOW: i64 = 1791919456;

#[test]
fn epochs() {
    let s = |n: i64| Timestamp::from_second(n).unwrap();
    let ms = |n: i64| Timestamp::from_millisecond(n).unwrap();
    assert_eq!(epoch_of("1791919456"), Some(s(NOW)));
    assert_eq!(epoch_of(" 1791919456\n"), Some(s(NOW)));
    assert_eq!(epoch_of("1791919456123"), Some(ms(1791919456123)));
    assert_eq!(epoch_of("1791919456.25"), Some(ms(1791919456250)));
    assert_eq!(
        epoch_of("1791919456123.5"),
        Some(Timestamp::from_microsecond(1791919456123500).unwrap())
    );
    assert_eq!(
        epoch_of("1791919456123456"),
        Some(Timestamp::from_microsecond(1791919456123456).unwrap())
    );
    assert_eq!(
        epoch_of("1791919456123456789"),
        Some(Timestamp::from_nanosecond(1791919456123456789).unwrap())
    );
    assert_eq!(epoch_of("@1791919456"), Some(s(NOW)));
    assert_eq!(epoch_of("\"created_at\": 1791919456,"), Some(s(NOW)));
    assert_eq!(epoch_of("10000000"), Some(s(10_000_000)));
    assert_eq!(epoch_of("1.791919456e9"), Some(s(NOW)));
    assert_eq!(epoch_of("1791919456123 ms"), Some(ms(1791919456123)));
}

#[test]
fn numbers_that_are_dates() {
    check_dates(&[
        ("14", "2026-10-14 00:00:00+02:00"),
        ("20261013", "2026-10-13 00:00:00+02:00"),
        ("202610132124", "2026-10-13 21:24:00+02:00"),
        ("20261013212416", "2026-10-13 21:24:16+02:00"),
    ]);
    let it = run("20261013").unwrap();
    assert!(
        it.as_epoch.is_some(),
        "the epoch reading is kept as an alternative"
    );
}

#[test]
fn epoch_in_text() {
    let it =
        run("{\"id\": 42, \"name\": \"something long\", \"created_at\": 1791919456789}").unwrap();
    assert_eq!(it.stage, Stage::EpochInText);
    assert!(matches!(it.kind, Kind::Epoch(e) if e.ts.as_millisecond() == 1791919456789));
}

#[test]
fn small_numbers_are_not_epochs() {
    // Below 10000000 (1970-04-26) a number is a count, a year or a day, not an instant.
    assert_eq!(date_of("0"), "nothing");
    assert_eq!(date_of("45"), "nothing");
    assert_eq!(date_of("9999999"), "nothing");
    assert_eq!(date_of("2026"), "2026-01-01 00:00:00+01:00");
}

#[test]
fn flags_are_not_relatives() {
    for input in [
        "-s",
        "+d",
        "-h",
        "ls -sat",
        "tar -xzf x.tgz",
        "git log --since",
        "-99999",
    ] {
        assert_eq!(date_of(input), "nothing", "{input:?}");
    }
    assert_eq!(date_of("-1s"), "2026-10-13 21:24:15+02:00");
    assert_eq!(date_of("+2mo"), "2026-12-13 00:00:00+01:00");
}

#[test]
fn org_mode() {
    check_dates(&[
        ("14", "2026-10-14 00:00:00+02:00"),
        ("12", "2026-11-12 00:00:00+01:00"),
        ("2/5", "2027-02-05 00:00:00+01:00"),
        ("Fri", "2026-10-16 00:00:00+02:00"),
        ("tue", "2026-10-13 00:00:00+02:00"),
        ("sep 15", "2027-09-15 00:00:00+02:00"),
        ("feb 15", "2027-02-15 00:00:00+01:00"),
        ("12:45", "2026-10-13 12:45:00+02:00"),
        ("22 sept 0:34", "2027-09-22 00:34:00+02:00"),
        ("w4", "2026-01-19 00:00:00+01:00"),
        ("2012 w4 fri", "2012-01-27 00:00:00+01:00"),
        ("2012-w04-5", "2012-01-27 00:00:00+01:00"),
        ("+0", "2026-10-13 00:00:00+02:00"),
        (".", "2026-10-13 00:00:00+02:00"),
        ("+4d", "2026-10-17 00:00:00+02:00"),
        ("+4", "2026-10-17 00:00:00+02:00"),
        ("+2w", "2026-10-27 00:00:00+01:00"),
        ("++5", "2026-10-18 00:00:00+02:00"),
        ("+2tue", "2026-10-27 00:00:00+01:00"),
        ("-wed", "2026-10-07 00:00:00+02:00"),
        ("+1m", "2026-11-13 00:00:00+01:00"),
        ("-1y", "2025-10-13 00:00:00+02:00"),
        ("+3h", "2026-10-14 00:24:16+02:00"),
        ("fri 14:00", "2026-10-16 14:00:00+02:00"),
        ("<2026-10-20 Tue 10:00 +1w>", "2026-10-20 10:00:00+02:00"),
        ("[2026-10-20 Di]", "2026-10-20 00:00:00+02:00"),
    ]);
}

#[test]
fn numeric_dates() {
    check_dates(&[
        ("2026-10-13", "2026-10-13 00:00:00+02:00"),
        ("2026/10/13", "2026-10-13 00:00:00+02:00"),
        ("2026.10.13", "2026-10-13 00:00:00+02:00"),
        ("13.10.2026", "2026-10-13 00:00:00+02:00"),
        ("13-10-2026", "2026-10-13 00:00:00+02:00"),
        ("10/13/2026", "2026-10-13 00:00:00+02:00"),
        ("13/10/2026", "2026-10-13 00:00:00+02:00"),
        ("03/04/2026", "2026-03-04 00:00:00+01:00"),
        ("03.04.2026", "2026-04-03 00:00:00+02:00"),
        ("13.10.26", "2026-10-13 00:00:00+02:00"),
        ("13.10.", "2026-10-13 00:00:00+02:00"),
        ("10/13", "2026-10-13 00:00:00+02:00"),
        ("2026-10", "2026-10-01 00:00:00+02:00"),
        ("2026-286", "2026-10-13 00:00:00+02:00"),
        ("2026-W42-2", "2026-10-13 00:00:00+02:00"),
        ("2026W42", "2026-10-12 00:00:00+02:00"),
        ("13.10.2026 21:24", "2026-10-13 21:24:00+02:00"),
        ("10/13/2026 09:24:16 PM", "2026-10-13 21:24:16+02:00"),
    ]);
}

#[test]
fn ambiguous_readings_are_offered() {
    let readings = |input: &str| match run(input).unwrap().kind {
        Kind::Date(r) => r
            .readings
            .iter()
            .map(|z| z.strftime("%Y-%m-%d").to_string())
            .collect::<Vec<_>>(),
        Kind::Epoch(_) => panic!("{input} read as an epoch"),
    };
    assert_eq!(readings("03/04/2026"), ["2026-04-03"]);
    assert_eq!(readings("03.04.2026"), ["2026-03-04"]);
    assert_eq!(readings("13/10/2026"), Vec::<String>::new());
    assert_eq!(readings("26-10-13"), ["2026-10-13"]);
    assert_eq!(readings("feb 15"), ["2026-02-15"]);
}

#[test]
fn iso_and_machine_formats() {
    check_dates(&[
        ("2026-10-13T21:24:16Z", "2026-10-13 21:24:16+00:00"),
        ("2026-10-13T21:24:16.123+02:00", "2026-10-13 21:24:16+02:00"),
        ("2026-10-13 21:24:16+0530", "2026-10-13 21:24:16+05:30"),
        ("2026-10-13T21:24+09:00", "2026-10-13 21:24:00+09:00"),
        (
            "2026-10-13T21:24:16+09:00[Asia/Tokyo]",
            "2026-10-13 21:24:16+09:00",
        ),
        ("20261013T212416Z", "2026-10-13 21:24:16+00:00"),
        ("2026-10-13 21:24:16 UTC", "2026-10-13 21:24:16+00:00"),
        (
            "2026-10-13 21:24:16 America/New_York",
            "2026-10-13 21:24:16-04:00",
        ),
        (
            "Tue, 13 Oct 2026 21:24:16 +0200",
            "2026-10-13 21:24:16+02:00",
        ),
        ("Tue, 13 Oct 2026 19:24:16 GMT", "2026-10-13 19:24:16+00:00"),
        ("Tue Oct 13 21:24:16 CEST 2026", "2026-10-13 21:24:16+02:00"),
        ("Tue Oct 13 21:24:16 2026", "2026-10-13 21:24:16+02:00"),
        ("13/Oct/2026:21:24:16 +0200", "2026-10-13 21:24:16+02:00"),
        (
            "Tue Oct 13 2026 21:24:16 GMT+0200 (Central European Summer Time)",
            "2026-10-13 21:24:16+02:00",
        ),
        (
            "2026-10-13 21:24:16.123456789 +0200 CEST m=+0.000000001",
            "2026-10-13 21:24:16+02:00",
        ),
        (
            "datetime.datetime(2026, 10, 13, 21, 24, 16)",
            "2026-10-13 21:24:16+02:00",
        ),
        ("9am PST", "2026-10-13 09:00:00-08:00"),
        ("21:24 UTC+9", "2026-10-13 21:24:00+09:00"),
    ]);
}

#[test]
fn dates_in_text() {
    check_dates(&[
        (
            "Oct 13 09:15:02 myhost sshd[1234]: Accepted publickey for chiya",
            "2026-10-13 09:15:02+02:00",
        ),
        (
            r#"127.0.0.1 - - [13/Oct/2026:21:24:16 +0200] "GET / HTTP/1.1" 200 612"#,
            "2026-10-13 21:24:16+02:00",
        ),
        (
            "commit 3f9a2c1\nAuthor: A <a@b.c>\nDate:   Tue Oct 13 21:24:16 2026 +0200\n\n    fix",
            "2026-10-13 21:24:16+02:00",
        ),
        (
            "Last-Modified: Tue, 13 Oct 2026 19:24:16 GMT\nETag: \"abc\"",
            "2026-10-13 19:24:16+00:00",
        ),
        (
            r#"{"id": 5, "created": "2026-10-13T19:24:16.000Z", "updated": "2026-10-14T00:00:00Z"}"#,
            "2026-10-13 19:24:16+00:00",
        ),
        (
            "2026-10-13 21:24:16,123 INFO worker started",
            "2026-10-13 21:24:16+02:00",
        ),
        (
            "Let's meet on 13.10.2026 at 14:00 in room 5",
            "2026-10-13 14:00:00+02:00",
        ),
        ("Can we do tomorrow at 3pm?", "2026-10-14 15:00:00+02:00"),
        ("Deadline is next friday, ok?", "2026-10-16 00:00:00+02:00"),
        ("I'll be back in 3 days.", "2026-10-16 21:24:16+02:00"),
        (
            "Released v1.2.10 on 2026-10-13, see notes",
            "2026-10-13 00:00:00+02:00",
        ),
        ("backup_2026-10-13_2124.tar.gz", "2026-10-13 00:00:00+02:00"),
        (
            "first 2026-10-13, then 2026-10-20",
            "2026-10-13 00:00:00+02:00",
        ),
        // A month and year, read as the first of the month; a full date later still wins.
        (
            "Monica Jewelers - Updated October 2026 - 34 Reviews",
            "2026-10-01 00:00:00+02:00",
        ),
        (
            "Published May 2026 by the team",
            "2026-05-01 00:00:00+02:00",
        ),
        (
            "Updated October 2026; the event is on 2026-11-05",
            "2026-11-05 00:00:00+01:00",
        ),
        ("会社概要 2026年10月 更新", "2026-10-01 00:00:00+02:00"),
        // A weekday next to a month and year doesn't move the date (see the note test).
        ("blarg november 2026 fri blarg", "2026-11-01 00:00:00+01:00"),
    ]);
}

#[test]
fn span_covers_the_date() {
    let input = "Date:   Tue Oct 13 21:24:16 2026 +0200\nmore";
    let it = run(input).unwrap();
    assert_eq!(&input[it.span], "Tue Oct 13 21:24:16 2026 +0200");
    let input = "deploy finished 2026-10-13T19:24:16Z ok";
    assert_eq!(&input[run(input).unwrap().span], "2026-10-13T19:24:16Z");
}

#[test]
fn not_dates() {
    for input in [
        "hello world",
        "upgraded to v1.2.10 and 10.10.10.10",
        "you may want to check",
        "it may 2026 be different",
        "id 550e8400-e29b-41d4-a716-446655440000",
        "call +49 30 1234567",
        "Released in 2026",
        "the ratio is 3/4",
        "",
        "   ",
    ] {
        assert_eq!(date_of(input), "nothing", "{input:?}");
    }
}

#[test]
fn weekday_mismatch_is_noted() {
    let Kind::Date(r) = run("Mon 2026-10-13").unwrap().kind else {
        panic!()
    };
    assert!(r.note.unwrap().contains("Tuesday"));
    // Without a day the weekday can't apply either; the note says so.
    let Kind::Date(r) = run("blarg november 2026 fri blarg").unwrap().kind else {
        panic!()
    };
    assert_eq!(
        r.note.as_deref(),
        Some("2026-11-01 is a Sunday, not a Friday")
    );
}

#[test]
fn dst_gap_and_fold() {
    // 2026-03-29 02:30 does not exist in Berlin; 2026-10-25 02:30 happens twice.
    check_dates(&[
        ("2026-03-29 02:30", "2026-03-29 03:30:00+02:00"),
        ("2026-10-25 02:30", "2026-10-25 02:30:00+02:00"),
    ]);
}

#[test]
fn language_of_the_words() {
    let lang = |input: &str| run(input).and_then(|i| i.lang);
    for (input, want) in [
        ("13. Oktober 2026 um 15 Uhr", Some("de")),
        ("vor 3 Tagen", Some("de")),
        ("2026年10月13日 21時24分16秒", Some("ja")),
        ("明天下午三点半", Some("zh")),
        ("2026年10月13日 星期二", Some("zh")),
        ("2026년 10월 13일", Some("ko")),
        ("завтра в 3 часа дня", Some("ru")),
        ("13 października 2026", Some("pl")),
        ("13 ตุลาคม 2569", Some("th")),
        ("13 octobre 2026", Some("fr")),
        ("next friday at 3pm", Some("en")),
        ("Meeting moved to 13. Oktober, see you", Some("de")),
        ("2026-10-13T21:24:16Z", None),
        ("1791919456", None),
        ("13.10.2026", None),
    ] {
        assert_eq!(lang(input), want, "{input:?}");
    }
}
