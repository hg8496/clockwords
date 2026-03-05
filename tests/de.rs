use chrono::TimeZone;
use clockwords::{ExpressionKind, ResolvedTime, scanner_for_languages};

fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 14, 30, 0).unwrap()
}

macro_rules! point_test {
    ($name:ident, $lang:expr, $input:expr, $kind:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let s = scanner_for_languages(&[$lang]);
            let m = s.scan($input, now());
            assert_eq!(
                m.len(),
                1,
                "expected 1 match for {:?}, got {}",
                $input,
                m.len()
            );
            assert_eq!(m[0].kind, $kind);
            assert_eq!(m[0].resolved, ResolvedTime::Point($expected));
        }
    };
}

macro_rules! range_test {
    ($name:ident, $lang:expr, $input:expr, $kind:expr, $start:expr, $end:expr) => {
        #[test]
        fn $name() {
            let s = scanner_for_languages(&[$lang]);
            let m = s.scan($input, now());
            assert_eq!(
                m.len(),
                1,
                "expected 1 match for {:?}, got {}",
                $input,
                m.len()
            );
            assert_eq!(m[0].kind, $kind);
            assert_eq!(
                m[0].resolved,
                ResolvedTime::Range {
                    start: $start,
                    end: $end
                }
            );
        }
    };
}

macro_rules! kind_test {
    ($name:ident, $lang:expr, $input:expr, $kind:expr) => {
        #[test]
        fn $name() {
            let s = scanner_for_languages(&[$lang]);
            let m = s.scan($input, now());
            assert_eq!(
                m.len(),
                1,
                "expected 1 match for {:?}, got {}",
                $input,
                m.len()
            );
            assert_eq!(m[0].kind, $kind);
        }
    };
}

// --- Relative days ---

kind_test!(de_heute, "de", "heute", ExpressionKind::RelativeDay);

range_test!(
    de_gestern,
    "de",
    "gestern",
    ExpressionKind::RelativeDay,
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 0, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap()
);

kind_test!(de_morgen, "de", "morgen", ExpressionKind::RelativeDay);

// --- Day offsets ---

range_test!(
    de_vor_3_tagen,
    "de",
    "vor 3 Tagen",
    ExpressionKind::RelativeDayOffset,
    chrono::Utc.with_ymd_and_hms(2026, 2, 4, 0, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 5, 0, 0, 0).unwrap()
);

range_test!(
    de_vor_zwei_tagen,
    "de",
    "vor zwei Tagen",
    ExpressionKind::RelativeDayOffset,
    chrono::Utc.with_ymd_and_hms(2026, 2, 5, 0, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 0, 0, 0).unwrap()
);

range_test!(
    de_in_3_tagen,
    "de",
    "in 3 Tagen",
    ExpressionKind::RelativeDayOffset,
    chrono::Utc.with_ymd_and_hms(2026, 2, 10, 0, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 11, 0, 0, 0).unwrap()
);

// --- Time specifications ---

point_test!(
    de_um_15_uhr,
    "de",
    "um 15 Uhr",
    ExpressionKind::TimeSpecification,
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 15, 0, 0).unwrap()
);

// --- Time ranges ---

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

kind_test!(
    de_letzte_stunde,
    "de",
    "letzte Stunde",
    ExpressionKind::TimeRange
);

range_test!(
    de_von_9_bis_12_uhr,
    "de",
    "von 9 bis 12 Uhr",
    ExpressionKind::TimeRange,
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 9, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 12, 0, 0).unwrap()
);

kind_test!(
    de_zwischen_9_und_12_uhr,
    "de",
    "zwischen 9 und 12 Uhr",
    ExpressionKind::TimeRange
);

// --- Combined ---

point_test!(
    de_gestern_um_15_uhr,
    "de",
    "gestern um 15 Uhr",
    ExpressionKind::Combined,
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 15, 0, 0).unwrap()
);

range_test!(
    de_gestern_von_9_bis_12_uhr,
    "de",
    "gestern von 9 bis 12 Uhr",
    ExpressionKind::Combined,
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap()
);

// --- Combined: HH:MM ranges with minutes ---

range_test!(
    de_heute_von_hhmm_bis_hhmm,
    "de",
    "heute von 10:15 bis 13:45",
    ExpressionKind::Combined,
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 10, 15, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 13, 45, 0).unwrap()
);

range_test!(
    de_heute_von_hhmm_dash_hhmm,
    "de",
    "heute von 10:15 - 13:45",
    ExpressionKind::Combined,
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 10, 15, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 13, 45, 0).unwrap()
);

range_test!(
    de_heute_hhmm_dash_hhmm,
    "de",
    "heute 8:30 - 9:30",
    ExpressionKind::Combined,
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 8, 30, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 9, 30, 0).unwrap()
);

range_test!(
    de_gestern_von_hhmm_bis_hhmm,
    "de",
    "gestern von 9:00 bis 11:30",
    ExpressionKind::Combined,
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 11, 30, 0).unwrap()
);

range_test!(
    de_von_hhmm_bis_hhmm,
    "de",
    "von 10:15 bis 13:45",
    ExpressionKind::TimeRange,
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 10, 15, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 13, 45, 0).unwrap()
);

range_test!(
    de_von_hhmm_dash_hhmm,
    "de",
    "von 9:30 - 11:00",
    ExpressionKind::TimeRange,
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 9, 30, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 11, 0, 0).unwrap()
);

// --- Embedding ---

kind_test!(
    de_embedded_in_sentence,
    "de",
    "Die letzte Stunde habe ich an der Bibliothek gearbeitet",
    ExpressionKind::TimeRange
);
