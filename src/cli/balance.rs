// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::formatting::{format_duration, format_timerange, local_midnight, to_local};
use crate::output::{DisplayOutput, print_output};
use crate::persistence::{
    load_config, load_frames, load_holidays, load_sick_days, load_state, load_vacations,
};
use crate::types::{
    Config, DayPortion, Frame, Frames, Holidays, SickDays, Timespan, Vacations, WorkingHours,
};
use crate::{BalanceArgs, Format};
use anyhow::Context;
use chrono::{Datelike, Days, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize)]
struct BalanceOutput {
    expected_working_seconds: i64,
    actual_working_seconds: i64,
    remaining_working_seconds: i64,
    timespan: Timespan,
}

impl DisplayOutput for BalanceOutput {
    fn to_text(&self) -> String {
        let timerange_str = format_timerange(self.timespan.from, self.timespan.to);
        let expected_duration = format_duration(self.expected_working_seconds);
        let actual_duration = format_duration(self.actual_working_seconds);
        let remaining_duration = format_duration(self.remaining_working_seconds);

        let width = expected_duration
            .len()
            .max(actual_duration.len())
            .max(remaining_duration.len());

        format!(
            r#"
{timerange_str}

Expected:  {expected_duration:>width$}
Actual:    {actual_duration:>width$}
Remaining: {remaining_duration:>width$}
"#
        )
    }
}

pub fn run_balance(args: &BalanceArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    if let (Some(from), Some(to)) = (args.from, args.to)
        && from > to
    {
        anyhow::bail!("'to' must not be before 'from'");
    }

    let now = Utc::now().timestamp();

    let config = load_config(config_path)?;
    let mut frames = load_frames(config_path)?;
    let state = load_state(config_path)?;
    let holidays = load_holidays(config_path)?;
    let sick_days = load_sick_days(config_path)?;
    let vacations = load_vacations(config_path)?;

    // The start is after the current time if the clock was set back since `start`.
    if let Some(current_frame) = &state.current_frame
        && current_frame.start_time < now
    {
        frames.frames.push(Frame {
            start_time: current_frame.start_time,
            end_time: now,
            project: current_frame.project.clone(),
            tags: current_frame.tags.clone(),
            updated_at: now,
        });
    }

    let timespan = resolve_timespan(args, now, &frames.frames)?;

    if timespan.from > timespan.to {
        if args.exclude_today {
            anyhow::bail!(
                "'from' must not be after the start of today when --exclude-today is set"
            );
        }
        anyhow::bail!("'to' must be after 'from'");
    }

    frames
        .filter_by_start_time(timespan.from)
        .filter_by_end_time(timespan.to);

    let expected_working_seconds =
        expected_duration(&config, &timespan, &holidays, &sick_days, &vacations)?;
    let actual_working_seconds = total_duration(&frames);
    let remaining_working_seconds = expected_working_seconds - actual_working_seconds;

    let output = BalanceOutput {
        expected_working_seconds,
        actual_working_seconds,
        remaining_working_seconds,
        timespan,
    };

    print_output(&output, format)?;

    Ok(())
}

fn resolve_timespan(args: &BalanceArgs, now: i64, frames: &[Frame]) -> anyhow::Result<Timespan> {
    let today = to_local(now).date_naive();
    let end = if args.exclude_today {
        local_midnight(today)?.timestamp()
    } else {
        now
    };

    if let Some(start) = args.period.start(today) {
        return Ok(Timespan {
            from: local_midnight(start)?.timestamp(),
            to: end,
        });
    }

    let from = match (args.from, frames.first()) {
        (Some(date), _) => local_midnight(date)?.timestamp(),
        (None, Some(frame)) => local_midnight(to_local(frame.start_time).date_naive())?
            .timestamp()
            .min(end),
        (None, None) => 0,
    };
    let to = match args.to {
        Some(date) => local_midnight(date + Days::new(1))?.timestamp(),
        None => end,
    };

    Ok(Timespan { from, to })
}

fn expected_duration(
    config: &Config,
    timespan: &Timespan,
    holidays: &Holidays,
    sick_days: &SickDays,
    vacations: &Vacations,
) -> anyhow::Result<i64> {
    if timespan.from >= timespan.to {
        return Ok(0);
    }

    let first_date = timestamp_to_local_date(timespan.from);
    let last_date = timestamp_to_local_date(timespan.to - 1);
    let day_offs = merge_day_offs_in_range(vacations, holidays, sick_days, first_date, last_date);

    first_date
        .iter_days()
        .take_while(|date| *date <= last_date)
        .map(|date| hours_owed(date, &config.working_hours, &day_offs))
        .try_fold(Duration::ZERO, Duration::checked_add)
        .and_then(|total| total.as_secs().try_into().ok())
        .context("The expected working duration for the timespan is too large.")
}

/// The working hours for the date's weekday minus the days off on that date.
fn hours_owed(
    date: NaiveDate,
    working_hours: &WorkingHours,
    day_offs: &BTreeMap<NaiveDate, DayPortion>,
) -> Duration {
    let hours = working_hours.on(date.weekday());

    match day_offs.get(&date) {
        None => hours,
        Some(DayPortion::Half) => hours / 2,
        Some(DayPortion::Full) => Duration::ZERO,
    }
}

fn merge_day_offs_in_range(
    vacations: &Vacations,
    holidays: &Holidays,
    sick_days: &SickDays,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> BTreeMap<NaiveDate, DayPortion> {
    let mut combined = BTreeMap::new();

    for (&date, entry) in vacations.range(start_date..=end_date) {
        add_portion(&mut combined, date, &entry.portion);
    }

    for (&date, entry) in holidays.range(start_date..=end_date) {
        add_portion(&mut combined, date, &entry.portion);
    }

    for (&date, entry) in sick_days.range(start_date..=end_date) {
        add_portion(&mut combined, date, &entry.portion);
    }

    combined
}

/// Two days off on one date add up to at least a whole day, and are capped at it.
fn add_portion(map: &mut BTreeMap<NaiveDate, DayPortion>, date: NaiveDate, portion: &DayPortion) {
    map.entry(date)
        .and_modify(|existing| *existing = DayPortion::Full)
        .or_insert_with(|| portion.clone());
}

fn timestamp_to_local_date(secs: i64) -> NaiveDate {
    to_local(secs).date_naive()
}

fn total_duration(frames: &Frames) -> i64 {
    let mut total_time: i64 = 0;

    for frame in &frames.frames {
        let duration = frame.end_time - frame.start_time;
        total_time += duration;
    }

    total_time
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Period;
    use crate::types;
    use chrono::{Local, NaiveDateTime, TimeZone};

    const SECONDS_PER_HOUR: u64 = 3600;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn make_timespan(from_date: NaiveDate, to_date: NaiveDate) -> Timespan {
        Timespan {
            from: Local
                .from_local_datetime(&from_date.and_hms_opt(0, 0, 0).unwrap())
                .unwrap()
                .timestamp(),
            to: Local
                .from_local_datetime(&to_date.and_hms_opt(12, 0, 0).unwrap())
                .unwrap()
                .timestamp(),
        }
    }

    fn make_config(working_hours: WorkingHours) -> Config {
        Config {
            working_hours,
            ..Default::default()
        }
    }

    #[test]
    fn test_counts_whole_end_date() {
        // Monday
        let start_date = date(2024, 1, 1);

        let config = make_config(WorkingHours {
            monday: Duration::from_secs(8 * SECONDS_PER_HOUR),
            tuesday: Duration::from_secs(6 * SECONDS_PER_HOUR),
            wednesday: Duration::ZERO,
            thursday: Duration::ZERO,
            friday: Duration::ZERO,
            saturday: Duration::ZERO,
            sunday: Duration::ZERO,
        });

        let holidays: Holidays = BTreeMap::new();
        let sick_days: SickDays = BTreeMap::new();
        let vacations: Vacations = BTreeMap::new();

        let cases = [
            (date(2024, 1, 1), 8 * 60 * 60),
            (date(2024, 1, 2), 8 * 60 * 60 + 6 * 60 * 60),
            (date(2024, 1, 3), 8 * 60 * 60 + 6 * 60 * 60),
        ];

        for (end_date, expected_seconds) in cases {
            let timespan = make_timespan(start_date, end_date);
            let result =
                expected_duration(&config, &timespan, &holidays, &sick_days, &vacations).unwrap();
            assert_eq!(result, expected_seconds, "for end date {end_date}")
        }
    }

    #[test]
    fn test_covers_multiple_weeks() {
        // Monday
        let start_date = date(2024, 1, 1);

        let config = make_config(WorkingHours {
            monday: Duration::from_secs(8 * SECONDS_PER_HOUR),
            tuesday: Duration::from_secs(6 * SECONDS_PER_HOUR),
            wednesday: Duration::ZERO,
            thursday: Duration::ZERO,
            friday: Duration::ZERO,
            saturday: Duration::ZERO,
            sunday: Duration::ZERO,
        });

        let holidays: Holidays = BTreeMap::new();
        let sick_days: SickDays = BTreeMap::new();
        let vacations: Vacations = BTreeMap::new();

        let end_date = date(2024, 1, 14);
        let expected_seconds = (8 * 60 * 60 + 6 * 60 * 60) * 2;

        let timespan = make_timespan(start_date, end_date);
        let result =
            expected_duration(&config, &timespan, &holidays, &sick_days, &vacations).unwrap();
        assert_eq!(result, expected_seconds, "for end date {end_date}")
    }

    #[test]
    fn test_handles_mid_week_end_date() {
        // Monday
        let start_date = date(2024, 1, 1);

        let config = make_config(WorkingHours {
            monday: Duration::from_secs(8 * SECONDS_PER_HOUR),
            tuesday: Duration::from_secs(6 * SECONDS_PER_HOUR),
            wednesday: Duration::from_secs(5 * SECONDS_PER_HOUR),
            thursday: Duration::ZERO,
            friday: Duration::ZERO,
            saturday: Duration::ZERO,
            sunday: Duration::ZERO,
        });

        let holidays: Holidays = BTreeMap::new();
        let sick_days: SickDays = BTreeMap::new();
        let vacations: Vacations = BTreeMap::new();

        let end_date = date(2024, 1, 16);

        let expected_seconds =
            (8 * 60 * 60 + 6 * 60 * 60 + 5 * 60 * 60) * 2 + (8 * 60 * 60 + 6 * 60 * 60);

        let timespan = make_timespan(start_date, end_date);
        let result =
            expected_duration(&config, &timespan, &holidays, &sick_days, &vacations).unwrap();
        assert_eq!(result, expected_seconds, "for end date {end_date}")
    }

    #[test]
    fn test_considers_vacation_days() {
        // Monday
        let start_date = date(2024, 1, 1);

        let config = make_config(WorkingHours {
            monday: Duration::from_secs(8 * SECONDS_PER_HOUR),
            tuesday: Duration::from_secs(6 * SECONDS_PER_HOUR),
            wednesday: Duration::from_secs(5 * SECONDS_PER_HOUR),
            thursday: Duration::ZERO,
            friday: Duration::ZERO,
            saturday: Duration::ZERO,
            sunday: Duration::ZERO,
        });

        let holidays: Holidays = BTreeMap::new();
        let sick_days: SickDays = BTreeMap::new();
        let vacations: Vacations = BTreeMap::from([(
            date(2024, 1, 3),
            types::VacationEntry {
                description: "Vacation".to_string(),
                portion: types::DayPortion::Full,
            },
        )]);

        let end_date = date(2024, 1, 16);

        let expected_seconds = (8 * 60 * 60 + 6 * 60 * 60 + 5 * 60 * 60) * 2
            + (8 * 60 * 60 + 6 * 60 * 60)
            - 5 * 60 * 60;

        let timespan = make_timespan(start_date, end_date);
        let result =
            expected_duration(&config, &timespan, &holidays, &sick_days, &vacations).unwrap();
        assert_eq!(result, expected_seconds, "for end date {end_date}")
    }

    #[test]
    fn test_considers_half_vacation_days() {
        // Monday
        let start_date = date(2024, 1, 1);

        let config = make_config(WorkingHours {
            monday: Duration::from_secs(8 * SECONDS_PER_HOUR),
            tuesday: Duration::from_secs(6 * SECONDS_PER_HOUR),
            wednesday: Duration::from_secs(5 * SECONDS_PER_HOUR),
            thursday: Duration::ZERO,
            friday: Duration::ZERO,
            saturday: Duration::ZERO,
            sunday: Duration::ZERO,
        });

        let holidays: Holidays = BTreeMap::new();
        let sick_days: SickDays = BTreeMap::new();
        let vacations: Vacations = BTreeMap::from([(
            date(2024, 1, 3),
            types::VacationEntry {
                description: "Vacation".to_string(),
                portion: types::DayPortion::Half,
            },
        )]);

        let end_date = date(2024, 1, 16);

        let expected_seconds = (8 * 60 * 60 + 6 * 60 * 60 + 5 * 60 * 60) * 2
            + (8 * 60 * 60 + 6 * 60 * 60)
            - (5 * 60 * 60 / 2);

        let timespan = make_timespan(start_date, end_date);
        let result =
            expected_duration(&config, &timespan, &holidays, &sick_days, &vacations).unwrap();
        assert_eq!(result, expected_seconds, "for end date {end_date}")
    }

    #[test]
    fn test_does_not_consider_vacation_days_out_of_range() {
        // Monday
        let start_date = date(2024, 1, 1);

        let config = make_config(WorkingHours {
            monday: Duration::from_secs(8 * SECONDS_PER_HOUR),
            tuesday: Duration::from_secs(6 * SECONDS_PER_HOUR),
            wednesday: Duration::from_secs(5 * SECONDS_PER_HOUR),
            thursday: Duration::ZERO,
            friday: Duration::ZERO,
            saturday: Duration::ZERO,
            sunday: Duration::ZERO,
        });

        let holidays: Holidays = BTreeMap::new();
        let sick_days: SickDays = BTreeMap::new();
        let vacations: Vacations = BTreeMap::from([(
            date(2024, 1, 17),
            types::VacationEntry {
                description: "Vacation".to_string(),
                portion: types::DayPortion::Full,
            },
        )]);

        let end_date = date(2024, 1, 16);

        let expected_seconds =
            (8 * 60 * 60 + 6 * 60 * 60 + 5 * 60 * 60) * 2 + (8 * 60 * 60 + 6 * 60 * 60);

        let timespan = make_timespan(start_date, end_date);
        let result =
            expected_duration(&config, &timespan, &holidays, &sick_days, &vacations).unwrap();
        assert_eq!(result, expected_seconds, "for end date {end_date}")
    }

    #[test]
    fn test_considers_holidays() {
        // Monday
        let start_date = date(2024, 1, 1);

        let config = make_config(WorkingHours {
            monday: Duration::from_secs(8 * SECONDS_PER_HOUR),
            tuesday: Duration::from_secs(6 * SECONDS_PER_HOUR),
            wednesday: Duration::from_secs(5 * SECONDS_PER_HOUR),
            thursday: Duration::ZERO,
            friday: Duration::ZERO,
            saturday: Duration::ZERO,
            sunday: Duration::ZERO,
        });

        let holidays: Holidays = BTreeMap::from([(
            date(2024, 1, 3),
            types::HolidayEntry {
                description: "Holiday".to_string(),
                portion: types::DayPortion::Full,
            },
        )]);
        let sick_days: SickDays = BTreeMap::new();
        let vacations: Vacations = BTreeMap::new();

        let end_date = date(2024, 1, 16);

        let expected_seconds = (8 * 60 * 60 + 6 * 60 * 60 + 5 * 60 * 60) * 2
            + (8 * 60 * 60 + 6 * 60 * 60)
            - 5 * 60 * 60;

        let timespan = make_timespan(start_date, end_date);
        let result =
            expected_duration(&config, &timespan, &holidays, &sick_days, &vacations).unwrap();
        assert_eq!(result, expected_seconds, "for end date {end_date}")
    }

    #[test]
    fn test_considers_sickdays() {
        // Monday
        let start_date = date(2024, 1, 1);

        let config = make_config(WorkingHours {
            monday: Duration::from_secs(8 * SECONDS_PER_HOUR),
            tuesday: Duration::from_secs(6 * SECONDS_PER_HOUR),
            wednesday: Duration::from_secs(5 * SECONDS_PER_HOUR),
            thursday: Duration::ZERO,
            friday: Duration::ZERO,
            saturday: Duration::ZERO,
            sunday: Duration::ZERO,
        });

        let holidays: Holidays = BTreeMap::new();
        let sick_days: SickDays = BTreeMap::from([(
            date(2024, 1, 3),
            types::SickDayEntry {
                description: "Sick Day".to_string(),
                portion: types::DayPortion::Full,
            },
        )]);
        let vacations: Vacations = BTreeMap::new();

        let end_date = date(2024, 1, 16);

        let expected_seconds = (8 * 60 * 60 + 6 * 60 * 60 + 5 * 60 * 60) * 2
            + (8 * 60 * 60 + 6 * 60 * 60)
            - 5 * 60 * 60;

        let timespan = make_timespan(start_date, end_date);
        let result =
            expected_duration(&config, &timespan, &holidays, &sick_days, &vacations).unwrap();
        assert_eq!(result, expected_seconds, "for end date {end_date}")
    }

    #[test]
    fn test_does_not_count_days_off_twice() {
        // Monday
        let start_date = date(2024, 1, 1);

        let config = make_config(WorkingHours {
            monday: Duration::from_secs(8 * SECONDS_PER_HOUR),
            tuesday: Duration::from_secs(6 * SECONDS_PER_HOUR),
            wednesday: Duration::from_secs(5 * SECONDS_PER_HOUR),
            thursday: Duration::ZERO,
            friday: Duration::ZERO,
            saturday: Duration::ZERO,
            sunday: Duration::ZERO,
        });

        let holidays: Holidays = BTreeMap::from([(
            date(2024, 1, 3),
            types::HolidayEntry {
                description: "Holiday".to_string(),
                portion: types::DayPortion::Full,
            },
        )]);
        let sick_days: SickDays = BTreeMap::from([(
            date(2024, 1, 3),
            types::SickDayEntry {
                description: "Sick".to_string(),
                portion: types::DayPortion::Full,
            },
        )]);
        let vacations: Vacations = BTreeMap::from([(
            date(2024, 1, 3),
            types::VacationEntry {
                description: "Vacation".to_string(),
                portion: types::DayPortion::Full,
            },
        )]);

        let end_date = date(2024, 1, 16);

        let expected_seconds = (8 * 60 * 60 + 6 * 60 * 60 + 5 * 60 * 60) * 2
            + (8 * 60 * 60 + 6 * 60 * 60)
            - 5 * 60 * 60;

        let timespan = make_timespan(start_date, end_date);
        let result =
            expected_duration(&config, &timespan, &holidays, &sick_days, &vacations).unwrap();
        assert_eq!(result, expected_seconds, "for end date {end_date}")
    }

    #[test]
    fn test_adds_up_half_days_off_on_one_date() {
        // Monday
        let start_date = date(2024, 1, 1);

        let config = make_config(WorkingHours {
            monday: Duration::from_secs(8 * SECONDS_PER_HOUR),
            tuesday: Duration::from_secs(6 * SECONDS_PER_HOUR),
            wednesday: Duration::from_secs(5 * SECONDS_PER_HOUR),
            thursday: Duration::ZERO,
            friday: Duration::ZERO,
            saturday: Duration::ZERO,
            sunday: Duration::ZERO,
        });

        let holidays: Holidays = BTreeMap::from([(
            date(2024, 1, 3),
            types::HolidayEntry {
                description: "Holiday".to_string(),
                portion: types::DayPortion::Half,
            },
        )]);
        let sick_days: SickDays = BTreeMap::new();
        let vacations: Vacations = BTreeMap::from([(
            date(2024, 1, 3),
            types::VacationEntry {
                description: "Vacation".to_string(),
                portion: types::DayPortion::Half,
            },
        )]);

        let end_date = date(2024, 1, 3);
        let expected_seconds = 8 * 60 * 60 + 6 * 60 * 60;

        let timespan = make_timespan(start_date, end_date);
        let result =
            expected_duration(&config, &timespan, &holidays, &sick_days, &vacations).unwrap();
        assert_eq!(result, expected_seconds, "for end date {end_date}")
    }

    #[test]
    fn test_does_not_owe_the_date_of_an_end_at_midnight() {
        let config = make_config(WorkingHours::default());

        let holidays: Holidays = BTreeMap::new();
        let sick_days: SickDays = BTreeMap::new();
        let vacations: Vacations = BTreeMap::new();

        // Monday to Wednesday at midnight.
        let timespan = Timespan {
            from: timestamp("2024-01-01 00:00"),
            to: timestamp("2024-01-03 00:00"),
        };

        let result =
            expected_duration(&config, &timespan, &holidays, &sick_days, &vacations).unwrap();
        assert_eq!(result, 2 * 8 * 60 * 60);
    }

    #[test]
    fn test_owes_nothing_for_a_reversed_timespan() {
        let config = make_config(WorkingHours {
            monday: Duration::from_secs(8 * SECONDS_PER_HOUR),
            tuesday: Duration::from_secs(8 * SECONDS_PER_HOUR),
            wednesday: Duration::from_secs(8 * SECONDS_PER_HOUR),
            thursday: Duration::from_secs(8 * SECONDS_PER_HOUR),
            friday: Duration::from_secs(8 * SECONDS_PER_HOUR),
            saturday: Duration::ZERO,
            sunday: Duration::ZERO,
        });

        let holidays: Holidays = BTreeMap::new();
        let sick_days: SickDays = BTreeMap::new();
        let vacations: Vacations = BTreeMap::new();

        let timespan = make_timespan(date(2024, 1, 16), date(2024, 1, 1));
        let result = expected_duration(&config, &timespan, &holidays, &sick_days, &vacations);

        assert_eq!(result.unwrap(), 0);
    }

    fn timestamp(datetime: &str) -> i64 {
        NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M")
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp()
    }

    fn this_week() -> Period {
        Period {
            week: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_ends_the_range_at_the_current_time() {
        // A Wednesday at 10:00.
        let now = timestamp("2026-10-07 10:00");
        let timespan = resolve_timespan(
            &BalanceArgs {
                period: this_week(),
                ..Default::default()
            },
            now,
            &[],
        )
        .unwrap();

        assert_eq!(timespan.from, timestamp("2026-10-05 00:00"));
        assert_eq!(timespan.to, now);
    }

    #[test]
    fn test_ends_the_range_at_the_start_of_today_with_exclude_today() {
        // A Wednesday at 10:00.
        let now = timestamp("2026-10-07 10:00");
        let timespan = resolve_timespan(
            &BalanceArgs {
                period: this_week(),
                exclude_today: true,
                ..Default::default()
            },
            now,
            &[],
        )
        .unwrap();

        assert_eq!(timespan.from, timestamp("2026-10-05 00:00"));
        assert_eq!(timespan.to, timestamp("2026-10-07 00:00"));
    }

    #[test]
    fn test_starts_an_empty_range_with_exclude_today_if_all_frames_are_from_today() {
        // A Wednesday at 10:00.
        let now = timestamp("2026-10-07 10:00");
        let frames = [Frame {
            start_time: timestamp("2026-10-07 08:00"),
            end_time: timestamp("2026-10-07 09:00"),
            project: "work".to_string(),
            tags: vec![],
            updated_at: timestamp("2026-10-07 09:00"),
        }];
        let args = BalanceArgs {
            exclude_today: true,
            ..Default::default()
        };
        let timespan = resolve_timespan(&args, now, &frames).unwrap();

        assert_eq!(timespan.from, timestamp("2026-10-07 00:00"));
        assert_eq!(timespan.to, timestamp("2026-10-07 00:00"));
    }

    #[test]
    fn test_ends_a_range_set_by_from_at_the_start_of_today_with_exclude_today() {
        let now = timestamp("2026-10-07 10:00");
        let args = BalanceArgs {
            from: Some(date(2026, 10, 6)),
            exclude_today: true,
            ..Default::default()
        };
        let timespan = resolve_timespan(&args, now, &[]).unwrap();

        assert_eq!(timespan.from, timestamp("2026-10-06 00:00"));
        assert_eq!(timespan.to, timestamp("2026-10-07 00:00"));
    }

    #[test]
    fn test_includes_the_to_date() {
        let now = timestamp("2026-10-17 10:00");
        let args = BalanceArgs {
            from: Some(date(2026, 10, 5)),
            to: Some(date(2026, 10, 6)),
            ..Default::default()
        };
        let timespan = resolve_timespan(&args, now, &[]).unwrap();

        assert_eq!(timespan.from, timestamp("2026-10-05 00:00"));
        assert_eq!(timespan.to, timestamp("2026-10-07 00:00"));
    }

    #[test]
    fn test_starts_at_midnight_of_the_first_frame() {
        let now = timestamp("2026-10-17 10:00");
        let frames = [Frame {
            start_time: timestamp("2026-10-06 14:00"),
            end_time: timestamp("2026-10-06 15:00"),
            project: "work".to_string(),
            tags: vec![],
            updated_at: timestamp("2026-10-06 15:00"),
        }];
        let args = BalanceArgs {
            ..Default::default()
        };
        let timespan = resolve_timespan(&args, now, &frames).unwrap();

        assert_eq!(timespan.from, timestamp("2026-10-06 00:00"));
        assert_eq!(timespan.to, now);
    }
}
