use chrono::{TimeZone, Utc};
use clockwords::{ExpressionKind, ResolvedTime, scanner_for_languages};

/// Sunday Feb 8, 2026 12:00:00 UTC — same reference as next_friday tests.
fn now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 2, 8, 12, 0, 0).unwrap()
}

/// Test expecting exactly one match resolving to a point in time.
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

/// Test expecting exactly one match resolving to a time range.
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

// ================================================================
//  English
// ================================================================

point_test!(
    en_last_friday_at_3pm,
    "en",
    "last Friday at 3pm",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 15, 0, 0).unwrap()
);

point_test!(
    en_next_monday_at_9am,
    "en",
    "next Monday at 9am",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 16, 9, 0, 0).unwrap()
);

range_test!(
    en_last_friday_between_9_and_12,
    "en",
    "last Friday between 9 and 12",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap(),
    Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap()
);

range_test!(
    en_last_friday_from_9_to_eleven,
    "en",
    "Last Friday from 9 to eleven",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap(),
    Utc.with_ymd_and_hms(2026, 2, 6, 11, 0, 0).unwrap()
);

range_test!(
    en_this_wednesday_from_nine_to_five,
    "en",
    "this Wednesday from nine to five",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 11, 9, 0, 0).unwrap(),
    Utc.with_ymd_and_hms(2026, 2, 11, 5, 0, 0).unwrap()
);

range_test!(
    en_yesterday_from_9_to_11,
    "en",
    "yesterday from 9 to 11",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 7, 9, 0, 0).unwrap(),
    Utc.with_ymd_and_hms(2026, 2, 7, 11, 0, 0).unwrap()
);

range_test!(
    en_from_9_to_12_standalone,
    "en",
    "from 9 to 12",
    ExpressionKind::TimeRange,
    Utc.with_ymd_and_hms(2026, 2, 8, 9, 0, 0).unwrap(),
    Utc.with_ymd_and_hms(2026, 2, 8, 12, 0, 0).unwrap()
);

range_test!(
    en_embedded_last_friday_from_9_to_11,
    "en",
    "Last Friday from 9 to 11 I worked on the project",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap(),
    Utc.with_ymd_and_hms(2026, 2, 6, 11, 0, 0).unwrap()
);

// ================================================================
//  German
// ================================================================

point_test!(
    de_letzten_freitag_um_15_uhr,
    "de",
    "letzten Freitag um 15 Uhr",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 15, 0, 0).unwrap()
);

point_test!(
    de_naechsten_montag_um_9_uhr,
    "de",
    "nächsten Montag um 9 Uhr",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 16, 9, 0, 0).unwrap()
);

range_test!(
    de_letzten_freitag_von_9_bis_12_uhr,
    "de",
    "letzten Freitag von 9 bis 12 Uhr",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap(),
    Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap()
);

range_test!(
    de_diesen_mittwoch_zwischen_9_und_11,
    "de",
    "diesen Mittwoch zwischen 9 und 11",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 11, 9, 0, 0).unwrap(),
    Utc.with_ymd_and_hms(2026, 2, 11, 11, 0, 0).unwrap()
);

range_test!(
    de_diesen_mittwoch_von_9_bis_11,
    "de",
    "diesen Mittwoch von 9 bis 11",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 11, 9, 0, 0).unwrap(),
    Utc.with_ymd_and_hms(2026, 2, 11, 11, 0, 0).unwrap()
);

// ================================================================
//  French
// ================================================================

point_test!(
    fr_vendredi_dernier_a_13h,
    "fr",
    "vendredi dernier à 13h",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 13, 0, 0).unwrap()
);

point_test!(
    fr_lundi_prochain_a_9h,
    "fr",
    "lundi prochain à 9h",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 16, 9, 0, 0).unwrap()
);

range_test!(
    fr_vendredi_dernier_entre_9_et_12,
    "fr",
    "vendredi dernier entre 9 et 12 heures",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap(),
    Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap()
);

point_test!(
    fr_ce_lundi_a_14h,
    "fr",
    "ce lundi à 14h",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 9, 14, 0, 0).unwrap()
);

range_test!(
    fr_ce_mercredi_entre_9_et_11,
    "fr",
    "ce mercredi entre 9 et 11 heures",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 11, 9, 0, 0).unwrap(),
    Utc.with_ymd_and_hms(2026, 2, 11, 11, 0, 0).unwrap()
);

// ================================================================
//  Spanish
// ================================================================

point_test!(
    es_proximo_lunes_a_las_9,
    "es",
    "el próximo lunes a las 9",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 16, 9, 0, 0).unwrap()
);

point_test!(
    es_viernes_pasado_a_las_3,
    "es",
    "el viernes pasado a las 3",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 3, 0, 0).unwrap()
);

range_test!(
    es_pasado_viernes_entre_las_9_y_las_12,
    "es",
    "el pasado viernes entre las 9 y las 12",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap(),
    Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap()
);

range_test!(
    es_viernes_pasado_entre_las_9_y_las_12,
    "es",
    "el viernes pasado entre las 9 y las 12",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap(),
    Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap()
);
