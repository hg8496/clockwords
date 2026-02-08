pub mod de;
pub mod en;
pub mod es;
pub mod fr;
pub mod numbers;

use crate::types::{ExpressionKind, ResolvedTime, TimeMatch};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use regex::Regex;

/// A grammar rule: compiled regex + metadata + resolver function.
pub struct GrammarRule {
    pub pattern: Regex,
    pub kind: ExpressionKind,
    pub resolver: fn(captures: &regex::Captures, now: DateTime<Utc>, tz: Tz) -> Option<ResolvedTime>,
}

/// Trait that each language must implement.
pub trait LanguageParser: Send + Sync {
    fn lang_id(&self) -> &'static str;

    /// Keywords for the Aho-Corasick prefilter.
    fn keywords(&self) -> &[&str];

    /// Keyword prefixes (length >= 3) for partial match detection.
    fn keyword_prefixes(&self) -> &[&str];

    /// Parse all time expressions from the text.
    fn parse(&self, text: &str, now: DateTime<Utc>, tz: Tz) -> Vec<TimeMatch>;
}

/// Shared helper: run all grammar rules against text and collect matches.
pub fn apply_rules(rules: &[GrammarRule], text: &str, now: DateTime<Utc>, tz: Tz) -> Vec<TimeMatch> {
    use crate::types::{MatchConfidence, Span};

    let mut matches = Vec::new();
    let mut covered: Vec<std::ops::Range<usize>> = Vec::new();

    for rule in rules {
        for caps in rule.pattern.captures_iter(text) {
            let m = caps.get(0).unwrap();
            let range = m.start()..m.end();

            // Skip if this range is already covered by a longer match
            if covered
                .iter()
                .any(|c| c.start <= range.start && c.end >= range.end)
            {
                continue;
            }

            if let Some(resolved) = (rule.resolver)(&caps, now, tz) {
                // Remove any shorter matches that this one covers
                let new_range = range.clone();
                matches.retain(|tm: &TimeMatch| {
                    let s = tm.span.start..tm.span.end;
                    !(new_range.start <= s.start && new_range.end >= s.end)
                });
                covered.retain(|c| !(new_range.start <= c.start && new_range.end >= c.end));

                matches.push(TimeMatch {
                    span: Span::new(range.start, range.end),
                    confidence: MatchConfidence::Complete,
                    resolved,
                    kind: rule.kind,
                });
                covered.push(range);
            }
        }
    }
    matches
}
