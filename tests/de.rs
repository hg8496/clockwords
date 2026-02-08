use clockwords::{scanner_for_languages, ExpressionKind, ResolvedTime, Span};
use chrono::TimeZone;

fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 14, 30, 0).unwrap()
}

#[test]
fn de_heute() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("heute", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
}

#[test]
fn de_gestern() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("gestern", now());
    assert_eq!(m.len(), 1);
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
fn de_morgen() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("morgen", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
}

#[test]
fn de_vor_3_tagen() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("vor 3 Tagen", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].span, Span::new(0, 11));
    assert_eq!(m[0].kind, ExpressionKind::RelativeDayOffset);
    let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 4, 0, 0, 0).unwrap();
    assert_eq!(
        m[0].resolved,
        ResolvedTime::Range {
            start: expected_start,
            end: expected_start + chrono::Duration::days(1),
        }
    );
}

#[test]
fn de_vor_zwei_tagen() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("vor zwei Tagen", now());
    assert_eq!(m.len(), 1);
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
fn de_in_3_tagen() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("in 3 Tagen", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::RelativeDayOffset);
    let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 10, 0, 0, 0).unwrap();
    assert_eq!(
        m[0].resolved,
        ResolvedTime::Range {
            start: expected_start,
            end: expected_start + chrono::Duration::days(1),
        }
    );
}

#[test]
fn de_um_15_uhr() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("um 15 Uhr", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeSpecification);
    let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 15, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn de_die_letzte_stunde() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("die letzte Stunde", now());
    assert_eq!(m.len(), 1);
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
fn de_letzte_stunde() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("letzte Stunde", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);
}

#[test]
fn de_von_9_bis_12_uhr() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("von 9 bis 12 Uhr", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);
    let start = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 9, 0, 0).unwrap();
    let end = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 12, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn de_zwischen_9_und_12_uhr() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("zwischen 9 und 12 Uhr", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);
}

#[test]
fn de_gestern_um_15_uhr() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("gestern um 15 Uhr", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 15, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn de_gestern_von_9_bis_12_uhr() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("gestern von 9 bis 12 Uhr", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let start = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap();
    let end = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn de_embedded_in_sentence() {
    let s = scanner_for_languages(&["de"]);
    let text = "Die letzte Stunde habe ich an der Bibliothek gearbeitet";
    let m = s.scan(text, now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);
}
