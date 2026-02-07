use chrono::{DateTime, Utc};
use std::ops::Range;

/// A byte-offset span in the input text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn as_range(&self) -> Range<usize> {
        self.start..self.end
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn overlaps(&self, other: &Span) -> bool {
        self.start < other.end && other.start < self.end
    }
}

/// Resolved concrete time from a parsed expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedTime {
    /// A single point in time (e.g., "yesterday at 3pm").
    Point(DateTime<Utc>),
    /// A time range with inclusive start and exclusive end.
    Range {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
}

/// A complete match: text span + resolved time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeMatch {
    pub span: Span,
    pub confidence: MatchConfidence,
    pub resolved: ResolvedTime,
    pub kind: ExpressionKind,
}

/// Confidence level of a match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchConfidence {
    /// The input is a prefix of a valid time expression.
    Partial,
    /// Fully matches a time expression.
    Complete,
}

/// Categories of time expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionKind {
    /// "today", "tomorrow", "yesterday"
    RelativeDay,
    /// "in 4 days", "two days ago"
    RelativeDayOffset,
    /// "at 3pm", "13 o'clock", "um 15 Uhr"
    TimeSpecification,
    /// "the last hour", "between 9 and 12"
    TimeRange,
    /// "yesterday at 3pm", "tomorrow between 9 and 12"
    Combined,
}

/// Parser configuration.
#[derive(Debug, Clone)]
pub struct ParserConfig {
    pub report_partial: bool,
    pub max_matches: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            report_partial: true,
            max_matches: 10,
        }
    }
}
