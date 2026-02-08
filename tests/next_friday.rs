use chrono::{TimeZone, Utc};
use clockwords::{ResolvedTime, scanner_for_languages};

fn assert_range(resolved: ResolvedTime, expected_start_ymd: (i32, u32, u32)) {
    match resolved {
        ResolvedTime::Range { start, .. } => {
            let expected = Utc
                .with_ymd_and_hms(
                    expected_start_ymd.0,
                    expected_start_ymd.1,
                    expected_start_ymd.2,
                    0,
                    0,
                    0,
                )
                .unwrap();
            assert_eq!(
                start, expected,
                "Expected start date {}, got {}",
                expected, start
            );
        }
        _ => panic!("Expected Range resolution"),
    }
}

#[test]
fn test_english_weekdays() {
    let s = scanner_for_languages(&["en"]);
    // Sunday Feb 8, 2026.
    let now = Utc.with_ymd_and_hms(2026, 2, 8, 12, 0, 0).unwrap();

    // "This Friday" -> Feb 13 (coming)
    let m = s.scan("this friday", now);
    assert_eq!(m.len(), 1);
    assert_range(m[0].resolved.clone(), (2026, 2, 13));

    // "Next Friday" -> Feb 20 (following week)
    let m = s.scan("next friday", now);
    assert_eq!(m.len(), 1);
    assert_range(m[0].resolved.clone(), (2026, 2, 20));

    // "Last Friday" -> Feb 6 (past)
    let m = s.scan("last Friday", now);
    assert_eq!(m.len(), 1);
    assert_range(m[0].resolved.clone(), (2026, 2, 6));
}

#[test]
fn test_german_weekdays() {
    let s = scanner_for_languages(&["de"]);
    let now = Utc.with_ymd_and_hms(2026, 2, 8, 12, 0, 0).unwrap(); // Sunday

    // "diesen Freitag" -> Feb 13
    let m = s.scan("diesen Freitag", now);
    assert_eq!(m.len(), 1);
    assert_range(m[0].resolved.clone(), (2026, 2, 13));

    // "nächsten Freitag" -> Feb 20
    let m = s.scan("nächsten Freitag", now);
    assert_eq!(m.len(), 1);
    assert_range(m[0].resolved.clone(), (2026, 2, 20));

    // "letzten Freitag" -> Feb 6
    let m = s.scan("letzten Freitag", now);
    assert_eq!(m.len(), 1);
    assert_range(m[0].resolved.clone(), (2026, 2, 6));
}

#[test]
fn test_french_weekdays() {
    let s = scanner_for_languages(&["fr"]);
    let now = Utc.with_ymd_and_hms(2026, 2, 8, 12, 0, 0).unwrap(); // Sunday

    // "ce vendredi" -> Feb 13
    let m = s.scan("ce vendredi", now);
    assert_eq!(m.len(), 1);
    assert_range(m[0].resolved.clone(), (2026, 2, 13));

    // "vendredi prochain" -> Feb 20
    let m = s.scan("vendredi prochain", now);
    assert_eq!(m.len(), 1);
    assert_range(m[0].resolved.clone(), (2026, 2, 20));

    // "vendredi dernier" -> Feb 6
    let m = s.scan("vendredi dernier", now);
    assert_eq!(m.len(), 1);
    assert_range(m[0].resolved.clone(), (2026, 2, 6));
}

#[test]
fn test_spanish_weekdays() {
    let s = scanner_for_languages(&["es"]);
    let now = Utc.with_ymd_and_hms(2026, 2, 8, 12, 0, 0).unwrap(); // Sunday

    // "este viernes" -> Feb 13
    let m = s.scan("este viernes", now);
    assert_eq!(m.len(), 1, "Failed to match 'este viernes'");
    assert_range(m[0].resolved.clone(), (2026, 2, 13));

    // "el próximo viernes" -> Feb 20
    let m = s.scan("el próximo viernes", now);
    assert_eq!(m.len(), 1);
    assert_range(m[0].resolved.clone(), (2026, 2, 20));

    // "el viernes pasado" -> Feb 6
    let m = s.scan("el viernes pasado", now);
    assert_eq!(m.len(), 1);
    assert_range(m[0].resolved.clone(), (2026, 2, 6));

    // "el viernes que viene" -> Feb 20
    let m = s.scan("el viernes que viene", now);
    assert_eq!(m.len(), 1);
    assert_range(m[0].resolved.clone(), (2026, 2, 20));
}
