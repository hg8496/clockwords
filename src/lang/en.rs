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
    regex_set: regex::RegexSet,
}

impl Default for English {
    fn default() -> Self {
        Self::new()
    }
}

impl English {
    pub fn new() -> Self {
        let rules = build_rules();
        let regex_set = regex::RegexSet::new(rules.iter().map(|r| r.pattern.as_str())).unwrap();
        Self { rules, regex_set }
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

/// Parse hour with optional :MM and optional am/pm/o'clock from captures.
/// Handles both colon form (H:MM with optional am/pm in "ampm" group)
/// and whole-hour form (H with suffix in "sfx" group).
fn parse_hm_ampm(caps: &regex::Captures) -> Option<(u32, u32)> {
    let hour = caps.name("hour")?.as_str().parse::<u32>().ok()?;
    let min = caps
        .name("min")
        .and_then(|m| m.as_str().parse::<u32>().ok())
        .unwrap_or(0);
    if min > 59 {
        return None;
    }
    let ampm = caps.name("ampm").or(caps.name("sfx"));
    let h = match ampm {
        Some(ap) => resolve_hour(hour, ap.as_str())?,
        None => {
            if hour > 23 {
                return None;
            }
            hour
        }
    };
    Some((h, min))
}

/// Parse a HH:MM–HH:MM range from captures with groups `fh`, `fm`, `th`, `tm`.
fn parse_hm_range(caps: &regex::Captures) -> Option<(u32, u32, u32, u32)> {
    let fh = caps.name("fh")?.as_str().parse::<u32>().ok()?;
    let fm = caps.name("fm")?.as_str().parse::<u32>().ok()?;
    let th = caps.name("th")?.as_str().parse::<u32>().ok()?;
    let tm = caps.name("tm")?.as_str().parse::<u32>().ok()?;
    if fh > 23 || fm > 59 || th > 23 || tm > 59 {
        return None;
    }
    Some((fh, fm, th, tm))
}

/// Parse hour and optional :MM minute from captures (24h format, no am/pm).
fn parse_hm(caps: &regex::Captures) -> Option<(u32, u32)> {
    let h = caps.name("hour")?.as_str().parse::<u32>().ok()?;
    let m = caps
        .name("min")
        .and_then(|m| m.as_str().parse::<u32>().ok())
        .unwrap_or(0);
    if h > 23 || m > 59 {
        return None;
    }
    Some((h, m))
}

fn build_rules() -> Vec<GrammarRule> {
    // Number pattern for inline use
    let num = NUM_WORD_PATTERN;
    let wd = WEEKDAY_PAT;

    vec![
        // ============================================================
        //  Combined: Weekday + time spec
        //  "last Friday at 3:30pm", "next Monday at 15:30", "last Friday at 3pm"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?P<dir>next|last|this)\s+(?P<wd>{wd})\s+at\s+(?P<hour>\d{{1,2}})(?::(?P<min>\d{{2}})(?:\s*(?P<ampm>am|pm))?|\s*(?P<sfx>am|pm|o'?clock))\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let direction = weekday_direction(caps.name("dir")?.as_str())?;
                let weekday = parse_weekday(caps.name("wd")?.as_str())?;
                let (h, m) = parse_hm_ampm(caps)?;
                let date = resolve::resolve_weekday_date(weekday, direction, now, tz)?;
                resolve::resolve_time_on_date(date, h, m, tz)
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
        //  Combined: Weekday + HH:MM to/- HH:MM
        //  "next Monday 9:00 to 11:30", "last Friday 8:30 - 17:00"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?P<dir>next|last|this)\s+(?P<wd>{wd})\s+(?:from\s+)?(?P<fh>\d{{1,2}}):(?P<fm>\d{{2}})\s*(?:to\b|-)\s*(?P<th>\d{{1,2}}):(?P<tm>\d{{2}})\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let direction = weekday_direction(caps.name("dir")?.as_str())?;
                let weekday = parse_weekday(caps.name("wd")?.as_str())?;
                let (fh, fm, th, tm) = parse_hm_range(caps)?;
                let date = resolve::resolve_weekday_date(weekday, direction, now, tz)?;
                resolve::resolve_time_range_with_minutes_on_date(date, fh, fm, th, tm, tz)
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
        //  "yesterday at 3:30pm", "tomorrow at 15:30", "yesterday at 3pm"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?P<day>today|tomorrow|yesterday)\s+at\s+(?P<hour>\d{1,2})(?::(?P<min>\d{2})(?:\s*(?P<ampm>am|pm))?|\s*(?P<sfx>am|pm|o'?clock))\b"
            )
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                let (h, m) = parse_hm_ampm(caps)?;
                let date = resolve::resolve_day_offset(offset, now, tz)?;
                resolve::resolve_time_on_date(date, h, m, tz)
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
        //  Combined: relative day + HH:MM to/- HH:MM
        //  "today 8:30 to 9:30", "tomorrow 9:00 - 17:00", "yesterday from 10:15 to 11:45"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?P<day>today|tomorrow|yesterday)\s+(?:from\s+)?(?P<fh>\d{1,2}):(?P<fm>\d{2})\s*(?:to\b|-)\s*(?P<th>\d{1,2}):(?P<tm>\d{2})\b"
            )
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                let (fh, fm, th, tm) = parse_hm_range(caps)?;
                let date = resolve::resolve_day_offset(offset, now, tz)?;
                resolve::resolve_time_range_with_minutes_on_date(date, fh, fm, th, tm, tz)
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
        // --- Time spec with suffix: "at 3:30pm", "11:30am", "at 3pm", "3 o'clock" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?:at\s+)?(?P<hour>\d{1,2})(?::(?P<min>\d{2}))?\s*(?P<ampm>am|pm|o'?clock)\b"
            )
            .unwrap(),
            kind: ExpressionKind::TimeSpecification,
            resolver: |caps, now, tz| {
                let (h, m) = parse_hm_ampm(caps)?;
                resolve::resolve_time_today(h, m, now, tz)
            },
        },
        // --- Time spec bare colon: "at 15:30", "at 9:00" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\bat\s+(?P<hour>\d{1,2}):(?P<min>\d{2})\b"
            )
            .unwrap(),
            kind: ExpressionKind::TimeSpecification,
            resolver: |caps, now, tz| {
                let (h, m) = parse_hm(caps)?;
                resolve::resolve_time_today(h, m, now, tz)
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
        // --- Time range: "from 8:30 to 9:30", "from 10:00 - 11:30" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\bfrom\s+(?P<fh>\d{1,2}):(?P<fm>\d{2})\s*(?:to\b|-)\s*(?P<th>\d{1,2}):(?P<tm>\d{2})\b"
            )
            .unwrap(),
            kind: ExpressionKind::TimeRange,
            resolver: |caps, now, tz| {
                let (fh, fm, th, tm) = parse_hm_range(caps)?;
                resolve::resolve_time_range_with_minutes_today(fh, fm, th, tm, now, tz)
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
        apply_rules(&self.rules, &self.regex_set, text, now, tz)
    }
}
