use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use regex::Regex;

use crate::lang::numbers::parse_number_fr;
use crate::lang::{GrammarRule, LanguageParser, apply_rules};
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
    "ce",
    "prochain",
    "dernier",
    "lundi",
    "mardi",
    "mercredi",
    "jeudi",
    "vendredi",
    "samedi",
    "dimanche",
];

const PREFIXES: &[&str] = &[
    "auj", "aujo", "aujou", "aujour", "aujourd", "dem", "dema", "demai", "hie", "ent", "entr",
    "der", "dern", "derni", "pro", "proc", "proch", "procha", "prochai", "lun", "lund", "mar",
    "mard", "mer", "merc", "mercr", "mercre", "mercred", "jeu", "jeud", "ven", "vend", "vendr",
    "vendre", "vendred", "sam", "same", "samed", "dim", "dima", "diman", "dimanc", "dimanch",
];

const NUM_WORD_PATTERN: &str = r"(?:\d+|un|une|deux|trois|quatre|cinq|six|sept|huit|neuf|dix|onze|douze|treize|quatorze|quinze|seize|vingt|trente)";

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

fn parse_weekday(s: &str) -> Option<chrono::Weekday> {
    match s.to_lowercase().as_str() {
        "lundi" => Some(chrono::Weekday::Mon),
        "mardi" => Some(chrono::Weekday::Tue),
        "mercredi" => Some(chrono::Weekday::Wed),
        "jeudi" => Some(chrono::Weekday::Thu),
        "vendredi" => Some(chrono::Weekday::Fri),
        "samedi" => Some(chrono::Weekday::Sat),
        "dimanche" => Some(chrono::Weekday::Sun),
        _ => None,
    }
}

fn parse_num(s: &str) -> Option<u32> {
    s.parse::<u32>()
        .ok()
        .or_else(|| parse_number_fr(&s.to_lowercase()))
}

pub struct French {
    rules: Vec<GrammarRule>,
    regex_set: regex::RegexSet,
}

impl Default for French {
    fn default() -> Self {
        Self::new()
    }
}

impl French {
    pub fn new() -> Self {
        let rules = build_rules();
        let regex_set = regex::RegexSet::new(rules.iter().map(|r| r.pattern.as_str())).unwrap();
        Self { rules, regex_set }
    }
}

/// Shared weekday pattern
const WEEKDAY_PAT: &str = r"lundi|mardi|mercredi|jeudi|vendredi|samedi|dimanche";

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

/// Parse hour and optional minutes from captures (24h format).
/// Supports both `Xh30` / `X:30` (colon/h with digits) and bare `Xh` forms.
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
    let num = NUM_WORD_PATTERN;
    let wd = WEEKDAY_PAT;

    vec![
        // ============================================================
        //  Combined: Weekday (post-positive) + "à Xh30" / "à X:30" / "à Xh"
        //  "vendredi dernier à 13h30", "vendredi dernier à 13:30", "vendredi dernier à 13h"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?:le\s+)?(?P<day>{wd})\s+(?P<dir>prochain|dernier)\s+[àa]\s+(?P<hour>\d{{1,2}})(?:[h:](?P<min>\d{{2}})|\s*h)\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let direction = match caps.name("dir")?.as_str().to_lowercase().as_str() {
                    "prochain" => 1,
                    "dernier" => -1,
                    _ => return None,
                };
                let weekday = parse_weekday(caps.name("day")?.as_str())?;
                let (h, m) = parse_hm(caps)?;
                let date = resolve::resolve_weekday_date(weekday, direction, now, tz)?;
                resolve::resolve_time_on_date(date, h, m, tz)
            },
        },
        // ============================================================
        //  Combined: Weekday (post-positive) + "de HH:MM à/- HH:MM"
        //  "vendredi dernier de 10:15 à 13:45", "vendredi dernier de 9:00 - 11:30"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?:le\s+)?(?P<day>{wd})\s+(?P<dir>prochain|dernier)\s+de\s+(?P<fh>\d{{1,2}}):(?P<fm>\d{{2}})\s*(?:[àa]|-)\s*(?P<th>\d{{1,2}}):(?P<tm>\d{{2}})\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let direction = match caps.name("dir")?.as_str().to_lowercase().as_str() {
                    "prochain" => 1,
                    "dernier" => -1,
                    _ => return None,
                };
                let weekday = parse_weekday(caps.name("day")?.as_str())?;
                let (fh, fm, th, tm) = parse_hm_range(caps)?;
                let date = resolve::resolve_weekday_date(weekday, direction, now, tz)?;
                resolve::resolve_time_range_with_minutes_on_date(date, fh, fm, th, tm, tz)
            },
        },
        // ============================================================
        //  Combined: Weekday (post-positive) + "HH:MM - HH:MM" (bare dash)
        //  "vendredi dernier 9:00 - 11:30"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?:le\s+)?(?P<day>{wd})\s+(?P<dir>prochain|dernier)\s+(?P<fh>\d{{1,2}}):(?P<fm>\d{{2}})\s*-\s*(?P<th>\d{{1,2}}):(?P<tm>\d{{2}})\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let direction = match caps.name("dir")?.as_str().to_lowercase().as_str() {
                    "prochain" => 1,
                    "dernier" => -1,
                    _ => return None,
                };
                let weekday = parse_weekday(caps.name("day")?.as_str())?;
                let (fh, fm, th, tm) = parse_hm_range(caps)?;
                let date = resolve::resolve_weekday_date(weekday, direction, now, tz)?;
                resolve::resolve_time_range_with_minutes_on_date(date, fh, fm, th, tm, tz)
            },
        },
        // ============================================================
        //  Combined: Weekday (post-positive) + "entre X et Y heures"
        //  "vendredi dernier entre 9 et 12 heures"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\b(?:le\s+)?(?P<day>{wd})\s+(?P<dir>prochain|dernier)\s+entre\s+(?P<from>\d{{1,2}})\s+et\s+(?P<to>\d{{1,2}})\s*(?:heures?)?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let direction = match caps.name("dir")?.as_str().to_lowercase().as_str() {
                    "prochain" => 1,
                    "dernier" => -1,
                    _ => return None,
                };
                let weekday = parse_weekday(caps.name("day")?.as_str())?;
                let from = caps.name("from")?.as_str().parse::<u32>().ok()?;
                let to = caps.name("to")?.as_str().parse::<u32>().ok()?;
                if from > 23 || to > 23 { return None; }
                let date = resolve::resolve_weekday_date(weekday, direction, now, tz)?;
                resolve::resolve_time_range_on_date(date, from, to, tz)
            },
        },
        // ============================================================
        //  Combined: "ce lundi à 13h30" / "ce lundi à 13:30" / "ce lundi à 13h"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\bce\s+(?P<day>{wd})\s+[àa]\s+(?P<hour>\d{{1,2}})(?:[h:](?P<min>\d{{2}})|\s*h)\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let weekday = parse_weekday(caps.name("day")?.as_str())?;
                let (h, m) = parse_hm(caps)?;
                let date = resolve::resolve_weekday_date(weekday, 0, now, tz)?;
                resolve::resolve_time_on_date(date, h, m, tz)
            },
        },
        // ============================================================
        //  Combined: "ce lundi de 10:15 à 13:45", "ce lundi de 9:00 - 11:30"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\bce\s+(?P<day>{wd})\s+de\s+(?P<fh>\d{{1,2}}):(?P<fm>\d{{2}})\s*(?:[àa]|-)\s*(?P<th>\d{{1,2}}):(?P<tm>\d{{2}})\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let weekday = parse_weekday(caps.name("day")?.as_str())?;
                let (fh, fm, th, tm) = parse_hm_range(caps)?;
                let date = resolve::resolve_weekday_date(weekday, 0, now, tz)?;
                resolve::resolve_time_range_with_minutes_on_date(date, fh, fm, th, tm, tz)
            },
        },
        // ============================================================
        //  Combined: "ce lundi entre 9 et 12 heures"
        // ============================================================
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\bce\s+(?P<day>{wd})\s+entre\s+(?P<from>\d{{1,2}})\s+et\s+(?P<to>\d{{1,2}})\s*(?:heures?)?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let weekday = parse_weekday(caps.name("day")?.as_str())?;
                let from = caps.name("from")?.as_str().parse::<u32>().ok()?;
                let to = caps.name("to")?.as_str().parse::<u32>().ok()?;
                if from > 23 || to > 23 { return None; }
                let date = resolve::resolve_weekday_date(weekday, 0, now, tz)?;
                resolve::resolve_time_range_on_date(date, from, to, tz)
            },
        },
        // --- Combined: "hier à 13h30" / "hier à 13:30" / "hier à 13h" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?P<day>aujourd['\u{2019}]hui|demain|hier)\s+[àa]\s+(?P<hour>\d{1,2})(?:[h:](?P<min>\d{2})|\s*h)\b",
            )
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                let (h, m) = parse_hm(caps)?;
                let date = resolve::resolve_day_offset(offset, now, tz)?;
                resolve::resolve_time_on_date(date, h, m, tz)
            },
        },
        // --- Combined: "hier de 10:15 à 13:45", "hier de 9:00 - 11:30" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?P<day>aujourd['\u{2019}]hui|demain|hier)\s+de\s+(?P<fh>\d{1,2}):(?P<fm>\d{2})\s*(?:[àa]|-)\s*(?P<th>\d{1,2}):(?P<tm>\d{2})\b",
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
        // --- Combined: "hier 10:15 - 13:45" (day + bare dash) ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?P<day>aujourd['\u{2019}]hui|demain|hier)\s+(?P<fh>\d{1,2}):(?P<fm>\d{2})\s*-\s*(?P<th>\d{1,2}):(?P<tm>\d{2})\b",
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
        // --- Combined: "hier entre 9 et 12 heures" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?P<day>aujourd['\u{2019}]hui|demain|hier)\s+entre\s+(?P<from>\d{1,2})\s+et\s+(?P<to>\d{1,2})\s*(?:heures?)?\b",
            )
            .unwrap(),
            kind: ExpressionKind::Combined,
            resolver: |caps, now, tz| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                let from = caps.name("from")?.as_str().parse::<u32>().ok()?;
                let to = caps.name("to")?.as_str().parse::<u32>().ok()?;
                if from > 23 || to > 23 { return None; }
                let date = resolve::resolve_day_offset(offset, now, tz)?;
                resolve::resolve_time_range_on_date(date, from, to, tz)
            },
        },
        // --- Relative days ---
        GrammarRule {
            pattern: Regex::new(r"(?i)\b(?P<day>aujourd['\u{2019}]hui|demain|hier)\b").unwrap(),
            kind: ExpressionKind::RelativeDay,
            resolver: |caps, now, tz| {
                let offset = day_keyword_offset(caps.name("day")?.as_str())?;
                resolve::resolve_relative_day(offset, now, tz)
            },
        },
        // --- Day offset: "il y a 3 jours" ---
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\bil\s+y\s+a\s+(?P<num>{num})\s+jours?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::RelativeDayOffset,
            resolver: |caps, now, tz| {
                let n = parse_num(caps.name("num")?.as_str())?;
                resolve::resolve_relative_day(-(n as i64), now, tz)
            },
        },
        // --- Day offset: "dans 3 jours" ---
        GrammarRule {
            pattern: Regex::new(&format!(
                r"(?i)\bdans\s+(?P<num>{num})\s+jours?\b"
            ))
            .unwrap(),
            kind: ExpressionKind::RelativeDayOffset,
            resolver: |caps, now, tz| {
                let n = parse_num(caps.name("num")?.as_str())?;
                resolve::resolve_relative_day(n as i64, now, tz)
            },
        },
        // --- Time spec: "à 13h30" / "à 13:30" / "à 13h" ---
        GrammarRule {
            pattern: Regex::new(r"(?i)(?:^|\b)[àa]\s+(?P<hour>\d{1,2})(?:[h:](?P<min>\d{2})|\s*h)\b").unwrap(),
            kind: ExpressionKind::TimeSpecification,
            resolver: |caps, now, tz| {
                let (h, m) = parse_hm(caps)?;
                resolve::resolve_time_today(h, m, now, tz)
            },
        },
        // --- Time range: "la dernière heure" ---
        GrammarRule {
            pattern: Regex::new(r"(?i)\b(?:la\s+)?derni[èe]re\s+(?P<unit>heure|minute)\b")
                .unwrap(),
            kind: ExpressionKind::TimeRange,
            resolver: |caps, now, _tz| {
                let unit = caps.name("unit")?.as_str().to_lowercase();
                let mapped = match unit.as_str() {
                    "heure" => "hour",
                    "minute" => "minute",
                    _ => return None,
                };
                resolve::resolve_last_duration(mapped, now)
            },
        },
        // --- Time range: "de 10:15 à 13:45", "de 9:00 - 11:30" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\bde\s+(?P<fh>\d{1,2}):(?P<fm>\d{2})\s*(?:[àa]|-)\s*(?P<th>\d{1,2}):(?P<tm>\d{2})\b",
            )
            .unwrap(),
            kind: ExpressionKind::TimeRange,
            resolver: |caps, now, tz| {
                let (fh, fm, th, tm) = parse_hm_range(caps)?;
                resolve::resolve_time_range_with_minutes_today(fh, fm, th, tm, now, tz)
            },
        },
        // --- Time range: "entre 9 et 12 heures" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\bentre\s+(?P<from>\d{1,2})\s+et\s+(?P<to>\d{1,2})\s*(?:heures?)?\b",
            )
            .unwrap(),
            kind: ExpressionKind::TimeRange,
            resolver: |caps, now, tz| {
                let from = caps.name("from")?.as_str().parse::<u32>().ok()?;
                let to = caps.name("to")?.as_str().parse::<u32>().ok()?;
                if from > 23 || to > 23 { return None; }
                resolve::resolve_time_range_today(from, to, now, tz)
            },
        },
        // --- Next/Last/This Weekday (Post-positive: "lundi prochain") ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?:le\s+)?(?P<day>lundi|mardi|mercredi|jeudi|vendredi|samedi|dimanche)\s+(?P<dir>prochain|dernier)\b"
            )
            .unwrap(),
            kind: ExpressionKind::RelativeDay,
            resolver: |caps, now, tz| {
                let dir_str = caps.name("dir")?.as_str().to_lowercase();
                let direction = match dir_str.as_str() {
                    "prochain" => 1,
                    "dernier" => -1,
                    _ => return None,
                };
                let weekday = parse_weekday(caps.name("day")?.as_str())?;
                resolve::resolve_weekday(weekday, direction, now, tz)
            },
        },
        // --- Next/Last Weekday (Pre-positive: "prochain lundi") ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\b(?:le\s+)?(?P<dir>prochain|dernier)\s+(?P<day>lundi|mardi|mercredi|jeudi|vendredi|samedi|dimanche)\b"
            )
            .unwrap(),
            kind: ExpressionKind::RelativeDay,
            resolver: |caps, now, tz| {
                let dir_str = caps.name("dir")?.as_str().to_lowercase();
                let direction = match dir_str.as_str() {
                    "prochain" => 1,
                    "dernier" => -1,
                    _ => return None,
                };
                let weekday = parse_weekday(caps.name("day")?.as_str())?;
                resolve::resolve_weekday(weekday, direction, now, tz)
            },
        },
        // --- This Weekday: "ce lundi" ---
        GrammarRule {
            pattern: Regex::new(
                r"(?i)\bce\s+(?P<day>lundi|mardi|mercredi|jeudi|vendredi|samedi|dimanche)\b"
            )
            .unwrap(),
            kind: ExpressionKind::RelativeDay,
            resolver: |caps, now, tz| {
                let weekday = parse_weekday(caps.name("day")?.as_str())?;
                resolve::resolve_weekday(weekday, 0, now, tz)
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

    fn parse(&self, text: &str, now: DateTime<Utc>, tz: Tz) -> Vec<TimeMatch> {
        apply_rules(&self.rules, &self.regex_set, text, now, tz)
    }
}
