use chrono::{DateTime, Duration, Utc};

use crate::types::ResolvedTime;

/// Resolve a relative day offset to midnight (00:00:00) of that day.
///
/// Returns `None` if the resulting date cannot be represented (e.g., overflow).
pub fn resolve_day_offset(days: i64, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let target = now.checked_add_signed(Duration::days(days))?;
    Some(target.date_naive().and_hms_opt(0, 0, 0)?.and_utc())
}

/// Resolve a relative day keyword to a full-day range (midnight to midnight).
///
/// `offset` is the number of days from `now`: 0 = today, 1 = tomorrow, -1 = yesterday.
/// Returns `None` if the date arithmetic overflows.
pub fn resolve_relative_day(offset: i64, now: DateTime<Utc>) -> Option<ResolvedTime> {
    let start = resolve_day_offset(offset, now)?;
    let end = start.checked_add_signed(Duration::days(1))?;
    Some(ResolvedTime::Range { start, end })
}

/// Set time-of-day on a given date, returning a point.
///
/// Returns `None` if `hour` >= 24 or `minute` >= 60.
pub fn resolve_time_on_date(date: DateTime<Utc>, hour: u32, minute: u32) -> Option<ResolvedTime> {
    let point = date.date_naive().and_hms_opt(hour, minute, 0)?.and_utc();
    Some(ResolvedTime::Point(point))
}

/// Set time-of-day on the same date as `now`.
///
/// Returns `None` if `hour` >= 24 or `minute` >= 60.
pub fn resolve_time_today(hour: u32, minute: u32, now: DateTime<Utc>) -> Option<ResolvedTime> {
    resolve_time_on_date(now, hour, minute)
}

/// Resolve "the last hour/minute" as a range ending at `now`.
///
/// Supported unit strings: `"hour"`, `"minute"`.
/// Returns `None` if the subtraction overflows (should not happen in practice).
pub fn resolve_last_duration(unit: &str, now: DateTime<Utc>) -> Option<ResolvedTime> {
    let duration = match unit {
        "hour" => Duration::hours(1),
        "minute" => Duration::minutes(1),
        _ => return None,
    };
    let start = now.checked_sub_signed(duration)?;
    Some(ResolvedTime::Range { start, end: now })
}

/// Resolve "between X and Y o'clock" on a given date.
///
/// Returns `None` if `from_hour` >= 24 or `to_hour` >= 24.
pub fn resolve_time_range_on_date(
    date: DateTime<Utc>,
    from_hour: u32,
    to_hour: u32,
) -> Option<ResolvedTime> {
    let start = date.date_naive().and_hms_opt(from_hour, 0, 0)?.and_utc();
    let end = date.date_naive().and_hms_opt(to_hour, 0, 0)?.and_utc();
    Some(ResolvedTime::Range { start, end })
}

/// Resolve "between X and Y" on the same date as `now`.
///
/// Returns `None` if `from_hour` >= 24 or `to_hour` >= 24.
pub fn resolve_time_range_today(
    from_hour: u32,
    to_hour: u32,
    now: DateTime<Utc>,
) -> Option<ResolvedTime> {
    resolve_time_range_on_date(now, from_hour, to_hour)
}

/// Convert 12-hour time to 24-hour.
///
/// - `"pm"` with hour < 12 adds 12 (e.g., 3pm → 15).
/// - `"am"` with hour == 12 returns 0 (midnight).
/// - All other cases return the hour unchanged.
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

/// Compute the day-offset for a weekday relative to `now`.
///
/// `direction`:
/// - `1`: next week's occurrence
/// - `-1`: last week's occurrence
/// - `0`: this week's occurrence (today or future within 6 days)
fn weekday_offset(weekday: chrono::Weekday, direction: i64, now: DateTime<Utc>) -> Option<i64> {
    use chrono::Datelike;
    let current_weekday = now.weekday();

    let offset_this =
        (weekday.number_from_monday() as i64 - current_weekday.number_from_monday() as i64 + 7) % 7;

    match direction {
        1 => Some(offset_this + 7),
        -1 => Some(offset_this - 7),
        0 => Some(offset_this),
        _ => None,
    }
}

/// Resolve a relative weekday to a full-day range (midnight to midnight).
///
/// `direction`:
/// - `1`: "Next Monday" (next week's Monday)
/// - `-1`: "Last Monday" (last week's Monday)
/// - `0`: "This Monday" (this coming Monday, or today if it's Monday)
pub fn resolve_weekday(
    weekday: chrono::Weekday,
    direction: i64,
    now: DateTime<Utc>,
) -> Option<ResolvedTime> {
    let true_offset = weekday_offset(weekday, direction, now)?;
    resolve_relative_day(true_offset, now)
}

/// Resolve a relative weekday to midnight of that day (for combining with time specs).
///
/// Returns `DateTime<Utc>` at 00:00:00 of the target day, suitable for passing
/// to [`resolve_time_on_date`] or [`resolve_time_range_on_date`].
pub fn resolve_weekday_date(
    weekday: chrono::Weekday,
    direction: i64,
    now: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    let true_offset = weekday_offset(weekday, direction, now)?;
    resolve_day_offset(true_offset, now)
}
