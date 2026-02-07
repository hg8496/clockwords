use chrono::{DateTime, Utc};
use regex::Regex;

use crate::lang::numbers::parse_number_en;
use crate::lang::{apply_rules, GrammarRule, LanguageParser};
use crate::resolve;
use crate::types::*;

const KEYWORDS: &[&str] = &[
    "today",
    "tomorrow",
    "yesterday",
    "ago",
    "last",
    "hour",
    "hours",
    "o'clock",
    "oclock",
    "am",
    "pm",
    "between",
    "at",
    "in",
    "day",
    "days",
    "minute",
    "minutes",
];

const PREFIXES: &[&str] = &[
    "tod", "toda", "tom", "tomo", "tomor", "tomorr", "tomorro",
    "yes", "yest", "yeste", "yester", "yesterd", "yesterda",
    "bet", "betw", "betwe", "betwee",
];

const NUM_WORD_PATTERN: &str =
    r"(?:\d+|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|thirty)";

pub struct English {
    rules: Vec<GrammarRule>,
}

impl Default for English {
    fn default() -> Self {
        Self::new()
    }
}

impl English {
    pub fn new() -> Self {
        Self {
            rules: build_rules(),
        }
    }
}

fn day_keyword_offset(s: &str) -> Option<i64> {
    match s.to_lowercase().as_str() {
        "today" => Some(0),
        "tomorrow" => Some(1),
        "yesterday" => Some(-1),
        _ => None,
    }
}

fn parse_num(s: &str) -> Option<u32> {
    s.parse::<u32>().ok().or_else(|| parse_number_en(&s.to_lowercase()))
}

fn build_rules() -> Vec<GrammarRule> {
    // Number pattern for inline use
    let num = NUM_WORD_PATTERN;

    vec![
        // --- Combined: "yesterday at 3pm" / "tomorrow at 13 o'clock" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?P<day>today|tomorrow|yesterday)\s+at\s+(?P<hour>\d{1,2})\s*(?P<ampm>am|pm|o'?clock)\b"
            )
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                let hour = caps.name("hour")?.as_str().parse::<u32>().ok()?;
                let ampm = caps.name("ampm")?.as_str();
                let h = if ampm.to_lowercase().starts_with("o") {
                    hour
                } else {
                    resolve::to_24h(hour, ampm)
                };
                if h > 23 { return None; }
                let date = resolve::resolve_day_offset(offset, now);
                Some(resolve::resolve_time_on_date(date, h, 0))
            },
        },
        // --- Combined: "yesterday between 9 and 12 (o'clock)" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?P<day>today|tomorrow|yesterday)\s+between\s+(?P<from>\d{1,2})\s+and\s+(?P<to>\d{1,2})\s*(?:o'?clock)?\b"
            )
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                let from = caps.name("from")?.as_str().parse::<u32>().ok()?;
                let to = caps.name("to")?.as_str().parse::<u32>().ok()?;
                if from > 23 || to > 23 { return None; }
                let date = resolve::resolve_day_offset(offset, now);
                Some(resolve::resolve_time_range_on_date(date, from, to))
            },
        },
        // --- Relative days ---
        GrammarRule {
            pattern: Regex::new(r"(?i)\b(?P<day>today|tomorrow|yesterday)\b").unwrap(),
            kind: ExpressionKind::RelativeDay,
            resolver: |caps, now| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                Some(resolve::resolve_relative_day(offset, now))
            },
        },
        // --- Day offset: "in 4 days" ---
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\bin\s+(?P<num>{num})\s+days?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::RelativeDayOffset,
            resolver: |caps, now| {
                let n = parse_num(caps.name("num")?.as_str())?;
                Some(resolve::resolve_relative_day(n as i64, now))
            },
        },
        // --- Day offset: "two days ago" ---
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?P<num>{num})\s+days?\s+ago\b"
            ))
            .unwrap(),
            kind: ExpressionKind::RelativeDayOffset,
            resolver: |caps, now| {
                let n = parse_num(caps.name("num")?.as_str())?;
                Some(resolve::resolve_relative_day(-(n as i64), now))
            },
        },
        // --- Time spec: "at 3pm", "at 3 am", "13 o'clock" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?:at\s+)?(?P<hour>\d{1,2})\s*(?P<ampm>am|pm|o'?clock)\b"
            )
            .unwrap(),
            kind: ExpressionKind::TimeSpecification,
            resolver: |caps, now| {
                let hour = caps.name("hour")?.as_str().parse::<u32>().ok()?;
                let ampm = caps.name("ampm")?.as_str();
                let h = if ampm.to_lowercase().starts_with("o") {
                    hour
                } else {
                    resolve::to_24h(hour, ampm)
                };
                if h > 23 { return None; }
                Some(resolve::resolve_time_today(h, 0, now))
            },
        },
        // --- Time range: "the last hour/minute" ---
        GrammarRule {
            pattern: Regex::new(r"(?i)\b(?:the\s+)?last\s+(?P<unit>hour|minute)\b").unwrap(),
            kind: ExpressionKind::TimeRange,
            resolver: |caps, now| {
                let unit = caps.name("unit")?.as_str().to_lowercase();
                Some(resolve::resolve_last_duration(&unit, now))
            },
        },
        // --- Time range: "between 9 and 12 (o'clock)" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\bbetween\s+(?P<from>\d{1,2})\s+and\s+(?P<to>\d{1,2})\s*(?:o'?clock)?\b"
            )
            .unwrap(),
            kind: ExpressionKind::TimeRange,
            resolver: |caps, now| {
                let from = caps.name("from")?.as_str().parse::<u32>().ok()?;
                let to = caps.name("to")?.as_str().parse::<u32>().ok()?;
                if from > 23 || to > 23 { return None; }
                Some(resolve::resolve_time_range_today(from, to, now))
            },
        },
    ]
}

impl LanguageParser for English {
    fn lang_id(&self) -> &'static str {
        "en"
    }

    fn keywords(&self) -> &[&str] {
        KEYWORDS
    }

    fn keyword_prefixes(&self) -> &[&str] {
        PREFIXES
    }

    fn parse(&self, text: &str, now: DateTime<Utc>) -> Vec<TimeMatch> {
        apply_rules(&self.rules, text, now)
    }
}
