use aho_corasick::AhoCorasick;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;

use crate::lang::LanguageParser;
use crate::types::*;

/// The main parser combining multiple language parsers with an Aho-Corasick prefilter.
pub struct TimeExpressionScanner {
    languages: Vec<Box<dyn LanguageParser>>,
    keyword_filter: AhoCorasick,
    prefix_filter: AhoCorasick,
    config: ParserConfig,
}

impl TimeExpressionScanner {
    pub fn new(languages: Vec<Box<dyn LanguageParser>>, config: ParserConfig) -> Self {
        let all_keywords: Vec<&str> = languages
            .iter()
            .flat_map(|lang| lang.keywords().iter().copied())
            .collect();

        let all_prefixes: Vec<&str> = languages
            .iter()
            .flat_map(|lang| lang.keyword_prefixes().iter().copied())
            .collect();

        let keyword_filter = AhoCorasick::builder()
            .ascii_case_insensitive(true)
            .build(&all_keywords)
            .expect("Failed to build keyword automaton");

        let prefix_filter = AhoCorasick::builder()
            .ascii_case_insensitive(true)
            .build(&all_prefixes)
            .expect("Failed to build prefix automaton");

        Self {
            languages,
            keyword_filter,
            prefix_filter,
            config,
        }
    }

    /// Scan the input text and return all time expression matches.
    ///
    /// Uses the timezone configured in [`ParserConfig::timezone`] (defaults to UTC).
    /// Call this on every keystroke with the current buffer contents.
    pub fn scan(&self, text: &str, now: DateTime<Utc>) -> Vec<TimeMatch> {
        self.scan_with_tz(text, now, self.config.timezone)
    }

    /// Scan the input text using an explicit timezone override.
    ///
    /// Times entered by the user are interpreted in the given timezone.
    /// The resolved output remains in UTC.
    pub fn scan_with_tz(&self, text: &str, now: DateTime<Utc>, tz: Tz) -> Vec<TimeMatch> {
        let has_keywords = self.keyword_filter.find(text).is_some();
        let has_prefixes = self.config.report_partial && self.prefix_filter.find(text).is_some();

        if !has_keywords && !has_prefixes {
            return Vec::new();
        }

        let mut matches = Vec::new();

        if has_keywords {
            for lang in &self.languages {
                matches.extend(lang.parse(text, now, tz));
            }
        }

        if has_prefixes && self.config.report_partial {
            self.find_partial_matches(text, now, &mut matches);
        }

        matches.sort_by(|a, b| {
            a.span
                .start
                .cmp(&b.span.start)
                .then(b.confidence.cmp(&a.confidence))
                .then(b.span.len().cmp(&a.span.len()))
        });

        self.deduplicate(matches)
    }

    fn find_partial_matches(&self, text: &str, _now: DateTime<Utc>, matches: &mut Vec<TimeMatch>) {
        // Only check if the text ends with a prefix of a time keyword.
        // This detects the user currently typing a time expression.
        for lang in &self.languages {
            for prefix in lang.keyword_prefixes() {
                if text.to_lowercase().ends_with(&prefix.to_lowercase()) {
                    let start = text.len() - prefix.len();
                    // Check that this prefix starts at a word boundary
                    if start > 0 {
                        let prev = text.as_bytes()[start - 1];
                        if prev != b' ' && prev != b'\t' && prev != b'\n' {
                            continue;
                        }
                    }
                    // Don't add partial if a complete match already covers this span
                    let already_matched = matches.iter().any(|m| {
                        m.confidence == MatchConfidence::Complete
                            && m.span.start <= start
                            && m.span.end >= text.len()
                    });
                    if !already_matched {
                        matches.push(TimeMatch {
                            span: Span::new(start, text.len()),
                            confidence: MatchConfidence::Partial,
                            resolved: ResolvedTime::Point(chrono::Utc::now()),
                            kind: ExpressionKind::RelativeDay,
                        });
                        return; // Only report one partial match
                    }
                }
            }
        }
    }

    fn deduplicate(&self, matches: Vec<TimeMatch>) -> Vec<TimeMatch> {
        if matches.is_empty() {
            return matches;
        }

        let mut result: Vec<TimeMatch> = Vec::new();

        for m in matches {
            let dominated = result.iter().any(|existing| {
                existing.span.overlaps(&m.span)
                    && (existing.confidence > m.confidence
                        || (existing.confidence == m.confidence
                            && existing.span.len() >= m.span.len()))
            });

            if !dominated {
                // Remove any existing matches that this new one dominates
                result.retain(|existing| {
                    !(m.span.overlaps(&existing.span)
                        && (m.confidence > existing.confidence
                            || (m.confidence == existing.confidence
                                && m.span.len() > existing.span.len())))
                });
                result.push(m);
            }
        }

        // Limit results
        result.truncate(self.config.max_matches);
        result
    }
}
