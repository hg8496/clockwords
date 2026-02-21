use chrono::{TimeZone, Utc};
use clockwords::{ExpressionKind, ResolvedTime, scanner_for_languages};

/// Saturday Feb 7, 2026 14:30:00 UTC
fn now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 2, 7, 14, 30, 0).unwrap()
}

/// Sunday Feb 8, 2026 12:00:00 UTC (same as combined_weekday_time tests)
fn now_sunday() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 2, 8, 12, 0, 0).unwrap()
}

/// Generate a test that expects exactly one match resolving to a point in time.
macro_rules! time_test {
    ($name:ident, $lang:expr, $input:expr, $kind:expr, $expected:expr, $now:expr) => {
        #[test]
        fn $name() {
            let s = scanner_for_languages(&[$lang]);
            let m = s.scan($input, $now);
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

// ================================================================
//  English
// ================================================================

time_test!(
    en_at_3_30pm,
    "en",
    "at 3:30pm",
    ExpressionKind::TimeSpecification,
    Utc.with_ymd_and_hms(2026, 2, 7, 15, 30, 0).unwrap(),
    now()
);

time_test!(
    en_at_11_30am,
    "en",
    "at 11:30am",
    ExpressionKind::TimeSpecification,
    Utc.with_ymd_and_hms(2026, 2, 7, 11, 30, 0).unwrap(),
    now()
);

time_test!(
    en_at_15_30_bare,
    "en",
    "at 15:30",
    ExpressionKind::TimeSpecification,
    Utc.with_ymd_and_hms(2026, 2, 7, 15, 30, 0).unwrap(),
    now()
);

time_test!(
    en_at_9_00,
    "en",
    "at 9:00",
    ExpressionKind::TimeSpecification,
    Utc.with_ymd_and_hms(2026, 2, 7, 9, 0, 0).unwrap(),
    now()
);

time_test!(
    en_yesterday_at_3_30pm,
    "en",
    "yesterday at 3:30pm",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 15, 30, 0).unwrap(),
    now()
);

time_test!(
    en_tomorrow_at_11_15am,
    "en",
    "tomorrow at 11:15am",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 8, 11, 15, 0).unwrap(),
    now()
);

time_test!(
    en_yesterday_at_15_30_bare,
    "en",
    "yesterday at 15:30",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 15, 30, 0).unwrap(),
    now()
);

time_test!(
    en_last_friday_at_3_30pm,
    "en",
    "last Friday at 3:30pm",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 15, 30, 0).unwrap(),
    now_sunday()
);

time_test!(
    en_next_monday_at_9_15am,
    "en",
    "next Monday at 9:15am",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 16, 9, 15, 0).unwrap(),
    now_sunday()
);

time_test!(
    en_last_friday_at_15_30_bare,
    "en",
    "last Friday at 15:30",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 15, 30, 0).unwrap(),
    now_sunday()
);

time_test!(
    en_12_30pm,
    "en",
    "12:30pm",
    ExpressionKind::TimeSpecification,
    Utc.with_ymd_and_hms(2026, 2, 7, 12, 30, 0).unwrap(),
    now()
);

time_test!(
    en_12_30am,
    "en",
    "12:30am",
    ExpressionKind::TimeSpecification,
    Utc.with_ymd_and_hms(2026, 2, 7, 0, 30, 0).unwrap(),
    now()
);

time_test!(
    en_embedded_colon_time,
    "en",
    "The meeting is at 11:30am in the conference room",
    ExpressionKind::TimeSpecification,
    Utc.with_ymd_and_hms(2026, 2, 7, 11, 30, 0).unwrap(),
    now()
);

// ================================================================
//  German
// ================================================================

time_test!(
    de_um_15_30_uhr,
    "de",
    "um 15:30 Uhr",
    ExpressionKind::TimeSpecification,
    Utc.with_ymd_and_hms(2026, 2, 7, 15, 30, 0).unwrap(),
    now()
);

time_test!(
    de_um_15_30_without_uhr,
    "de",
    "um 15:30",
    ExpressionKind::TimeSpecification,
    Utc.with_ymd_and_hms(2026, 2, 7, 15, 30, 0).unwrap(),
    now()
);

time_test!(
    de_gestern_um_15_30_uhr,
    "de",
    "gestern um 15:30 Uhr",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 15, 30, 0).unwrap(),
    now()
);

time_test!(
    de_gestern_um_15_30_without_uhr,
    "de",
    "gestern um 15:30",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 15, 30, 0).unwrap(),
    now()
);

time_test!(
    de_letzten_freitag_um_15_30_uhr,
    "de",
    "letzten Freitag um 15:30 Uhr",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 15, 30, 0).unwrap(),
    now_sunday()
);

time_test!(
    de_naechsten_montag_um_9_15,
    "de",
    "nächsten Montag um 9:15",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 16, 9, 15, 0).unwrap(),
    now_sunday()
);

// ================================================================
//  French
// ================================================================

time_test!(
    fr_a_13h30,
    "fr",
    "à 13h30",
    ExpressionKind::TimeSpecification,
    Utc.with_ymd_and_hms(2026, 2, 7, 13, 30, 0).unwrap(),
    now()
);

time_test!(
    fr_a_13_colon_30,
    "fr",
    "à 13:30",
    ExpressionKind::TimeSpecification,
    Utc.with_ymd_and_hms(2026, 2, 7, 13, 30, 0).unwrap(),
    now()
);

time_test!(
    fr_hier_a_13h30,
    "fr",
    "hier à 13h30",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 13, 30, 0).unwrap(),
    now()
);

time_test!(
    fr_hier_a_13_colon_30,
    "fr",
    "hier à 13:30",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 13, 30, 0).unwrap(),
    now()
);

time_test!(
    fr_vendredi_dernier_a_13h30,
    "fr",
    "vendredi dernier à 13h30",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 13, 30, 0).unwrap(),
    now_sunday()
);

time_test!(
    fr_vendredi_dernier_a_13_colon_30,
    "fr",
    "vendredi dernier à 13:30",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 13, 30, 0).unwrap(),
    now_sunday()
);

time_test!(
    fr_ce_lundi_a_14h30,
    "fr",
    "ce lundi à 14h30",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 9, 14, 30, 0).unwrap(),
    now_sunday()
);

time_test!(
    fr_ce_lundi_a_14_colon_30,
    "fr",
    "ce lundi à 14:30",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 9, 14, 30, 0).unwrap(),
    now_sunday()
);

// ================================================================
//  Spanish
// ================================================================

time_test!(
    es_a_las_15_30,
    "es",
    "a las 15:30",
    ExpressionKind::TimeSpecification,
    Utc.with_ymd_and_hms(2026, 2, 7, 15, 30, 0).unwrap(),
    now()
);

time_test!(
    es_ayer_a_las_15_30,
    "es",
    "ayer a las 15:30",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 15, 30, 0).unwrap(),
    now()
);

time_test!(
    es_proximo_lunes_a_las_9_30,
    "es",
    "el próximo lunes a las 9:30",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 16, 9, 30, 0).unwrap(),
    now_sunday()
);

time_test!(
    es_viernes_pasado_a_las_3_30,
    "es",
    "el viernes pasado a las 3:30",
    ExpressionKind::Combined,
    Utc.with_ymd_and_hms(2026, 2, 6, 3, 30, 0).unwrap(),
    now_sunday()
);

// ================================================================
//  Edge cases
// ================================================================

#[test]
fn en_invalid_minutes_60() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("at 3:60pm", now());
    assert_eq!(m.len(), 0);
}

time_test!(
    en_midnight_12_30am,
    "en",
    "at 12:30am",
    ExpressionKind::TimeSpecification,
    Utc.with_ymd_and_hms(2026, 2, 7, 0, 30, 0).unwrap(),
    now()
);

time_test!(
    en_noon_12_30pm,
    "en",
    "at 12:30pm",
    ExpressionKind::TimeSpecification,
    Utc.with_ymd_and_hms(2026, 2, 7, 12, 30, 0).unwrap(),
    now()
);
