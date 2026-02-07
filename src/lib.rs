pub mod lang;
pub mod resolve;
pub mod scanner;
pub mod types;

pub use scanner::TimeExpressionScanner;
pub use types::*;

/// Create a scanner with all four languages enabled (EN, DE, FR, ES).
pub fn default_scanner() -> TimeExpressionScanner {
    scanner_for_languages(&["en", "de", "fr", "es"])
}

/// Create a scanner for specific languages.
///
/// Supported language ids: `"en"`, `"de"`, `"fr"`, `"es"`.
/// Languages are tried in the order given; earlier languages take priority
/// when deduplicating overlapping matches.
pub fn scanner_for_languages(lang_ids: &[&str]) -> TimeExpressionScanner {
    let languages: Vec<Box<dyn lang::LanguageParser>> = lang_ids
        .iter()
        .filter_map(|id| match *id {
            "en" => Some(Box::new(lang::en::English::new()) as Box<dyn lang::LanguageParser>),
            "de" => Some(Box::new(lang::de::German::new()) as Box<dyn lang::LanguageParser>),
            "fr" => Some(Box::new(lang::fr::French::new()) as Box<dyn lang::LanguageParser>),
            "es" => Some(Box::new(lang::es::Spanish::new()) as Box<dyn lang::LanguageParser>),
            _ => None,
        })
        .collect();

    TimeExpressionScanner::new(languages, ParserConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn now() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc.with_ymd_and_hms(2026, 2, 7, 14, 30, 0).unwrap()
    }

    // ──────────────────────────────────────────────
    //  English tests
    // ──────────────────────────────────────────────

    #[test]
    fn en_today() {
        let s = scanner_for_languages(&["en"]);
        let m = s.scan("today", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].span, Span::new(0, 5));
        assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
        assert_eq!(m[0].confidence, MatchConfidence::Complete);
    }

    #[test]
    fn en_yesterday() {
        let s = scanner_for_languages(&["en"]);
        let m = s.scan("yesterday", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].span, Span::new(0, 9));
        assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
        let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 0, 0, 0).unwrap();
        let expected_end = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap();
        assert_eq!(
            m[0].resolved,
            ResolvedTime::Range {
                start: expected_start,
                end: expected_end,
            }
        );
    }

    #[test]
    fn en_tomorrow() {
        let s = scanner_for_languages(&["en"]);
        let m = s.scan("tomorrow", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].span, Span::new(0, 8));
        let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 8, 0, 0, 0).unwrap();
        assert_eq!(
            m[0].resolved,
            ResolvedTime::Range {
                start: expected_start,
                end: expected_start + chrono::Duration::days(1),
            }
        );
    }

    #[test]
    fn en_in_4_days() {
        let s = scanner_for_languages(&["en"]);
        let m = s.scan("in 4 days", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].span, Span::new(0, 9));
        assert_eq!(m[0].kind, ExpressionKind::RelativeDayOffset);
        let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 11, 0, 0, 0).unwrap();
        assert_eq!(
            m[0].resolved,
            ResolvedTime::Range {
                start: expected_start,
                end: expected_start + chrono::Duration::days(1),
            }
        );
    }

    #[test]
    fn en_two_days_ago() {
        let s = scanner_for_languages(&["en"]);
        let m = s.scan("two days ago", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].span, Span::new(0, 12));
        assert_eq!(m[0].kind, ExpressionKind::RelativeDayOffset);
        let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 5, 0, 0, 0).unwrap();
        assert_eq!(
            m[0].resolved,
            ResolvedTime::Range {
                start: expected_start,
                end: expected_start + chrono::Duration::days(1),
            }
        );
    }

    #[test]
    fn en_at_3pm() {
        let s = scanner_for_languages(&["en"]);
        let m = s.scan("at 3pm", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].span, Span::new(0, 6));
        assert_eq!(m[0].kind, ExpressionKind::TimeSpecification);
        let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 15, 0, 0).unwrap();
        assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
    }

    #[test]
    fn en_13_oclock() {
        let s = scanner_for_languages(&["en"]);
        let m = s.scan("13 o'clock", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::TimeSpecification);
        let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 13, 0, 0).unwrap();
        assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
    }

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

    #[test]
    fn en_last_hour_without_the() {
        let s = scanner_for_languages(&["en"]);
        let m = s.scan("last hour", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::TimeRange);
    }

    #[test]
    fn en_between_9_and_12() {
        let s = scanner_for_languages(&["en"]);
        let m = s.scan("between 9 and 12", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::TimeRange);
        let start = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 9, 0, 0).unwrap();
        let end = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 12, 0, 0).unwrap();
        assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
    }

    #[test]
    fn en_between_9_and_12_oclock() {
        let s = scanner_for_languages(&["en"]);
        let m = s.scan("between 9 and 12 o'clock", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::TimeRange);
    }

    #[test]
    fn en_yesterday_at_3pm() {
        let s = scanner_for_languages(&["en"]);
        let m = s.scan("yesterday at 3pm", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].span, Span::new(0, 16));
        assert_eq!(m[0].kind, ExpressionKind::Combined);
        let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 15, 0, 0).unwrap();
        assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
    }

    #[test]
    fn en_tomorrow_between_9_and_12() {
        let s = scanner_for_languages(&["en"]);
        let m = s.scan("tomorrow between 9 and 12", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::Combined);
        let start = chrono::Utc.with_ymd_and_hms(2026, 2, 8, 9, 0, 0).unwrap();
        let end = chrono::Utc.with_ymd_and_hms(2026, 2, 8, 12, 0, 0).unwrap();
        assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
    }

    #[test]
    fn en_embedded_in_sentence() {
        let s = scanner_for_languages(&["en"]);
        let text = "The last hour I coded the initial code for the time library";
        let m = s.scan(text, now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::TimeRange);
        assert!(m[0].span.end <= 14, "span end should be at most 14, got {}", m[0].span.end);
    }

    #[test]
    fn en_case_insensitive() {
        let s = scanner_for_languages(&["en"]);
        let m = s.scan("YESTERDAY", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
    }

    #[test]
    fn en_no_match() {
        let s = scanner_for_languages(&["en"]);
        let m = s.scan("I wrote some code", now());
        assert_eq!(m.len(), 0);
    }

    // ──────────────────────────────────────────────
    //  German tests
    // ──────────────────────────────────────────────

    #[test]
    fn de_heute() {
        let s = scanner_for_languages(&["de"]);
        let m = s.scan("heute", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
    }

    #[test]
    fn de_gestern() {
        let s = scanner_for_languages(&["de"]);
        let m = s.scan("gestern", now());
        assert_eq!(m.len(), 1);
        let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 0, 0, 0).unwrap();
        let expected_end = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap();
        assert_eq!(
            m[0].resolved,
            ResolvedTime::Range {
                start: expected_start,
                end: expected_end,
            }
        );
    }

    #[test]
    fn de_morgen() {
        let s = scanner_for_languages(&["de"]);
        let m = s.scan("morgen", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
    }

    #[test]
    fn de_vor_3_tagen() {
        let s = scanner_for_languages(&["de"]);
        let m = s.scan("vor 3 Tagen", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].span, Span::new(0, 11));
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
    fn de_vor_zwei_tagen() {
        let s = scanner_for_languages(&["de"]);
        let m = s.scan("vor zwei Tagen", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::RelativeDayOffset);
        let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 5, 0, 0, 0).unwrap();
        assert_eq!(
            m[0].resolved,
            ResolvedTime::Range {
                start: expected_start,
                end: expected_start + chrono::Duration::days(1),
            }
        );
    }

    #[test]
    fn de_in_3_tagen() {
        let s = scanner_for_languages(&["de"]);
        let m = s.scan("in 3 Tagen", now());
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
    fn de_um_15_uhr() {
        let s = scanner_for_languages(&["de"]);
        let m = s.scan("um 15 Uhr", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::TimeSpecification);
        let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 15, 0, 0).unwrap();
        assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
    }

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

    #[test]
    fn de_letzte_stunde() {
        let s = scanner_for_languages(&["de"]);
        let m = s.scan("letzte Stunde", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::TimeRange);
    }

    #[test]
    fn de_von_9_bis_12_uhr() {
        let s = scanner_for_languages(&["de"]);
        let m = s.scan("von 9 bis 12 Uhr", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::TimeRange);
        let start = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 9, 0, 0).unwrap();
        let end = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 12, 0, 0).unwrap();
        assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
    }

    #[test]
    fn de_zwischen_9_und_12_uhr() {
        let s = scanner_for_languages(&["de"]);
        let m = s.scan("zwischen 9 und 12 Uhr", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::TimeRange);
    }

    #[test]
    fn de_gestern_um_15_uhr() {
        let s = scanner_for_languages(&["de"]);
        let m = s.scan("gestern um 15 Uhr", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::Combined);
        let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 15, 0, 0).unwrap();
        assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
    }

    #[test]
    fn de_gestern_von_9_bis_12_uhr() {
        let s = scanner_for_languages(&["de"]);
        let m = s.scan("gestern von 9 bis 12 Uhr", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::Combined);
        let start = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap();
        let end = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap();
        assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
    }

    #[test]
    fn de_embedded_in_sentence() {
        let s = scanner_for_languages(&["de"]);
        let text = "Die letzte Stunde habe ich an der Bibliothek gearbeitet";
        let m = s.scan(text, now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::TimeRange);
    }

    // ──────────────────────────────────────────────
    //  French tests
    // ──────────────────────────────────────────────

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
        assert!(m.iter().any(|tm| tm.kind == ExpressionKind::Combined
            || tm.kind == ExpressionKind::TimeSpecification));
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

    // ──────────────────────────────────────────────
    //  Spanish tests
    // ──────────────────────────────────────────────

    #[test]
    fn es_hoy() {
        let s = scanner_for_languages(&["es"]);
        let m = s.scan("hoy", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
    }

    #[test]
    fn es_ayer() {
        let s = scanner_for_languages(&["es"]);
        let m = s.scan("ayer", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
        let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 0, 0, 0).unwrap();
        let expected_end = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap();
        assert_eq!(
            m[0].resolved,
            ResolvedTime::Range {
                start: expected_start,
                end: expected_end,
            }
        );
    }

    #[test]
    fn es_manana() {
        let s = scanner_for_languages(&["es"]);
        let m = s.scan("ma\u{f1}ana", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
    }

    #[test]
    fn es_manana_ascii() {
        let s = scanner_for_languages(&["es"]);
        let m = s.scan("manana", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::RelativeDay);
    }

    #[test]
    fn es_hace_2_dias() {
        let s = scanner_for_languages(&["es"]);
        let m = s.scan("hace 2 d\u{ed}as", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::RelativeDayOffset);
        let expected_start = chrono::Utc.with_ymd_and_hms(2026, 2, 5, 0, 0, 0).unwrap();
        assert_eq!(
            m[0].resolved,
            ResolvedTime::Range {
                start: expected_start,
                end: expected_start + chrono::Duration::days(1),
            }
        );
    }

    #[test]
    fn es_hace_2_dias_ascii() {
        let s = scanner_for_languages(&["es"]);
        let m = s.scan("hace 2 dias", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::RelativeDayOffset);
    }

    #[test]
    fn es_en_3_dias() {
        let s = scanner_for_languages(&["es"]);
        let m = s.scan("en 3 d\u{ed}as", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::RelativeDayOffset);
    }

    #[test]
    fn es_a_las_3() {
        let s = scanner_for_languages(&["es"]);
        let m = s.scan("a las 3", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::TimeSpecification);
        let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 3, 0, 0).unwrap();
        assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
    }

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

    #[test]
    fn es_la_ultima_hora_ascii() {
        let s = scanner_for_languages(&["es"]);
        let m = s.scan("la ultima hora", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::TimeRange);
    }

    #[test]
    fn es_entre_las_9_y_las_12() {
        let s = scanner_for_languages(&["es"]);
        let m = s.scan("entre las 9 y las 12", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::TimeRange);
        let start = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 9, 0, 0).unwrap();
        let end = chrono::Utc.with_ymd_and_hms(2026, 2, 7, 12, 0, 0).unwrap();
        assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
    }

    #[test]
    fn es_ayer_a_las_3() {
        let s = scanner_for_languages(&["es"]);
        let m = s.scan("ayer a las 3", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::Combined);
        let expected = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 3, 0, 0).unwrap();
        assert_eq!(m[0].resolved, ResolvedTime::Point(expected));
    }

    #[test]
    fn es_ayer_entre_las_9_y_las_12() {
        let s = scanner_for_languages(&["es"]);
        let m = s.scan("ayer entre las 9 y las 12", now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::Combined);
        let start = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 9, 0, 0).unwrap();
        let end = chrono::Utc.with_ymd_and_hms(2026, 2, 6, 12, 0, 0).unwrap();
        assert_eq!(m[0].resolved, ResolvedTime::Range { start, end });
    }

    #[test]
    fn es_embedded_in_sentence() {
        let s = scanner_for_languages(&["es"]);
        let text = "La \u{fa}ltima hora estuve trabajando en el proyecto";
        let m = s.scan(text, now());
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].kind, ExpressionKind::TimeRange);
    }

    // ──────────────────────────────────────────────
    //  Incremental / partial match tests
    // ──────────────────────────────────────────────

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

    // ──────────────────────────────────────────────
    //  Multi-language / default scanner tests
    // ──────────────────────────────────────────────

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

    // ──────────────────────────────────────────────
    //  Edge case tests
    // ──────────────────────────────────────────────

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
}
