use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use regex::Regex;

use crate::lang::numbers::parse_number_en;
use crate::lang::{GrammarRule, LanguageParser, apply_rules};
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
    "from",
    "at",
    "in",
    "day",
    "days",
    "minute",
    "minutes",
    "next",
    "this",
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];

const PREFIXES: &[&str] = &[
    "tod", "toda", "tom", "tomo", "tomor", "tomorr", "tomorro", "yes", "yest", "yeste", "yester",
    "yesterd", "yesterda", "bet", "betw", "betwe", "betwee", "mon", "mond", "monda", "tue", "tues",
    "tuesd", "tuesda", "wed", "wedn", "wedne", "wednes", "wednesd", "wednesda", "thu", "thur",
    "thurs", "thursd", "thursda", "fri", "frid", "frida", "sat", "satu", "satur", "saturd",
    "saturda", "sun", "sund", "sunda",
];

const NUM_WORD_PATTERN: &str = r"(?:\d+|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|thirty)";

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

fn parse_weekday(s: &str) -> Option<chrono::Weekday> {
    match s.to_lowercase().as_str() {
        "monday" => Some(chrono::Weekday::Mon),
        "tuesday" => Some(chrono::Weekday::Tue),
        "wednesday" => Some(chrono::Weekday::Wed),
        "thursday" => Some(chrono::Weekday::Thu),
        "friday" => Some(chrono::Weekday::Fri),
        "saturday" => Some(chrono::Weekday::Sat),
        "sunday" => Some(chrono::Weekday::Sun),
        _ => None,
    }
}

fn parse_num(s: &str) -> Option<u32> {
    s.parse::<u32>()
        .ok()
        .or_else(|| parse_number_en(&s.to_lowercase()))
}

/// Shared day pattern for weekdays
const WEEKDAY_PAT: &str = r"monday|tuesday|wednesday|thursday|friday|saturday|sunday";

/// Resolve a weekday direction string to -1, 0, or 1
fn weekday_direction(s: &str) -> Option<i64> {
    match s.to_lowercase().as_str() {
        "next" => Some(1),
        "last" => Some(-1),
        "this" => Some(0),
        _ => None,
    }
}

/// Resolve hour+ampm to 24h, handling am/pm/o'clock
fn resolve_hour(hour: u32, ampm: &str) -> Option<u32> {
    let h = if ampm.to_lowercase().starts_with("o") {
        hour
    } else {
        resolve::to_24h(hour, ampm)
    };
    if h > 23 { None } else { Some(h) }
}

fn build_rules() -> Vec<GrammarRule> {
    // Number pattern for inline use
    let num = NUM_WORD_PATTERN;
    let wd = WEEKDAY_PAT;

    vec![
        // ============================================================
        //  Combined: Weekday + time spec
        //  "last Friday at 3pm", "next Monday at 13 o'clock"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?P<dir>next|last|this)\s+(?P<wd>{wd})\s+at\s+(?P<hour>\d{{1,2}})\s*(?P<ampm>am|pm|o'?clock)\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let direction = weekday_direction(caps.name("dir")?.as_str())?;
                let weekday = parse_weekday(caps.name("wd")?.as_str())?;
                let hour = caps.name("hour")?.as_str().parse::<u32>().ok()?;
                let h = resolve_hour(hour, caps.name("ampm")?.as_str())?;
                let date = resolve::resolve_weekday_date(weekday, direction, now, tz)?;
                resolve::resolve_time_on_date(date, h, 0, tz)
            },
        },
        // ============================================================
        //  Combined: Weekday + between range
        //  "last Friday between 9 and 12"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?P<dir>next|last|this)\s+(?P<wd>{wd})\s+between\s+(?P<from>{num})\s+and\s+(?P<to>{num})\s*(?:o'?clock)?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let direction = weekday_direction(caps.name("dir")?.as_str())?;
                let weekday = parse_weekday(caps.name("wd")?.as_str())?;
                let from = parse_num(caps.name("from")?.as_str())?;
                let to = parse_num(caps.name("to")?.as_str())?;
                if from > 23 || to > 23 { return None; }
                let date = resolve::resolve_weekday_date(weekday, direction, now, tz)?;
                resolve::resolve_time_range_on_date(date, from, to, tz)
            },
        },
        // ============================================================
        //  Combined: Weekday + from/to range
        //  "last Friday from 9 to eleven", "next Monday from 9 to 5"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?P<dir>next|last|this)\s+(?P<wd>{wd})\s+from\s+(?P<from>{num})\s+to\s+(?P<to>{num})\s*(?:o'?clock)?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let direction = weekday_direction(caps.name("dir")?.as_str())?;
                let weekday = parse_weekday(caps.name("wd")?.as_str())?;
                let from = parse_num(caps.name("from")?.as_str())?;
                let to = parse_num(caps.name("to")?.as_str())?;
                if from > 23 || to > 23 { return None; }
                let date = resolve::resolve_weekday_date(weekday, direction, now, tz)?;
                resolve::resolve_time_range_on_date(date, from, to, tz)
            },
        },
        // ============================================================
        //  Combined: relative day + at time
        //  "yesterday at 3pm", "tomorrow at 13 o'clock"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?P<day>today|tomorrow|yesterday)\s+at\s+(?P<hour>\d{1,2})\s*(?P<ampm>am|pm|o'?clock)\b"
            )
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                let hour = caps.name("hour")?.as_str().parse::<u32>().ok()?;
                let h = resolve_hour(hour, caps.name("ampm")?.as_str())?;
                let date = resolve::resolve_day_offset(offset, now, tz)?;
                resolve::resolve_time_on_date(date, h, 0, tz)
            },
        },
        // ============================================================
        //  Combined: relative day + between range
        //  "yesterday between 9 and 12 (o'clock)"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?P<day>today|tomorrow|yesterday)\s+between\s+(?P<from>{num})\s+and\s+(?P<to>{num})\s*(?:o'?clock)?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                let from = parse_num(caps.name("from")?.as_str())?;
                let to = parse_num(caps.name("to")?.as_str())?;
                if from > 23 || to > 23 { return None; }
                let date = resolve::resolve_day_offset(offset, now, tz)?;
                resolve::resolve_time_range_on_date(date, from, to, tz)
            },
        },
        // ============================================================
        //  Combined: relative day + from/to range
        //  "yesterday from 9 to 11", "tomorrow from nine to five"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?P<day>today|tomorrow|yesterday)\s+from\s+(?P<from>{num})\s+to\s+(?P<to>{num})\s*(?:o'?clock)?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                let from = parse_num(caps.name("from")?.as_str())?;
                let to = parse_num(caps.name("to")?.as_str())?;
                if from > 23 || to > 23 { return None; }
                let date = resolve::resolve_day_offset(offset, now, tz)?;
                resolve::resolve_time_range_on_date(date, from, to, tz)
            },
        },
        // --- Relative days ---
        GrammarRule {
            pattern: Regex::new(r"(?i)\b(?P<day>today|tomorrow|yesterday)\b").unwrap(),
            kind: ExpressionKind::RelativeDay,
            resolver: |caps, now, tz| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                resolve::resolve_relative_day(offset, now, tz)
            },
        },
        // --- Day offset: "in 4 days" ---
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\bin\s+(?P<num>{num})\s+days?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::RelativeDayOffset,
            resolver: |caps, now, tz| {
                let n = parse_num(caps.name("num")?.as_str())?;
                resolve::resolve_relative_day(n as i64, now, tz)
            },
        },
        // --- Day offset: "two days ago" ---
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?P<num>{num})\s+days?\s+ago\b"
            ))
            .unwrap(),
            kind: ExpressionKind::RelativeDayOffset,
            resolver: |caps, now, tz| {
                let n = parse_num(caps.name("num")?.as_str())?;
                resolve::resolve_relative_day(-(n as i64), now, tz)
            },
        },
        // --- Time spec: "at 3pm", "at 3 am", "13 o'clock" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?:at\s+)?(?P<hour>\d{1,2})\s*(?P<ampm>am|pm|o'?clock)\b"
            )
            .unwrap(),
            kind: ExpressionKind::TimeSpecification,
            resolver: |caps, now, tz| {
                let hour = caps.name("hour")?.as_str().parse::<u32>().ok()?;
                let h = resolve_hour(hour, caps.name("ampm")?.as_str())?;
                resolve::resolve_time_today(h, 0, now, tz)
            },
        },
        // --- Time range: "the last hour/minute" ---
        GrammarRule {
            pattern: Regex::new(r"(?i)\b(?:the\s+)?last\s+(?P<unit>hour|minute)\b").unwrap(),
            kind: ExpressionKind::TimeRange,
            resolver: |caps, now, _tz| {
                let unit = caps.name("unit")?.as_str().to_lowercase();
                resolve::resolve_last_duration(&unit, now)
            },
        },
        // --- Time range: "between 9 and 12 (o'clock)" ---
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\bbetween\s+(?P<from>{num})\s+and\s+(?P<to>{num})\s*(?:o'?clock)?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::TimeRange,
            resolver: |caps, now, tz| {
                let from = parse_num(caps.name("from")?.as_str())?;
                let to = parse_num(caps.name("to")?.as_str())?;
                if from > 23 || to > 23 { return None; }
                resolve::resolve_time_range_today(from, to, now, tz)
            },
        },
        // --- Time range: "from 9 to 12 (o'clock)" ---
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\bfrom\s+(?P<from>{num})\s+to\s+(?P<to>{num})\s*(?:o'?clock)?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::TimeRange,
            resolver: |caps, now, tz| {
                let from = parse_num(caps.name("from")?.as_str())?;
                let to = parse_num(caps.name("to")?.as_str())?;
                if from > 23 || to > 23 { return None; }
                resolve::resolve_time_range_today(from, to, now, tz)
            },
        },
        // --- Next/Last/This Weekday ---
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?P<dir>next|last|this)\s+(?P<day>{wd})\b"
            ))
            .unwrap(),
            kind: ExpressionKind::RelativeDay,
            resolver: |caps, now, tz| {
                let direction = weekday_direction(caps.name("dir")?.as_str())?;
                let weekday = parse_weekday(caps.name("day")?.as_str())?;
                resolve::resolve_weekday(weekday, direction, now, tz)
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

    fn parse(&self, text: &str, now: DateTime<Utc>, tz: Tz) -> Vec<TimeMatch> {
        apply_rules(&self.rules, text, now, tz)
    }
}
