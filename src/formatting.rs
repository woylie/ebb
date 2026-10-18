// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Result, anyhow, bail};
use chrono::{
    DateTime, Local, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone,
};

/// Loading a file checks every timestamp with this function.
pub fn local_datetime(ts: i64) -> Result<DateTime<Local>> {
    match Local.timestamp_opt(ts, 0) {
        LocalResult::Single(dt) => Ok(dt),
        LocalResult::Ambiguous(dt, _) => Ok(dt),
        LocalResult::None => bail!("{ts} is not a valid timestamp"),
    }
}

#[expect(
    clippy::expect_used,
    reason = "timestamps are checked when their file is loaded"
)]
pub fn to_local(ts: i64) -> DateTime<Local> {
    local_datetime(ts).expect("timestamps are checked when their file is loaded")
}

/// The first instant of a date in the local time zone. When midnight occurs twice, this is the
/// first midnight. When the clock skips midnight, this is the moment the clock jumps.
pub fn local_midnight(date: NaiveDate) -> Result<DateTime<Local>> {
    let midnight = date.and_time(NaiveTime::MIN);

    (0..=24 * 60)
        .map(|minutes| midnight + TimeDelta::minutes(minutes))
        .find_map(first_instant)
        .ok_or_else(|| anyhow!("The start of {date} cannot be found in the local time zone."))
}

/// The earliest instant with the given local time, or `None` if the clock skips that time.
fn first_instant(local: NaiveDateTime) -> Option<DateTime<Local>> {
    let candidates = match Local.from_local_datetime(&local) {
        LocalResult::Single(instant) => vec![instant],
        LocalResult::Ambiguous(first, second) => vec![first, second],
        LocalResult::None => vec![],
    };

    // chrono can return an instant at the edge of a clock change that shows another local time
    candidates
        .into_iter()
        .filter(|instant| Local.from_utc_datetime(&instant.naive_utc()).naive_local() == local)
        .min()
}

pub fn format_duration(secs: i64) -> String {
    let negative = secs < 0;
    let mut secs = secs.abs();

    let hours = secs / 3600;
    secs %= 3600;
    let minutes = secs / 60;
    secs %= 60;

    let mut parts = Vec::new();
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if secs > 0 || parts.is_empty() {
        parts.push(format!("{}s", secs));
    }

    let result = parts.join(" ");
    if negative {
        format!("-{result}")
    } else {
        result
    }
}

pub fn format_timerange(from: i64, to: i64) -> String {
    let from_str = format_timestamp(from);
    let to_str = format_timestamp(to);
    format!("From: {from_str}\nTo: {to_str}")
}

pub fn format_datetime(ts: i64) -> String {
    to_local(ts).format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn format_timestamp(ts: i64) -> String {
    to_local(ts).format("%Y-%m-%d %H:%M:%S (%a)").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_seconds_as_duration_str() {
        let cases = [
            (0, "0s"),
            (1, "1s"),
            (59, "59s"),
            (60, "1m"),
            (61, "1m 1s"),
            (3599, "59m 59s"),
            (3600, "1h"),
            (3601, "1h 1s"),
            (3662, "1h 1m 2s"),
            (86400, "24h"),
            (88888, "24h 41m 28s"),
            (93900, "26h 5m"),
            (1440000, "400h"),
            (-93900, "-26h 5m"),
        ];

        for (secs, expected_str) in cases {
            assert_eq!(format_duration(secs), expected_str, "for {secs} seconds")
        }
    }
}
