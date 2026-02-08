use chrono::TimeZone;
use chrono_tz::Europe::Berlin;
use chrono_tz::US::Eastern;
use clockwords::{
    ExpressionKind, ParserConfig, ResolvedTime, TimeExpressionScanner, scanner_for_languages,
};

/// Helper: create a scanner with a specific timezone.
fn scanner_with_tz(tz: chrono_tz::Tz) -> TimeExpressionScanner {
    let languages: Vec<Box<dyn clockwords::lang::LanguageParser>> = vec![
        Box::new(clockwords::lang::en::English::new()),
        Box::new(clockwords::lang::de::German::new()),
    ];
    let config = ParserConfig {
        timezone: tz,
        ..Default::default()
    };
    TimeExpressionScanner::new(languages, config)
}

// ============================================================
//  "today" near midnight — timezone changes which day it is
// ============================================================

#[test]
fn today_in_berlin_when_utc_is_previous_day() {
    let s = scanner_with_tz(Berlin);
    // At 23:30 UTC on Feb 6, it is 00:30 CET on Feb 7 in Berlin
    let now = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 23, 30, 0).unwrap();
    let m = s.scan("today", now);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::RelativeDay);

    // "Today" in Berlin is Feb 7 CET.
    // Midnight Feb 7 CET = Feb 6 23:00 UTC
    // Midnight Feb 8 CET = Feb 7 23:00 UTC
    let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 23, 0, 0).unwrap();
    let expected_end = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 23, 0, 0).unwrap();
    assert_eq!(
        m[0].resolved,
        ResolvedTime::Range {
            start: expected_start,
            end: expected_end,
        }
    );
}

#[test]
fn today_in_utc_default_unchanged() {
    // Default scanner (UTC timezone) should behave exactly like before
    let s = scanner_for_languages(&["en"]);
    let now = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 14, 30, 0).unwrap();
    let m = s.scan("today", now);
    assert_eq!(m.len(), 1);
    let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap();
    let expected_end = chrono::Utc.with_ymd_and_hms(2026, 2, 8, 0, 0, 0).unwrap();
    assert_eq!(
        m[0].resolved,
        ResolvedTime::Range {
            start: expected_start,
            end: expected_end,
        }
    );
}

// ============================================================
//  "at 3pm" — interpreted in user's timezone
// ============================================================

#[test]
fn at_3pm_in_berlin() {
    let s = scanner_with_tz(Berlin);
    // Feb 7 2026 is winter in Berlin → CET = UTC+1
    let now = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 10, 0, 0).unwrap();
    let m = s.scan("at 3pm", now);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeSpecification);
    // 3pm Berlin CET = 14:00 UTC
    let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 14, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn at_3pm_in_utc() {
    let s = scanner_for_languages(&["en"]);
    let now = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 10, 0, 0).unwrap();
    let m = s.scan("at 3pm", now);
    assert_eq!(m.len(), 1);
    // 3pm UTC = 15:00 UTC
    let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 15, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

// ============================================================
//  "yesterday at 3pm" — combined, timezone-aware
// ============================================================

#[test]
fn yesterday_at_3pm_in_berlin() {
    let s = scanner_with_tz(Berlin);
    // At 00:30 CET Feb 7 (= 23:30 UTC Feb 6)
    let now = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 23, 30, 0).unwrap();
    let m = s.scan("yesterday at 3pm", now);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);

    // "Today" in Berlin is Feb 7, so "yesterday" is Feb 6.
    // 3pm on Feb 6 Berlin CET = 14:00 UTC on Feb 6
    let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 14, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

// ============================================================
//  "between 9 and 12" — time range in user's timezone
// ============================================================

#[test]
fn between_9_and_12_in_berlin() {
    let s = scanner_with_tz(Berlin);
    let now = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 10, 0, 0).unwrap();
    let m = s.scan("between 9 and 12", now);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);

    // 9:00 Berlin CET = 8:00 UTC, 12:00 Berlin CET = 11:00 UTC
    let start = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 8, 0, 0).unwrap();
    let end = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 11, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

// ============================================================
//  scan_with_tz — explicit timezone override
// ============================================================

#[test]
fn scan_with_tz_overrides_config() {
    // Scanner configured with UTC
    let s = scanner_for_languages(&["en"]);
    let now = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 10, 0, 0).unwrap();

    // Use scan_with_tz to override to Berlin
    let m = s.scan_with_tz("at 3pm", now, Berlin);
    assert_eq!(m.len(), 1);
    // 3pm Berlin CET = 14:00 UTC
    let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 14, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

// ============================================================
//  Negative UTC offset timezone (US Eastern)
// ============================================================

#[test]
fn today_in_eastern_when_utc_is_next_day() {
    let s = scanner_with_tz(Eastern);
    // At 03:00 UTC on Feb 8, it is 22:00 EST on Feb 7 in Eastern time
    let now = chrono::Utc.with_ymd_and_hms(2026, 2, 8, 3, 0, 0).unwrap();
    let m = s.scan("today", now);
    assert_eq!(m.len(), 1);

    // "Today" in Eastern is still Feb 7.
    // Midnight Feb 7 EST = Feb 7 05:00 UTC
    // Midnight Feb 8 EST = Feb 8 05:00 UTC
    let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 5, 0, 0).unwrap();
    let expected_end = chrono::Utc.with_ymd_and_hms(2026, 2, 8, 5, 0, 0).unwrap();
    assert_eq!(
        m[0].resolved,
        ResolvedTime::Range {
            start: expected_start,
            end: expected_end,
        }
    );
}

#[test]
fn at_3pm_in_eastern() {
    let s = scanner_with_tz(Eastern);
    // Feb 7 2026 is winter in Eastern → EST = UTC-5
    let now = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 15, 0, 0).unwrap();
    let m = s.scan("at 3pm", now);
    assert_eq!(m.len(), 1);
    // 3pm EST = 20:00 UTC
    let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 20, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

// ============================================================
//  German: "gestern um 15 Uhr" with timezone
// ============================================================

#[test]
fn de_gestern_um_15_uhr_in_berlin() {
    let s = scanner_with_tz(Berlin);
    let now = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 10, 0, 0).unwrap();
    let m = s.scan("gestern um 15 Uhr", now);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    // Yesterday = Feb 6 in Berlin, 15:00 CET = 14:00 UTC
    let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 14, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

// ============================================================
//  Weekday with timezone
// ============================================================

#[test]
fn next_monday_in_berlin_near_midnight() {
    let s = scanner_with_tz(Berlin);
    // Sunday Feb 8 2026 at 23:30 UTC = Monday Feb 9 00:30 CET in Berlin
    let now = chrono::Utc.with_ymd_and_hms(2026, 2, 8, 23, 30, 0).unwrap();
    let m = s.scan("next monday", now);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::RelativeDay);

    // In Berlin it's already Monday Feb 9 CET, so "next monday" = Feb 16
    // Midnight Feb 16 CET = Feb 15 23:00 UTC
    // Midnight Feb 17 CET = Feb 16 23:00 UTC
    let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 15, 23, 0, 0).unwrap();
    let expected_end = chrono::Utc.with_ymd_and_hms(2026, 2, 16, 23, 0, 0).unwrap();
    assert_eq!(
        m[0].resolved,
        ResolvedTime::Range {
            start: expected_start,
            end: expected_end,
        }
    );
}

// ============================================================
//  "the last hour" — duration-based, timezone-independent
// ============================================================

#[test]
fn last_hour_same_regardless_of_timezone() {
    let now = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 14, 30, 0).unwrap();

    let s_utc = scanner_for_languages(&["en"]);
    let m_utc = s_utc.scan("the last hour", now);

    let s_berlin = scanner_with_tz(Berlin);
    let m_berlin = s_berlin.scan("the last hour", now);

    // Duration-based expressions should produce identical results
    assert_eq!(m_utc[0].resolved, m_berlin[0].resolved);
    assert_eq!(
        m_utc[0].resolved,
        ResolvedTime::Range {
            start: now - chrono::Duration::hours(1),
            end: now,
        }
    );
}
