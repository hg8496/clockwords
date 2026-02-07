use chrono::{DateTime, Utc};
use regex::Regex;

use crate::lang::numbers::parse_number_fr;
use crate::lang::{apply_rules, GrammarRule, LanguageParser};
use crate::resolve;
use crate::types::*;

const KEYWORDS: &[&str] = &[
    "aujourd'hui",
    "aujourd",
    "demain",
    "hier",
    "il y a",
    "dans",
    "jours",
    "jour",
    "heure",
    "heures",
    "minute",
    "minutes",
    "entre",
    "derni\u{e8}re",
    "derniere",
    "la",
    "\u{e0}",
];

const PREFIXES: &[&str] = &[
    "auj", "aujo", "aujou", "aujour", "aujourd",
    "dem", "dema", "demai",
    "hie",
    "ent", "entr",
    "der", "dern", "derni",
];

const NUM_WORD_PATTERN: &str =
    r"(?:\d+|un|une|deux|trois|quatre|cinq|six|sept|huit|neuf|dix|onze|douze|treize|quatorze|quinze|seize|vingt|trente)";

fn day_keyword_offset(s: &str) -> Option<i64> {
    let lower = s.to_lowercase();
    if lower == "aujourd'hui" || lower == "aujourd\u{2019}hui" || lower.starts_with("aujourd") {
        Some(0)
    } else if lower == "demain" {
        Some(1)
    } else if lower == "hier" {
        Some(-1)
    } else {
        None
    }
}

fn parse_num(s: &str) -> Option<u32> {
    s.parse::<u32>()
        .ok()
        .or_else(|| parse_number_fr(&s.to_lowercase()))
}

pub struct French {
    rules: Vec<GrammarRule>,
}

impl Default for French {
    fn default() -> Self {
        Self::new()
    }
}

impl French {
    pub fn new() -> Self {
        Self {
            rules: build_rules(),
        }
    }
}

fn build_rules() -> Vec<GrammarRule> {
    let num = NUM_WORD_PATTERN;

    vec![
        // --- Combined: "hier à 13h" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?P<day>aujourd['\u{2019}]hui|demain|hier)\s+[àa]\s+(?P<hour>\d{1,2})\s*h\b",
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
        // --- Combined: "hier entre 9 et 12 heures" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?P<day>aujourd['\u{2019}]hui|demain|hier)\s+entre\s+(?P<from>\d{1,2})\s+et\s+(?P<to>\d{1,2})\s*(?:heures?)?\b",
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
            pattern: Regex::new(r"(?i)\b(?P<day>aujourd['\u{2019}]hui|demain|hier)\b").unwrap(),
            kind: ExpressionKind::RelativeDay,
            resolver: |caps, now| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                Some(resolve::resolve_relative_day(offset, now))
            },
        },
        // --- Day offset: "il y a 3 jours" ---
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\bil\s+y\s+a\s+(?P<num>{num})\s+jours?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::RelativeDayOffset,
            resolver: |caps, now| {
                let n = parse_num(caps.name("num")?.as_str())?;
                Some(resolve::resolve_relative_day(-(n as i64), now))
            },
        },
        // --- Day offset: "dans 3 jours" ---
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\bdans\s+(?P<num>{num})\s+jours?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::RelativeDayOffset,
            resolver: |caps, now| {
                let n = parse_num(caps.name("num")?.as_str())?;
                Some(resolve::resolve_relative_day(n as i64, now))
            },
        },
        // --- Time spec: "à 13h" ---
        GrammarRule {
            pattern: Regex::new(r"(?i)(?:^|\b)[àa]\s+(?P<hour>\d{1,2})\s*h\b").unwrap(),
            kind: ExpressionKind::TimeSpecification,
            resolver: |caps, now| {
                let hour = caps.name("hour")?.as_str().parse::<u32>().ok()?;
                if hour > 23 { return None; }
                Some(resolve::resolve_time_today(hour, 0, now))
            },
        },
        // --- Time range: "la dernière heure" ---
        GrammarRule {
            pattern: Regex::new(r"(?i)\b(?:la\s+)?derni[èe]re\s+(?P<unit>heure|minute)\b")
                .unwrap(),
            kind: ExpressionKind::TimeRange,
            resolver: |caps, now| {
                let unit = caps.name("unit")?.as_str().to_lowercase();
                let mapped = match unit.as_str() {
                    "heure" => "hour",
                    "minute" => "minute",
                    _ => return None,
                };
                Some(resolve::resolve_last_duration(mapped, now))
            },
        },
        // --- Time range: "entre 9 et 12 heures" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\bentre\s+(?P<from>\d{1,2})\s+et\s+(?P<to>\d{1,2})\s*(?:heures?)?\b",
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

impl LanguageParser for French {
    fn lang_id(&self) -> &'static str {
        "fr"
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
