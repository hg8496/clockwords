use chrono::TimeZone;
use clockwords::{ExpressionKind, MatchConfidence, ResolvedTime, Span, scanner_for_languages};

fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 14, 30, 0).unwrap()
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

/// Test expecting exactly one match with a given kind (no value check).
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

#[test]
fn en_today() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("today", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].span, Span::new(0, 5));
    assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
    assert_eq!(m[0].confidence, MatchConfidence::Complete);
}

range_test!(
    en_yesterday,
    "en",
    "yesterday",
    ExpressionKind::RelativeDay,
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 0, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap()
);

range_test!(
    en_tomorrow,
    "en",
    "tomorrow",
    ExpressionKind::RelativeDay,
    chrono::Utc.with_ymd_and_hms(2026, 2, 8, 0, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 9, 0, 0, 0).unwrap()
);

// --- Day offsets ---

range_test!(
    en_in_4_days,
    "en",
    "in 4 days",
    ExpressionKind::RelativeDayOffset,
    chrono::Utc.with_ymd_and_hms(2026, 2, 11, 0, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 12, 0, 0, 0).unwrap()
);

range_test!(
    en_two_days_ago,
    "en",
    "two days ago",
    ExpressionKind::RelativeDayOffset,
    chrono::Utc.with_ymd_and_hms(2026, 2, 5, 0, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 0, 0, 0).unwrap()
);

// --- Time specifications ---

point_test!(
    en_at_3pm,
    "en",
    "at 3pm",
    ExpressionKind::TimeSpecification,
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 15, 0, 0).unwrap()
);

point_test!(
    en_13_oclock,
    "en",
    "13 o'clock",
    ExpressionKind::TimeSpecification,
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 13, 0, 0).unwrap()
);

// --- Time ranges ---

#[test]
fn en_the_last_hour() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("the last hour", now());
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].span, Span::new(0, 13));
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
    en_last_hour_without_the,
    "en",
    "last hour",
    ExpressionKind::TimeRange
);

range_test!(
    en_between_9_and_12,
    "en",
    "between 9 and 12",
    ExpressionKind::TimeRange,
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 9, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 7, 12, 0, 0).unwrap()
);

kind_test!(
    en_between_9_and_12_oclock,
    "en",
    "between 9 and 12 o'clock",
    ExpressionKind::TimeRange
);

// --- Combined ---

point_test!(
    en_yesterday_at_3pm,
    "en",
    "yesterday at 3pm",
    ExpressionKind::Combined,
    chrono::Utc.with_ymd_and_hms(2026, 2, 6, 15, 0, 0).unwrap()
);

range_test!(
    en_tomorrow_between_9_and_12,
    "en",
    "tomorrow between 9 and 12",
    ExpressionKind::Combined,
    chrono::Utc.with_ymd_and_hms(2026, 2, 8, 9, 0, 0).unwrap(),
    chrono::Utc.with_ymd_and_hms(2026, 2, 8, 12, 0, 0).unwrap()
);

// --- Embedding / edge cases ---

kind_test!(
    en_embedded_in_sentence,
    "en",
    "The last hour I coded the initial code for the time library",
    ExpressionKind::TimeRange
);

kind_test!(
    en_case_insensitive,
    "en",
    "YESTERDAY",
    ExpressionKind::RelativeDay
);

#[test]
fn en_no_match() {
    let s = scanner_for_languages(&["en"]);
    let m = s.scan("I wrote some code", now());
    assert_eq!(m.len(), 0);
}
