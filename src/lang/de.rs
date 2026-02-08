use chrono::{DateTime, Utc};
use regex::Regex;

use crate::lang::numbers::parse_number_de;
use crate::lang::{GrammarRule, LanguageParser, apply_rules};
use crate::resolve;
use crate::types::*;

const KEYWORDS: &[&str] = &[
    "heute",
    "morgen",
    "gestern",
    "vor",
    "tagen",
    "tag",
    "uhr",
    "um",
    "zwischen",
    "bis",
    "von",
    "letzte",
    "letzten",
    "stunde",
    "stunden",
    "minute",
    "minuten",
    "nächsten",
    "naechsten",
    "kommenden",
    "letzten",
    "vergangenen",
    "diesen",
    "montag",
    "dienstag",
    "mittwoch",
    "donnerstag",
    "freitag",
    "samstag",
    "sonntag",
    "sonnabend",
];

const PREFIXES: &[&str] = &[
    "heu",
    "heut",
    "mor",
    "morg",
    "morge",
    "ges",
    "gest",
    "geste",
    "gester",
    "zwi",
    "zwis",
    "zwisc",
    "zwisch",
    "zwische",
    "zwischen",
    "mon",
    "mont",
    "monta",
    "die",
    "dien",
    "diens",
    "dienst",
    "diensta",
    "mit",
    "mitt",
    "mittw",
    "mittwo",
    "mittwoc",
    "don",
    "donn",
    "donne",
    "donner",
    "donners",
    "donnerst",
    "donnersta",
    "fre",
    "frei",
    "freit",
    "freita",
    "sam",
    "sams",
    "samst",
    "samsta",
    "son",
    "sonn",
    "sonnt",
    "sonnta",
];

const NUM_WORD_PATTERN: &str = r"(?:\d+|ein|eins|eine|einem|einen|zwei|drei|vier|f[uü]n[f]?|sechs|sieben|acht|neun|zehn|elf|zw[oö]lf)";

fn day_keyword_offset(s: &str) -> Option<i64> {
    match s.to_lowercase().as_str() {
        "heute" => Some(0),
        "morgen" => Some(1),
        "gestern" => Some(-1),
        _ => None,
    }
}

fn parse_weekday(s: &str) -> Option<chrono::Weekday> {
    match s.to_lowercase().as_str() {
        "montag" => Some(chrono::Weekday::Mon),
        "dienstag" => Some(chrono::Weekday::Tue),
        "mittwoch" => Some(chrono::Weekday::Wed),
        "donnerstag" => Some(chrono::Weekday::Thu),
        "freitag" => Some(chrono::Weekday::Fri),
        "samstag" | "sonnabend" => Some(chrono::Weekday::Sat),
        "sonntag" => Some(chrono::Weekday::Sun),
        _ => None,
    }
}

fn parse_num(s: &str) -> Option<u32> {
    s.parse::<u32>()
        .ok()
        .or_else(|| parse_number_de(&s.to_lowercase()))
}

pub struct German {
    rules: Vec<GrammarRule>,
}

impl Default for German {
    fn default() -> Self {
        Self::new()
    }
}

impl German {
    pub fn new() -> Self {
        Self {
            rules: build_rules(),
        }
    }
}

/// Shared weekday pattern
const WEEKDAY_PAT: &str = r"montag|dienstag|mittwoch|donnerstag|freitag|samstag|sonnabend|sonntag";

fn weekday_direction(s: &str) -> Option<i64> {
    match s.to_lowercase().as_str() {
        "nächsten" | "naechsten" | "kommenden" => Some(1),
        "letzten" | "vergangenen" => Some(-1),
        "diesen" => Some(0),
        _ => None,
    }
}

fn build_rules() -> Vec<GrammarRule> {
    let num = NUM_WORD_PATTERN;
    let wd = WEEKDAY_PAT;

    vec![
        // ============================================================
        //  Combined: Weekday + "um X Uhr"
        //  "letzten Freitag um 15 Uhr"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?:am\s+)?(?P<dir>n[äae]chsten|kommenden|letzten|vergangenen|diesen)\s+(?P<wd>{wd})\s+um\s+(?P<hour>\d{{1,2}})\s+Uhr\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now| {
                let direction = weekday_direction(caps.name("dir")?.as_str())?;
                let weekday = parse_weekday(caps.name("wd")?.as_str())?;
                let hour = caps.name("hour")?.as_str().parse::<u32>().ok()?;
                if hour > 23 { return None; }
                let date = resolve::resolve_weekday_date(weekday, direction, now)?;
                resolve::resolve_time_on_date(date, hour, 0)
            },
        },
        // ============================================================
        //  Combined: Weekday + "von X bis Y Uhr"
        //  "letzten Freitag von 9 bis 12 Uhr"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?:am\s+)?(?P<dir>n[äae]chsten|kommenden|letzten|vergangenen|diesen)\s+(?P<wd>{wd})\s+von\s+(?P<from>\d{{1,2}})\s+bis\s+(?P<to>\d{{1,2}})(?:\s*Uhr)?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now| {
                let direction = weekday_direction(caps.name("dir")?.as_str())?;
                let weekday = parse_weekday(caps.name("wd")?.as_str())?;
                let from = caps.name("from")?.as_str().parse::<u32>().ok()?;
                let to = caps.name("to")?.as_str().parse::<u32>().ok()?;
                if from > 23 || to > 23 { return None; }
                let date = resolve::resolve_weekday_date(weekday, direction, now)?;
                resolve::resolve_time_range_on_date(date, from, to)
            },
        },
        // ============================================================
        //  Combined: Weekday + "zwischen X und Y Uhr"
        //  "letzten Freitag zwischen 9 und 12 Uhr"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?:am\s+)?(?P<dir>n[äae]chsten|kommenden|letzten|vergangenen|diesen)\s+(?P<wd>{wd})\s+zwischen\s+(?P<from>\d{{1,2}})\s+und\s+(?P<to>\d{{1,2}})\s*(?:Uhr)?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now| {
                let direction = weekday_direction(caps.name("dir")?.as_str())?;
                let weekday = parse_weekday(caps.name("wd")?.as_str())?;
                let from = caps.name("from")?.as_str().parse::<u32>().ok()?;
                let to = caps.name("to")?.as_str().parse::<u32>().ok()?;
                if from > 23 || to > 23 { return None; }
                let date = resolve::resolve_weekday_date(weekday, direction, now)?;
                resolve::resolve_time_range_on_date(date, from, to)
            },
        },
        // --- Combined: "gestern um 15 Uhr" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?P<day>heute|morgen|gestern)\s+um\s+(?P<hour>\d{1,2})\s+Uhr\b",
            )
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                let hour = caps.name("hour")?.as_str().parse::<u32>().ok()?;
                if hour > 23 { return None; }
                let date = resolve::resolve_day_offset(offset, now)?;
                resolve::resolve_time_on_date(date, hour, 0)
            },
        },
        // --- Combined: "gestern von 9 bis 12 Uhr" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?P<day>heute|morgen|gestern)\s+von\s+(?P<from>\d{1,2})\s+bis\s+(?P<to>\d{1,2})\s*Uhr\b",
            )
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                let from = caps.name("from")?.as_str().parse::<u32>().ok()?;
                let to = caps.name("to")?.as_str().parse::<u32>().ok()?;
                if from > 23 || to > 23 { return None; }
                let date = resolve::resolve_day_offset(offset, now)?;
                resolve::resolve_time_range_on_date(date, from, to)
            },
        },
        // --- Combined: "gestern zwischen 9 und 12 Uhr" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?P<day>heute|morgen|gestern)\s+zwischen\s+(?P<from>\d{1,2})\s+und\s+(?P<to>\d{1,2})\s*(?:Uhr)?\b",
            )
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                let from = caps.name("from")?.as_str().parse::<u32>().ok()?;
                let to = caps.name("to")?.as_str().parse::<u32>().ok()?;
                if from > 23 || to > 23 { return None; }
                let date = resolve::resolve_day_offset(offset, now)?;
                resolve::resolve_time_range_on_date(date, from, to)
            },
        },
        // --- Relative days ---
        GrammarRule {
            pattern: Regex::new(r"(?i)\b(?P<day>heute|morgen|gestern)\b").unwrap(),
            kind: ExpressionKind::RelativeDay,
            resolver: |caps, now| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                resolve::resolve_relative_day(offset, now)
            },
        },
        // --- Day offset: "vor 3 Tagen" ---
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\bvor\s+(?P<num>{num})\s+Tagen?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::RelativeDayOffset,
            resolver: |caps, now| {
                let n = parse_num(caps.name("num")?.as_str())?;
                resolve::resolve_relative_day(-(n as i64), now)
            },
        },
        // --- Day offset: "in 3 Tagen" ---
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\bin\s+(?P<num>{num})\s+Tagen?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::RelativeDayOffset,
            resolver: |caps, now| {
                let n = parse_num(caps.name("num")?.as_str())?;
                resolve::resolve_relative_day(n as i64, now)
            },
        },
        // --- Time spec: "um 15 Uhr" ---
        GrammarRule {
            pattern: Regex::new(r"(?i)\bum\s+(?P<hour>\d{1,2})\s+Uhr\b").unwrap(),
            kind: ExpressionKind::TimeSpecification,
            resolver: |caps, now| {
                let hour = caps.name("hour")?.as_str().parse::<u32>().ok()?;
                if hour > 23 { return None; }
                resolve::resolve_time_today(hour, 0, now)
            },
        },
        // --- Time range: "die letzte Stunde/Minute" ---
        GrammarRule {
            pattern: Regex::new(r"(?i)\b(?:die\s+)?letzte\s+(?P<unit>Stunde|Minute)\b").unwrap(),
            kind: ExpressionKind::TimeRange,
            resolver: |caps, now| {
                let unit = caps.name("unit")?.as_str().to_lowercase();
                let mapped = match unit.as_str() {
                    "stunde" => "hour",
                    "minute" => "minute",
                    _ => return None,
                };
                resolve::resolve_last_duration(mapped, now)
            },
        },
        // --- Time range: "von 9 bis 12 Uhr" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\bvon\s+(?P<from>\d{1,2})\s+bis\s+(?P<to>\d{1,2})\s*Uhr\b",
            )
            .unwrap(),
            kind: ExpressionKind::TimeRange,
            resolver: |caps, now| {
                let from = caps.name("from")?.as_str().parse::<u32>().ok()?;
                let to = caps.name("to")?.as_str().parse::<u32>().ok()?;
                if from > 23 || to > 23 { return None; }
                resolve::resolve_time_range_today(from, to, now)
            },
        },
        // --- More Time Ranges ---
        GrammarRule {
             pattern: Regex::new(
                 r"(?i)\bzwischen\s+(?P<from>\d{1,2})\s+und\s+(?P<to>\d{1,2})\s*(?:Uhr)?\b",
             )
             .unwrap(),
             kind: ExpressionKind::TimeRange,
             resolver: |caps, now| {
                 let from = caps.name("from")?.as_str().parse::<u32>().ok()?;
                 let to = caps.name("to")?.as_str().parse::<u32>().ok()?;
                 if from > 23 || to > 23 { return None; }
                 resolve::resolve_time_range_today(from, to, now)
             },
         },
        // --- Next/Last/This Weekday ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?:am\s+)?(?P<dir>n[äae]chsten|kommenden|letzten|vergangenen|diesen)\s+(?P<day>montag|dienstag|mittwoch|donnerstag|freitag|samstag|sonnabend|sonntag)\b"
            )
            .unwrap(),
            kind: ExpressionKind::RelativeDay,
            resolver: |caps, now| {
                let dir_str = caps.name("dir")?.as_str().to_lowercase();
                let direction = match dir_str.as_str() {
                    "nächsten" | "naechsten" | "kommenden" => 1,
                    "letzten" | "vergangenen" => -1,
                    "diesen" => 0,
                    _ => return None,
                };
                let weekday = parse_weekday(caps.name("day")?.as_str())?;
                resolve::resolve_weekday(weekday, direction, now)
            },
        },
    ]
}

impl LanguageParser for German {
    fn lang_id(&self) -> &'static str {
        "de"
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
