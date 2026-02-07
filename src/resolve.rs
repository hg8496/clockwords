use chrono::{DateTime, Duration, Utc};

use crate::types::ResolvedTime;

/// Resolve a relative day offset to midnight of that day.
pub fn resolve_day_offset(days: i64, now: DateTime<Utc>) -> DateTime<Utc> {
    let target = now + Duration::days(days);
    target
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
}

/// Resolve "today" to a full-day range.
pub fn resolve_today(now: DateTime<Utc>) -> ResolvedTime {
    let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end = start + Duration::days(1);
    ResolvedTime::Range { start, end }
}

/// Resolve a relative day keyword to a full-day range.
pub fn resolve_relative_day(offset: i64, now: DateTime<Utc>) -> ResolvedTime {
    let start = resolve_day_offset(offset, now);
    let end = start + Duration::days(1);
    ResolvedTime::Range { start, end }
}

/// Set time-of-day on a given date, returning a point.
pub fn resolve_time_on_date(
    date: DateTime<Utc>,
    hour: u32,
    minute: u32,
) -> ResolvedTime {
    let point = date
        .date_naive()
        .and_hms_opt(hour, minute, 0)
        .unwrap()
        .and_utc();
    ResolvedTime::Point(point)
}

/// Set time-of-day on today.
pub fn resolve_time_today(hour: u32, minute: u32, now: DateTime<Utc>) -> ResolvedTime {
    resolve_time_on_date(now, hour, minute)
}

/// Resolve "the last hour/minute" as a range ending at now.
pub fn resolve_last_duration(unit: &str, now: DateTime<Utc>) -> ResolvedTime {
    let duration = match unit {
        "hour" | "stunde" | "heure" | "hora" => Duration::hours(1),
        "minute" | "minuto" => Duration::minutes(1),
        _ => Duration::hours(1),
    };
    ResolvedTime::Range {
        start: now - duration,
        end: now,
    }
}

/// Resolve "between X and Y o'clock" on a given date.
pub fn resolve_time_range_on_date(
    date: DateTime<Utc>,
    from_hour: u32,
    to_hour: u32,
) -> ResolvedTime {
    let start = date
        .date_naive()
        .and_hms_opt(from_hour, 0, 0)
        .unwrap()
        .and_utc();
    let end = date
        .date_naive()
        .and_hms_opt(to_hour, 0, 0)
        .unwrap()
        .and_utc();
    ResolvedTime::Range { start, end }
}

/// Resolve "between X and Y" on today.
pub fn resolve_time_range_today(
    from_hour: u32,
    to_hour: u32,
    now: DateTime<Utc>,
) -> ResolvedTime {
    resolve_time_range_on_date(now, from_hour, to_hour)
}

/// Convert 12-hour time to 24-hour.
pub fn to_24h(hour: u32, ampm: &str) -> u32 {
    let ampm_lower = ampm.to_lowercase();
    if ampm_lower == "pm" && hour < 12 {
        hour + 12
    } else if ampm_lower == "am" && hour == 12 {
        0
    } else {
        hour
    }
}
