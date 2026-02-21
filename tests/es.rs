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

kind_test!(es_hoy, "es", "hoy", ExpressionKind::RelativeDay);

range_test!(
    es_ayer,
    "es",
    "ayer",
    ExpressionKind::RelativeDay,
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 0, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap()
);

kind_test!(es_manana, "es", "ma\u{f1}ana", ExpressionKind::RelativeDay);
kind_test!(es_manana_ascii, "es", "manana", ExpressionKind::RelativeDay);

// --- Day offsets ---

range_test!(
    es_hace_2_dias,
    "es",
    "hace 2 d\u{ed}as",
    ExpressionKind::RelativeDayOffset,
    chrono::Utc.with_ymd_and_hms(2026, 2, 5, 0, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 0, 0, 0).unwrap()
);

kind_test!(
    es_hace_2_dias_ascii,
    "es",
    "hace 2 dias",
    ExpressionKind::RelativeDayOffset
);
kind_test!(
    es_en_3_dias,
    "es",
    "en 3 d\u{ed}as",
    ExpressionKind::RelativeDayOffset
);

// --- Time specifications ---

point_test!(
    es_a_las_3,
    "es",
    "a las 3",
    ExpressionKind::TimeSpecification,
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 3, 0, 0).unwrap()
);

// --- Time ranges ---

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

kind_test!(
    es_la_ultima_hora_ascii,
    "es",
    "la ultima hora",
    ExpressionKind::TimeRange
);

range_test!(
    es_entre_las_9_y_las_12,
    "es",
    "entre las 9 y las 12",
    ExpressionKind::TimeRange,
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 9, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 12, 0, 0).unwrap()
);

// --- Combined ---

point_test!(
    es_ayer_a_las_3,
    "es",
    "ayer a las 3",
    ExpressionKind::Combined,
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 3, 0, 0).unwrap()
);

range_test!(
    es_ayer_entre_las_9_y_las_12,
    "es",
    "ayer entre las 9 y las 12",
    ExpressionKind::Combined,
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap()
);

// --- Embedding ---

kind_test!(
    es_embedded_in_sentence,
    "es",
    "La \u{fa}ltima hora estuve trabajando en el proyecto",
    ExpressionKind::TimeRange
);
