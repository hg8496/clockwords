use chrono::{DateTime, Utc};
use regex::Regex;

use crate::lang::numbers::parse_number_es;
use crate::lang::{apply_rules, GrammarRule, LanguageParser};
use crate::resolve;
use crate::types::*;

const KEYWORDS: &[&str] = &[
    "hoy",
    "ma\u{f1}ana",
    "manana",
    "ayer",
    "hace",
    "en",
    "d\u{ed}as",
    "dias",
    "d\u{ed}a",
    "dia",
    "hora",
    "horas",
    "minuto",
    "minutos",
    "entre",
    "las",
    "\u{fa}ltima",
    "ultima",
];

const PREFIXES: &[&str] = &[
    "hoy",
    "man", "mana", "mañan",
    "aye",
    "ent", "entr",
    "hac",
    "últ", "ulti", "ultim",
];

const NUM_WORD_PATTERN: &str =
    r"(?:\d+|un|uno|una|dos|tres|cuatro|cinco|seis|siete|ocho|nueve|diez|once|doce|trece|catorce|quince|veinte|treinta)";

fn day_keyword_offset(s: &str) -> Option<i64> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "hoy" => Some(0),
        "ma\u{f1}ana" | "manana" => Some(1),
        "ayer" => Some(-1),
        _ => None,
    }
}

fn parse_num(s: &str) -> Option<u32> {
    s.parse::<u32>()
        .ok()
        .or_else(|| parse_number_es(&s.to_lowercase()))
}

pub struct Spanish {
    rules: Vec<GrammarRule>,
}

impl Default for Spanish {
    fn default() -> Self {
        Self::new()
    }
}

impl Spanish {
    pub fn new() -> Self {
        Self {
            rules: build_rules(),
        }
    }
}

fn build_rules() -> Vec<GrammarRule> {
    let num = NUM_WORD_PATTERN;

    vec![
        // --- Combined: "ayer a las 3" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?P<day>hoy|ma[ñn]ana|ayer)\s+a\s+las\s+(?P<hour>\d{1,2})\b",
            )
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                let hour = caps.name("hour")?.as_str().parse::<u32>().ok()?;
                if hour > 23 { return None; }
                let date = resolve::resolve_day_offset(offset, now);
                Some(resolve::resolve_time_on_date(date, hour, 0))
            },
        },
        // --- Combined: "ayer entre las 9 y las 12" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?P<day>hoy|ma[ñn]ana|ayer)\s+entre\s+las\s+(?P<from>\d{1,2})\s+y\s+las\s+(?P<to>\d{1,2})\b",
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
            pattern: Regex::new(r"(?i)\b(?P<day>hoy|ma[ñn]ana|ayer)\b").unwrap(),
            kind: ExpressionKind::RelativeDay,
            resolver: |caps, now| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                Some(resolve::resolve_relative_day(offset, now))
            },
        },
        // --- Day offset: "hace 2 días" ---
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\bhace\s+(?P<num>{num})\s+d[ií]as?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::RelativeDayOffset,
            resolver: |caps, now| {
                let n = parse_num(caps.name("num")?.as_str())?;
                Some(resolve::resolve_relative_day(-(n as i64), now))
            },
        },
        // --- Day offset: "en 3 días" ---
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\ben\s+(?P<num>{num})\s+d[ií]as?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::RelativeDayOffset,
            resolver: |caps, now| {
                let n = parse_num(caps.name("num")?.as_str())?;
                Some(resolve::resolve_relative_day(n as i64, now))
            },
        },
        // --- Time spec: "a las 3" ---
        GrammarRule {
            pattern: Regex::new(r"(?i)\ba\s+las\s+(?P<hour>\d{1,2})\b").unwrap(),
            kind: ExpressionKind::TimeSpecification,
            resolver: |caps, now| {
                let hour = caps.name("hour")?.as_str().parse::<u32>().ok()?;
                if hour > 23 { return None; }
                Some(resolve::resolve_time_today(hour, 0, now))
            },
        },
        // --- Time range: "la última hora" ---
        GrammarRule {
            pattern: Regex::new(r"(?i)\b(?:la\s+)?[úu]ltima\s+(?P<unit>hora|minuto)\b").unwrap(),
            kind: ExpressionKind::TimeRange,
            resolver: |caps, now| {
                let unit = caps.name("unit")?.as_str().to_lowercase();
                let mapped = match unit.as_str() {
                    "hora" => "hour",
                    "minuto" => "minute",
                    _ => return None,
                };
                Some(resolve::resolve_last_duration(mapped, now))
            },
        },
        // --- Time range: "entre las 9 y las 12" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\bentre\s+las\s+(?P<from>\d{1,2})\s+y\s+las\s+(?P<to>\d{1,2})\b",
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

impl LanguageParser for Spanish {
    fn lang_id(&self) -> &'static str {
        "es"
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
