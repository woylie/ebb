// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use chrono::{Local, NaiveDate, TimeZone};

pub fn format_duration(secs: i64) -> String {
    let negative = secs < 0;
    let mut secs = secs.abs();

    let days = secs / 86400;
    secs %= 86400;
    let hours = secs / 3600;
    secs %= 3600;
    let minutes = secs / 60;
    secs %= 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}d", days));
    }
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

pub fn format_timestamp(ts: i64) -> String {
    match Local.timestamp_opt(ts, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S (%a)").to_string(),
        chrono::LocalResult::Ambiguous(dt1, _) => dt1.format("%Y-%m-%d %H:%M:%S (%a)").to_string(),
        chrono::LocalResult::None => {
            let fallback_date = NaiveDate::from_ymd_opt(1970, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let fallback_dt = Local.from_local_datetime(&fallback_date).unwrap();
            fallback_dt.format("%Y-%m-%d %H:%M:%S (%a)").to_string()
        }
    }
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
            (86400, "1d"),
            (88888, "1d 41m 28s"),
            (1440000, "16d 16h"),
        ];

        for (secs, expected_str) in cases {
            assert_eq!(format_duration(secs), expected_str, "for {secs} seconds")
        }
    }
}
