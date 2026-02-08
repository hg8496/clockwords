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
    "la",
    "las",
    "\u{fa}ltima",
    "ultima",
    "pr\u{f3}ximo",
    "proximo",
    "pasado",
    "que viene",
    "este",
    "lunes",
    "martes",
    "mi\u{e9}rcoles",
    "miercoles",
    "jueves",
    "viernes",
    "s\u{e1}bado",
    "sabado",
    "domingo",
];

const PREFIXES: &[&str] = &[
    "hoy",
    "man", "mana", "mañan",
    "aye",
    "ent", "entr",
    "hac",
    "últ", "ulti", "ultim",
    "pró", "pro", "prox", "próx", "próxi", "proxi",
    "pas", "pasa", "pasad",
    "est", "este",
    "lun", "lune",
    "mar", "mart", "marte",
    "mié", "mie", "mier", "miérc", "mierc",
    "jue", "juev", "jueve",
    "vie", "vier", "viern", "vierne",
    "sáb", "sab", "sába", "saba", "sábad", "sabad",
    "dom", "domi", "domin", "doming",
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

fn parse_weekday(s: &str) -> Option<chrono::Weekday> {
    match s.to_lowercase().as_str() {
        "lunes" => Some(chrono::Weekday::Mon),
        "martes" => Some(chrono::Weekday::Tue),
        "mi\u{e9}rcoles" | "miercoles" => Some(chrono::Weekday::Wed),
        "jueves" => Some(chrono::Weekday::Thu),
        "viernes" => Some(chrono::Weekday::Fri),
        "s\u{e1}bado" | "sabado" => Some(chrono::Weekday::Sat),
        "domingo" => Some(chrono::Weekday::Sun),
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

/// Shared weekday pattern (accent-tolerant)
const WEEKDAY_PAT: &str = r"lunes|martes|mi[eé]rcoles|jueves|viernes|s[aá]bado|domingo";

fn es_weekday_direction(s: &str) -> Option<i64> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "próximo" | "proximo" => Some(1),
        "pasado" => Some(-1),
        "este" => Some(0),
        _ if lower.contains("viene") => Some(1),
        _ => None,
    }
}

fn build_rules() -> Vec<GrammarRule> {
    let num = NUM_WORD_PATTERN;
    let wd = WEEKDAY_PAT;

    vec![
        // ============================================================
        //  Combined: Weekday (pre-positive) + "a las X"
        //  "el próximo lunes a las 3"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?:el\s+)?(?P<dir>pr[oó]ximo|pasado|este)\s+(?P<wd>{wd})\s+a\s+las\s+(?P<hour>\d{{1,2}})\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now| {
                let direction = es_weekday_direction(caps.name("dir")?.as_str())?;
                let weekday = parse_weekday(caps.name("wd")?.as_str())?;
                let hour = caps.name("hour")?.as_str().parse::<u32>().ok()?;
                if hour > 23 { return None; }
                let date = resolve::resolve_weekday_date(weekday, direction, now)?;
                resolve::resolve_time_on_date(date, hour, 0)
            },
        },
        // ============================================================
        //  Combined: Weekday (pre-positive) + "entre las X y las Y"
        //  "el pasado viernes entre las 9 y las 12"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?:el\s+)?(?P<dir>pr[oó]ximo|pasado|este)\s+(?P<wd>{wd})\s+entre\s+las\s+(?P<from>\d{{1,2}})\s+y\s+las\s+(?P<to>\d{{1,2}})\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now| {
                let direction = es_weekday_direction(caps.name("dir")?.as_str())?;
                let weekday = parse_weekday(caps.name("wd")?.as_str())?;
                let from = caps.name("from")?.as_str().parse::<u32>().ok()?;
                let to = caps.name("to")?.as_str().parse::<u32>().ok()?;
                if from > 23 || to > 23 { return None; }
                let date = resolve::resolve_weekday_date(weekday, direction, now)?;
                resolve::resolve_time_range_on_date(date, from, to)
            },
        },
        // ============================================================
        //  Combined: Weekday (post-positive) + "a las X"
        //  "el viernes pasado a las 3"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?:el\s+)?(?P<wd>{wd})\s+(?P<dir>pr[oó]ximo|pasado|que\s+viene)\s+a\s+las\s+(?P<hour>\d{{1,2}})\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now| {
                let direction = es_weekday_direction(caps.name("dir")?.as_str())?;
                let weekday = parse_weekday(caps.name("wd")?.as_str())?;
                let hour = caps.name("hour")?.as_str().parse::<u32>().ok()?;
                if hour > 23 { return None; }
                let date = resolve::resolve_weekday_date(weekday, direction, now)?;
                resolve::resolve_time_on_date(date, hour, 0)
            },
        },
        // ============================================================
        //  Combined: Weekday (post-positive) + "entre las X y las Y"
        //  "el viernes pasado entre las 9 y las 12"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?:el\s+)?(?P<wd>{wd})\s+(?P<dir>pr[oó]ximo|pasado|que\s+viene)\s+entre\s+las\s+(?P<from>\d{{1,2}})\s+y\s+las\s+(?P<to>\d{{1,2}})\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now| {
                let direction = es_weekday_direction(caps.name("dir")?.as_str())?;
                let weekday = parse_weekday(caps.name("wd")?.as_str())?;
                let from = caps.name("from")?.as_str().parse::<u32>().ok()?;
                let to = caps.name("to")?.as_str().parse::<u32>().ok()?;
                if from > 23 || to > 23 { return None; }
                let date = resolve::resolve_weekday_date(weekday, direction, now)?;
                resolve::resolve_time_range_on_date(date, from, to)
            },
        },
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
                let date = resolve::resolve_day_offset(offset, now)?;
                resolve::resolve_time_on_date(date, hour, 0)
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
                let date = resolve::resolve_day_offset(offset, now)?;
                resolve::resolve_time_range_on_date(date, from, to)
            },
        },
        // --- Relative days ---
        GrammarRule {
            pattern: Regex::new(r"(?i)\b(?P<day>hoy|ma[ñn]ana|ayer)\b").unwrap(),
            kind: ExpressionKind::RelativeDay,
            resolver: |caps, now| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                resolve::resolve_relative_day(offset, now)
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
                resolve::resolve_relative_day(-(n as i64), now)
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
                resolve::resolve_relative_day(n as i64, now)
            },
        },
        // --- Time spec: "a las 3" ---
        GrammarRule {
            pattern: Regex::new(r"(?i)\ba\s+las\s+(?P<hour>\d{1,2})\b").unwrap(),
            kind: ExpressionKind::TimeSpecification,
            resolver: |caps, now| {
                let hour = caps.name("hour")?.as_str().parse::<u32>().ok()?;
                if hour > 23 { return None; }
                resolve::resolve_time_today(hour, 0, now)
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
                resolve::resolve_last_duration(mapped, now)
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
                resolve::resolve_time_range_today(from, to, now)
            },
        },
        // --- Next/Last/This Weekday (Pre-positive: "el próximo lunes") ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?:el\s+)?(?P<dir>pr[oó]ximo|pasado|este)\s+(?P<day>lunes|martes|mi[eé]rcoles|jueves|viernes|s[aá]bado|domingo)\b"
            )
            .unwrap(),
            kind: ExpressionKind::RelativeDay,
            resolver: |caps, now| {
                let dir_str = caps.name("dir")?.as_str().to_lowercase();
                let direction = match dir_str.as_str() {
                    "próximo" | "proximo" => 1,
                    "pasado" => -1,
                    "este" => 0,
                    _ => return None,
                };
                let weekday = parse_weekday(caps.name("day")?.as_str())?;
                resolve::resolve_weekday(weekday, direction, now)
            },
        },
        // --- Next/Last/This Weekday (Post-positive: "el lunes que viene") ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?:el\s+)?(?P<day>lunes|martes|mi[eé]rcoles|jueves|viernes|s[aá]bado|domingo)\s+(?P<dir>pr[oó]ximo|pasado|que\s+viene)\b"
            )
            .unwrap(),
            kind: ExpressionKind::RelativeDay,
            resolver: |caps, now| {
                let dir_str = caps.name("dir")?.as_str().to_lowercase();
                let direction = if dir_str.contains("pasado") {
                     -1 
                } else { 
                     1 // "próximo" or "que viene"
                };
                let weekday = parse_weekday(caps.name("day")?.as_str())?;
                resolve::resolve_weekday(weekday, direction, now)
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
