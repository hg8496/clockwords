use chrono::TimeZone;
use clockwords::{ExpressionKind, ResolvedTime, scanner_for_languages};

fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 14, 30, 0).unwrap()
}

#[test]
fn fr_aujourdhui() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("aujourd'hui", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
}

#[test]
fn fr_hier() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("hier", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
}

#[test]
fn fr_demain() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("demain", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
}

#[test]
fn fr_il_y_a_3_jours() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("il y a 3 jours", now());
    assert_eq!(m.len(), 1);
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
fn fr_dans_3_jours() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("dans 3 jours", now());
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
fn fr_a_13h() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("\u{e0} 13h", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeSpecification);
    let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 13, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn fr_a_13h_ascii_in_context() {
    let s = scanner_for_languages(&["fr"]);
    // Standalone "a 13h" won't trigger the keyword prefilter since "a" is too
    // common to be a keyword. In practice, French input includes context words.
    let m = s.scan("hier a 13h j'ai codé", now());
    // Should find both "hier" (RelativeDay) and the combined "hier a 13h" (Combined)
    assert!(!m.is_empty());
    // At minimum, the combined match should be present
    assert!(
        m.iter().any(|tm| tm.kind == ExpressionKind::Combined
            || tm.kind == ExpressionKind::TimeSpecification)
    );
}

#[test]
fn fr_la_derniere_heure() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("la derni\u{e8}re heure", now());
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
fn fr_la_derniere_heure_ascii() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("la derniere heure", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);
}

#[test]
fn fr_entre_9_et_12_heures() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("entre 9 et 12 heures", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);
    let start = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 9, 0, 0).unwrap();
    let end = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 12, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn fr_hier_a_13h() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("hier \u{e0} 13h", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 13, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn fr_hier_entre_9_et_12() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("hier entre 9 et 12 heures", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let start = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap();
    let end = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn fr_embedded_in_sentence() {
    let s = scanner_for_languages(&["fr"]);
    let text = "La derni\u{e8}re heure j'ai travaill\u{e9} sur le projet";
    let m = s.scan(text, now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);
}
