use clockwords::{default_scanner, scanner_for_languages, MatchConfidence};
use chrono::TimeZone;

fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 14, 30, 0).unwrap()
}

#[test]
fn incremental_no_match_short_prefix() {
    let s = default_scanner();
    let m = s.scan("ye", now());
    assert_eq!(m.len(), 0, "Two characters should not match anything");
}

#[test]
fn incremental_partial_yester() {
    let s = default_scanner();
    let m = s.scan("yester", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].confidence, MatchConfidence::Partial);
}

#[test]
fn incremental_complete_yesterday() {
    let s = default_scanner();
    let m = s.scan("yesterday", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].confidence, MatchConfidence::Complete);
}

#[test]
fn incremental_partial_in_sentence() {
    let s = default_scanner();
    let m = s.scan("I worked yester", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].confidence, MatchConfidence::Partial);
}

#[test]
fn incremental_partial_german() {
    let s = default_scanner();
    let m = s.scan("gest", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].confidence, MatchConfidence::Partial);
}

#[test]
fn default_scanner_finds_english() {
    let s = default_scanner();
    let m = s.scan("yesterday", now());
    assert!(!m.is_empty());
}

#[test]
fn default_scanner_finds_german() {
    let s = default_scanner();
    let m = s.scan("gestern", now());
    assert!(!m.is_empty());
}

#[test]
fn default_scanner_finds_french() {
    let s = default_scanner();
    let m = s.scan("hier", now());
    assert!(!m.is_empty());
}

#[test]
fn default_scanner_finds_spanish() {
    let s = default_scanner();
    let m = s.scan("ayer", now());
    assert!(!m.is_empty());
}

#[test]
fn empty_input() {
    let s = default_scanner();
    let m = s.scan("", now());
    assert_eq!(m.len(), 0);
}

#[test]
fn no_false_positive_on_similar_words() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("The dog played all day", now());
    // "day" alone should not trigger a time match
    assert_eq!(m.len(), 0);
}
