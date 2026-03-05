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

kind_test!(
    fr_aujourdhui,
    "fr",
    "aujourd'hui",
    ExpressionKind::RelativeDay
);
kind_test!(fr_hier, "fr", "hier", ExpressionKind::RelativeDay);
kind_test!(fr_demain, "fr", "demain", ExpressionKind::RelativeDay);

// --- Day offsets ---

range_test!(
    fr_il_y_a_3_jours,
    "fr",
    "il y a 3 jours",
    ExpressionKind::RelativeDayOffset,
    chrono::Utc.with_ymd_and_hms(2026, 2, 4, 0, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 5, 0, 0, 0).unwrap()
);

range_test!(
    fr_dans_3_jours,
    "fr",
    "dans 3 jours",
    ExpressionKind::RelativeDayOffset,
    chrono::Utc.with_ymd_and_hms(2026, 2, 10, 0, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 11, 0, 0, 0).unwrap()
);

// --- Time specifications ---

point_test!(
    fr_a_13h,
    "fr",
    "\u{e0} 13h",
    ExpressionKind::TimeSpecification,
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 13, 0, 0).unwrap()
);

#[test]
fn fr_a_13h_ascii_in_context() {
    let s = scanner_for_languages(&["fr"]);
    let m = s.scan("hier a 13h j'ai codé", now());
    assert!(!m.is_empty());
    assert!(
        m.iter().any(|tm| tm.kind == ExpressionKind::Combined
            || tm.kind == ExpressionKind::TimeSpecification)
    );
}

// --- Time ranges ---

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

kind_test!(
    fr_la_derniere_heure_ascii,
    "fr",
    "la derniere heure",
    ExpressionKind::TimeRange
);

range_test!(
    fr_entre_9_et_12_heures,
    "fr",
    "entre 9 et 12 heures",
    ExpressionKind::TimeRange,
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 9, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 12, 0, 0).unwrap()
);

// --- Combined ---

point_test!(
    fr_hier_a_13h,
    "fr",
    "hier \u{e0} 13h",
    ExpressionKind::Combined,
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 13, 0, 0).unwrap()
);

range_test!(
    fr_hier_entre_9_et_12,
    "fr",
    "hier entre 9 et 12 heures",
    ExpressionKind::Combined,
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap()
);

// --- Combined: HH:MM ranges with minutes ---

range_test!(
    fr_hier_de_hhmm_a_hhmm,
    "fr",
    "hier de 10:15 \u{e0} 13:45",
    ExpressionKind::Combined,
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 10, 15, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 13, 45, 0).unwrap()
);

range_test!(
    fr_hier_de_hhmm_dash_hhmm,
    "fr",
    "hier de 9:00 - 11:30",
    ExpressionKind::Combined,
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 11, 30, 0).unwrap()
);

range_test!(
    fr_hier_hhmm_dash_hhmm,
    "fr",
    "hier 10:15 - 13:45",
    ExpressionKind::Combined,
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 10, 15, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 13, 45, 0).unwrap()
);

range_test!(
    fr_de_hhmm_a_hhmm,
    "fr",
    "de 10:15 \u{e0} 13:45",
    ExpressionKind::TimeRange,
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 10, 15, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 13, 45, 0).unwrap()
);

// --- Embedding ---

kind_test!(
    fr_embedded_in_sentence,
    "fr",
    "La derni\u{e8}re heure j'ai travaill\u{e9} sur le projet",
    ExpressionKind::TimeRange
);
