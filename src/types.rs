use chrono::{DateTime, Utc};
use std::ops::Range;

/// A byte-offset span identifying a substring within the input text.
///
/// Offsets are measured in bytes (not characters), matching Rust's `str` indexing.
/// You can use [`Span::as_range`] to slice the original input:
///
/// ```
/// # use clockwords::Span;
/// let text = "The last hour I coded";
/// let span = Span::new(0, 13);
/// assert_eq!(&text[span.as_range()], "The last hour");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// Inclusive start byte offset.
    pub start: usize,
    /// Exclusive end byte offset.
    pub end: usize,
}

impl Span {
    /// Create a new span from inclusive start to exclusive end.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Convert to a `std::ops::Range<usize>` for use with slice indexing.
    pub fn as_range(&self) -> Range<usize> {
        self.start..self.end
    }

    /// Length of the span in bytes.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns `true` if the span covers zero bytes.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Returns `true` if this span and `other` share at least one byte position.
    pub fn overlaps(&self, other: &Span) -> bool {
        self.start < other.end && other.start < self.end
    }
}

/// The resolved concrete time derived from a parsed time expression.
///
/// Every matched expression resolves to either a single point in time
/// or a time range with an inclusive start and exclusive end.
///
/// # Examples
///
/// - `"yesterday at 3pm"` resolves to `ResolvedTime::Point(2026-02-06T15:00:00Z)`
/// - `"the last hour"` resolves to `ResolvedTime::Range { start: now - 1h, end: now }`
/// - `"today"` resolves to `ResolvedTime::Range { start: 00:00, end: 00:00+1d }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedTime {
    /// A single point in time.
    ///
    /// Produced by expressions like `"yesterday at 3pm"`, `"um 15 Uhr"`,
    /// `"à 13h"`, or `"a las 3"`.
    Point(DateTime<Utc>),

    /// A time range with inclusive start and exclusive end.
    ///
    /// Produced by expressions like `"today"` (full day), `"the last hour"`,
    /// `"between 9 and 12"`, or `"von 9 bis 12 Uhr"`.
    Range {
        /// Inclusive start of the time range.
        start: DateTime<Utc>,
        /// Exclusive end of the time range.
        end: DateTime<Utc>,
    },
}

/// A complete match result: the text span where the time expression was found,
/// how confident the parser is, what it resolved to, and what kind of expression
/// it was.
///
/// This is the primary output type returned by
/// [`TimeExpressionScanner::scan`](crate::scanner::TimeExpressionScanner::scan).
///
/// # GUI integration
///
/// Use [`span`](TimeMatch::span) to highlight the matched region in the input field.
/// Use [`resolved`](TimeMatch::resolved) to obtain the concrete `DateTime` values.
/// Use [`confidence`](TimeMatch::confidence) to distinguish between complete matches
/// and partial matches (the user is still typing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeMatch {
    /// The byte range in the original input text that was matched.
    pub span: Span,

    /// Whether the match is complete or the user is still typing a prefix.
    ///
    /// A GUI can use this to style complete matches differently from partial ones
    /// (e.g., solid underline vs. dotted underline).
    pub confidence: MatchConfidence,

    /// The resolved concrete time for this match.
    ///
    /// For [`Partial`](MatchConfidence::Partial) matches this is a placeholder
    /// and should not be used for time calculations.
    pub resolved: ResolvedTime,

    /// The category of time expression that was matched.
    pub kind: ExpressionKind,
}

/// Confidence level of a match, indicating whether the parser has seen a
/// complete time expression or just a prefix being typed.
///
/// The ordering is `Partial < Complete`, which is used during deduplication
/// to prefer complete matches over partial ones on the same span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchConfidence {
    /// The input ends with a prefix of a known time keyword (e.g., `"yester"`
    /// is a prefix of `"yesterday"`).
    ///
    /// The resolved time in a partial match is a placeholder and should not be
    /// used for actual time calculations. Partial matches are useful for GUI
    /// hints (e.g., dimmed underline) to indicate the user is typing a time
    /// expression.
    Partial,

    /// The expression fully matches a known time pattern and the resolved time
    /// is meaningful.
    Complete,
}

/// Categories of time expressions recognized by the parser.
///
/// This enum lets callers distinguish the structural form of a match, which
/// can be useful for UI presentation or further processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionKind {
    /// A bare relative day keyword.
    ///
    /// Examples: `"today"`, `"yesterday"`, `"morgen"`, `"demain"`, `"ayer"`.
    /// Resolves to a full-day range (midnight to midnight).
    RelativeDay,

    /// A relative day offset with a numeric component.
    ///
    /// Examples: `"in 4 days"`, `"two days ago"`, `"vor 3 Tagen"`,
    /// `"il y a 3 jours"`, `"hace 2 días"`.
    /// Resolves to a full-day range.
    RelativeDayOffset,

    /// A specific time of day (on the current date unless combined).
    ///
    /// Examples: `"at 3pm"`, `"13 o'clock"`, `"um 15 Uhr"`, `"à 13h"`,
    /// `"a las 3"`.
    /// Resolves to a single point in time.
    TimeSpecification,

    /// A time range expression.
    ///
    /// Examples: `"the last hour"`, `"between 9 and 12"`,
    /// `"von 9 bis 12 Uhr"`, `"la dernière heure"`, `"la última hora"`.
    /// Resolves to a range with start and end.
    TimeRange,

    /// A combined expression pairing a relative day with a time specification
    /// or time range.
    ///
    /// Examples: `"yesterday at 3pm"`, `"gestern um 15 Uhr"`,
    /// `"tomorrow between 9 and 12"`, `"hier à 13h"`, `"ayer a las 3"`.
    /// Resolves to either a point or range on the specified day.
    Combined,
}

/// Configuration for the [`TimeExpressionScanner`](crate::scanner::TimeExpressionScanner).
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Whether to report partial (prefix) matches while the user is typing.
    ///
    /// When `true`, the scanner returns [`MatchConfidence::Partial`] results
    /// for incomplete keywords (e.g., `"yester"` before the user finishes
    /// typing `"yesterday"`). Defaults to `true`.
    pub report_partial: bool,

    /// Maximum number of matches to return per `scan()` call.
    ///
    /// Excess matches are dropped after deduplication and sorting.
    /// Defaults to `10`.
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
