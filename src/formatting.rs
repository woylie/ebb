// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

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
