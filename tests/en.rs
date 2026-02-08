use clockwords::{scanner_for_languages, ExpressionKind, ResolvedTime, MatchConfidence, Span};
use chrono::TimeZone;

fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 14, 30, 0).unwrap()
}

#[test]
fn en_today() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("today", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].span, Span::new(0, 5));
    assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
    assert_eq!(m[0].confidence, MatchConfidence::Complete);
}

#[test]
fn en_yesterday() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("yesterday", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].span, Span::new(0, 9));
    assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
    let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 0, 0, 0).unwrap();
    let expected_end = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap();
    assert_eq!(
        m[0].resolved,
        ResolvedTime::Range {
            start: expected_start,
            end: expected_end,
        }
    );
}

#[test]
fn en_tomorrow() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("tomorrow", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].span, Span::new(0, 8));
    let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 8, 0, 0, 0).unwrap();
    assert_eq!(
        m[0].resolved,
        ResolvedTime::Range {
            start: expected_start,
            end: expected_start + chrono::Duration::days(1),
        }
    );
}

#[test]
fn en_in_4_days() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("in 4 days", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].span, Span::new(0, 9));
    assert_eq!(m[0].kind, ExpressionKind::RelativeDayOffset);
    let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 11, 0, 0, 0).unwrap();
    assert_eq!(
        m[0].resolved,
        ResolvedTime::Range {
            start: expected_start,
            end: expected_start + chrono::Duration::days(1),
        }
    );
}

#[test]
fn en_two_days_ago() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("two days ago", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].span, Span::new(0, 12));
    assert_eq!(m[0].kind, ExpressionKind::RelativeDayOffset);
    let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 5, 0, 0, 0).unwrap();
    assert_eq!(
        m[0].resolved,
        ResolvedTime::Range {
            start: expected_start,
            end: expected_start + chrono::Duration::days(1),
        }
    );
}

#[test]
fn en_at_3pm() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("at 3pm", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].span, Span::new(0, 6));
    assert_eq!(m[0].kind, ExpressionKind::TimeSpecification);
    let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 15, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn en_13_oclock() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("13 o'clock", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeSpecification);
    let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 13, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn en_the_last_hour() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("the last hour", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].span, Span::new(0, 13));
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);
    let n = now();
    assert_eq!(
        m[0].resolved,
        ResolvedTime::Range {
            start: n - chrono::Duration::hours(1),
            end: n,
        }
    );
}

#[test]
fn en_last_hour_without_the() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("last hour", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);
}

#[test]
fn en_between_9_and_12() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("between 9 and 12", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);
    let start = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 9, 0, 0).unwrap();
    let end = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 12, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn en_between_9_and_12_oclock() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("between 9 and 12 o'clock", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);
}

#[test]
fn en_yesterday_at_3pm() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("yesterday at 3pm", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].span, Span::new(0, 16));
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 15, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn en_tomorrow_between_9_and_12() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("tomorrow between 9 and 12", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let start = chrono::Utc.with_ymd_and_hms(2026, 2, 8, 9, 0, 0).unwrap();
    let end = chrono::Utc.with_ymd_and_hms(2026, 2, 8, 12, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn en_embedded_in_sentence() {
    let s = scanner_for_languages(&["en"]);
    let text = "The last hour I coded the initial code for the time library";
    let m = s.scan(text, now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);
    assert!(m[0].span.end <= 14, "span end should be at most 14, got {}", m[0].span.end);
}

#[test]
fn en_case_insensitive() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("YESTERDAY", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
}

#[test]
fn en_no_match() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("I wrote some code", now());
    assert_eq!(m.len(), 0);
}
