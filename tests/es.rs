use chrono::TimeZone;
use clockwords::{ExpressionKind, ResolvedTime, scanner_for_languages};

fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 14, 30, 0).unwrap()
}

#[test]
fn es_hoy() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("hoy", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
}

#[test]
fn es_ayer() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("ayer", now());
    assert_eq!(m.len(), 1);
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
fn es_manana() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("ma\u{f1}ana", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
}

#[test]
fn es_manana_ascii() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("manana", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
}

#[test]
fn es_hace_2_dias() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("hace 2 d\u{ed}as", now());
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
fn es_hace_2_dias_ascii() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("hace 2 dias", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::RelativeDayOffset);
}

#[test]
fn es_en_3_dias() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("en 3 d\u{ed}as", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::RelativeDayOffset);
}

#[test]
fn es_a_las_3() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("a las 3", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeSpecification);
    let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 3, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn es_la_ultima_hora() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("la \u{fa}ltima hora", now());
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
fn es_la_ultima_hora_ascii() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("la ultima hora", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);
}

#[test]
fn es_entre_las_9_y_las_12() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("entre las 9 y las 12", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);
    let start = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 9, 0, 0).unwrap();
    let end = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 12, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn es_ayer_a_las_3() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("ayer a las 3", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 3, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn es_ayer_entre_las_9_y_las_12() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("ayer entre las 9 y las 12", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let start = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap();
    let end = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn es_embedded_in_sentence() {
    let s = scanner_for_languages(&["es"]);
    let text = "La \u{fa}ltima hora estuve trabajando en el proyecto";
    let m = s.scan(text, now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);
}
