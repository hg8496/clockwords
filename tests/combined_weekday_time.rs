use clockwords::{scanner_for_languages, ExpressionKind, ResolvedTime};
use chrono::{TimeZone, Utc};

/// Sunday Feb 8, 2026 12:00:00 UTC — same reference as next_friday tests.
fn now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 2, 8, 12, 0, 0).unwrap()
}

// ================================================================
//  English
// ================================================================

#[test]
fn en_last_friday_at_3pm() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("last Friday at 3pm", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    // Last Friday from Sunday Feb 8 = Feb 6
    let expected = Utc.with_ymd_and_hms(2026, 2, 6, 15, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn en_next_monday_at_9am() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("next Monday at 9am", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    // Next Monday from Sunday Feb 8 = Feb 16
    let expected = Utc.with_ymd_and_hms(2026, 2, 16, 9, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn en_last_friday_between_9_and_12() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("last Friday between 9 and 12", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let start = Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn en_last_friday_from_9_to_eleven() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("Last Friday from 9 to eleven", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let start = Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 2, 6, 11, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn en_this_wednesday_from_nine_to_five() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("this Wednesday from nine to five", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    // This Wednesday from Sunday Feb 8 = Feb 11
    let start = Utc.with_ymd_and_hms(2026, 2, 11, 9, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 2, 11, 5, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn en_yesterday_from_9_to_11() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("yesterday from 9 to 11", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    // Yesterday from Feb 8 = Feb 7
    let start = Utc.with_ymd_and_hms(2026, 2, 7, 9, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 2, 7, 11, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn en_from_9_to_12_standalone() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("from 9 to 12", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::TimeRange);
    // Standalone "from X to Y" resolves on today (Feb 8)
    let start = Utc.with_ymd_and_hms(2026, 2, 8, 9, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 2, 8, 12, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn en_embedded_last_friday_from_9_to_11() {
    let s = scanner_for_languages(&["en"]);
    let text = "Last Friday from 9 to 11 I worked on the project";
    let m = s.scan(text, now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let start = Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 2, 6, 11, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

// ================================================================
//  German
// ================================================================

#[test]
fn de_letzten_freitag_um_15_uhr() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("letzten Freitag um 15 Uhr", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let expected = Utc.with_ymd_and_hms(2026, 2, 6, 15, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn de_naechsten_montag_um_9_uhr() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("nächsten Montag um 9 Uhr", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let expected = Utc.with_ymd_and_hms(2026, 2, 16, 9, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn de_letzten_freitag_von_9_bis_12_uhr() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("letzten Freitag von 9 bis 12 Uhr", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let start = Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn de_diesen_mittwoch_zwischen_9_und_11() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("diesen Mittwoch zwischen 9 und 11", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    // This Wednesday from Sunday Feb 8 = Feb 11
    let start = Utc.with_ymd_and_hms(2026, 2, 11, 9, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 2, 11, 11, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn de_diesen_mittwoch_von_9_bis_11() {
    let s = scanner_for_languages(&["de"]);
    let m = s.scan("diesen Mittwoch von 9 bis 11", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    // This Wednesday from Sunday Feb 8 = Feb 11
    let start = Utc.with_ymd_and_hms(2026, 2, 11, 9, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 2, 11, 11, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

// ================================================================
//  French
// ================================================================

#[test]
fn fr_vendredi_dernier_a_13h() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("vendredi dernier à 13h", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let expected = Utc.with_ymd_and_hms(2026, 2, 6, 13, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn fr_lundi_prochain_a_9h() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("lundi prochain à 9h", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let expected = Utc.with_ymd_and_hms(2026, 2, 16, 9, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn fr_vendredi_dernier_entre_9_et_12() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("vendredi dernier entre 9 et 12 heures", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let start = Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn fr_ce_lundi_a_14h() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("ce lundi à 14h", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    // This Monday from Sunday Feb 8 = Feb 9
    let expected = Utc.with_ymd_and_hms(2026, 2, 9, 14, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn fr_ce_mercredi_entre_9_et_11() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("ce mercredi entre 9 et 11 heures", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let start = Utc.with_ymd_and_hms(2026, 2, 11, 9, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 2, 11, 11, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

// ================================================================
//  Spanish
// ================================================================

#[test]
fn es_proximo_lunes_a_las_9() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("el próximo lunes a las 9", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let expected = Utc.with_ymd_and_hms(2026, 2, 16, 9, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn es_viernes_pasado_a_las_3() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("el viernes pasado a las 3", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let expected = Utc.with_ymd_and_hms(2026, 2, 6, 3, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
}

#[test]
fn es_pasado_viernes_entre_las_9_y_las_12() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("el pasado viernes entre las 9 y las 12", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let start = Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}

#[test]
fn es_viernes_pasado_entre_las_9_y_las_12() {
    let s = scanner_for_languages(&["es"]);
    let m = s.scan("el viernes pasado entre las 9 y las 12", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].kind, ExpressionKind::Combined);
    let start = Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap();
    assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
}
