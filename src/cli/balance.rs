// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::formatting::{format_duration, format_timerange};
use crate::output::{DisplayOutput, print_output};
use crate::persistence::{
    load_config, load_frames, load_holidays, load_sick_days, load_state, load_vacations,
};
use crate::types::{
    Config, DayPortion, Frame, Frames, Holidays, SickDays, Timespan, Vacations, WorkingHours,
};
use crate::{BalanceArgs, Format};
use chrono::{Datelike, Local, NaiveDate, TimeZone, Utc};
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
        && from >= to
    {
        anyhow::bail!("'to' must be after 'from'");
    }

    let now = Utc::now().timestamp();

    let config = load_config(config_path)?;
    let mut frames = load_frames(config_path)?;
    let state = load_state(config_path)?;
    let holidays = load_holidays(config_path)?;
    let sick_days = load_sick_days(config_path)?;
    let vacations = load_vacations(config_path)?;

    if let Some(current_frame) = &state.current_frame {
        frames.frames.push(Frame {
            start_time: current_frame.start_time,
            end_time: now,
            project: current_frame.project.clone(),
            tags: current_frame.tags.clone(),
            updated_at: now,
        });
    }

    let timespan = resolve_timespan(args, now, &frames.frames);

    if timespan.from > timespan.to {
        frames.frames.clear();
    } else {
        frames
            .filter_by_start_time(timespan.from)
            .filter_by_end_time(timespan.to);
    }

    let expected_working_seconds =
        expected_duration(&config, &timespan, &holidays, &sick_days, &vacations);
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

fn resolve_timespan(args: &BalanceArgs, now: i64, frames: &[Frame]) -> Timespan {
    let local_now = Local.timestamp_opt(now, 0).unwrap();

    let from = if args.day {
        local_now.date_naive().and_hms_opt(0, 0, 0).unwrap()
    } else if args.week {
        let weekday = local_now.weekday().num_days_from_monday();
        (local_now.date_naive() - chrono::Duration::days(weekday.into()))
            .and_hms_opt(0, 0, 0)
            .unwrap()
    } else if args.month {
        local_now
            .date_naive()
            .with_day(1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    } else if args.year {
        local_now
            .date_naive()
            .with_month(1)
            .unwrap()
            .with_day(1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    } else {
        let from_ts = args
            .from
            .map(|dt| dt.with_timezone(&Utc).timestamp())
            .or_else(|| frames.first().map(|f| f.start_time))
            .unwrap_or(0);
        return Timespan {
            from: from_ts,
            to: args
                .to
                .map(|dt| dt.with_timezone(&Utc).timestamp())
                .unwrap_or(now),
        };
    };

    Timespan {
        from: from
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc)
            .timestamp(),
        to: now,
    }
}

fn expected_duration(
    config: &Config,
    timespan: &Timespan,
    holidays: &Holidays,
    sick_days: &SickDays,
    vacations: &Vacations,
) -> i64 {
    let start_date = timestamp_to_local_date(timespan.from);
    let end_date = timestamp_to_local_date(timespan.to);

    let (full_weeks, remaining_days) = calculate_weeks_and_days(timespan);
    let working_duration_per_week = config.working_hours.total_weekly_duration();
    let full_week_duration = working_duration_per_week
        .checked_mul(full_weeks as u32)
        .unwrap();

    let remaining_days_duration =
        calculate_remaining_days_hours(remaining_days, end_date, &config.working_hours);

    let day_offs = merge_day_offs_in_range(vacations, holidays, sick_days, start_date, end_date);

    let mut total_duration = full_week_duration + remaining_days_duration;
    total_duration = subtract_day_offs(total_duration, &day_offs, config);

    total_duration.as_secs().try_into().unwrap()
}

fn calculate_weeks_and_days(timespan: &Timespan) -> (i64, i64) {
    let from_date = timestamp_to_local_date(timespan.from);
    let to_date = timestamp_to_local_date(timespan.to);

    let days_diff = (to_date - from_date).num_days() + 1;
    let full_weeks = days_diff / 7;
    let remaining_days = days_diff % 7;
    (full_weeks, remaining_days)
}

fn calculate_remaining_days_hours(
    remaining_days: i64,
    end_date: NaiveDate,
    working_days: &WorkingHours,
) -> Duration {
    if remaining_days == 0 {
        return Duration::ZERO;
    }

    let start_date = end_date - chrono::Duration::days(remaining_days - 1);
    let mut total = Duration::ZERO;

    for offset in 0..remaining_days {
        let current_date = start_date + chrono::Duration::days(offset);
        total += get_hours_for_day(current_date, working_days);
    }

    total
}

fn get_hours_for_day(date: NaiveDate, working_hours: &WorkingHours) -> Duration {
    match date.weekday() {
        chrono::Weekday::Mon => working_hours.monday,
        chrono::Weekday::Tue => working_hours.tuesday,
        chrono::Weekday::Wed => working_hours.wednesday,
        chrono::Weekday::Thu => working_hours.thursday,
        chrono::Weekday::Fri => working_hours.friday,
        chrono::Weekday::Sat => working_hours.saturday,
        chrono::Weekday::Sun => working_hours.sunday,
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
        insert_or_upgrade_portion(&mut combined, date, &entry.portion);
    }

    for (&date, entry) in holidays.range(start_date..=end_date) {
        insert_or_upgrade_portion(&mut combined, date, &entry.portion);
    }

    for (&date, entry) in sick_days.range(start_date..=end_date) {
        insert_or_upgrade_portion(&mut combined, date, &entry.portion);
    }

    combined
}

fn insert_or_upgrade_portion(
    map: &mut BTreeMap<NaiveDate, DayPortion>,
    date: NaiveDate,
    portion: &DayPortion,
) {
    map.entry(date)
        .and_modify(|existing| {
            if portion_order(portion) > portion_order(existing) {
                *existing = portion.clone();
            }
        })
        .or_insert_with(|| portion.clone());
}

fn portion_order(portion: &DayPortion) -> u8 {
    match portion {
        DayPortion::Full => 2,
        DayPortion::Half => 1,
    }
}

fn subtract_day_offs(
    mut duration: Duration,
    day_offs: &BTreeMap<NaiveDate, DayPortion>,
    config: &Config,
) -> Duration {
    for (&date, portion) in day_offs {
        let daily_duration = get_hours_for_day(date, &config.working_hours);
        let subtract = match portion {
            DayPortion::Full => daily_duration,
            DayPortion::Half => daily_duration / 2,
        };
        if duration >= subtract {
            duration -= subtract;
        } else {
            duration = Duration::ZERO;
        }
    }
    duration
}

fn timestamp_to_local_date(secs: i64) -> NaiveDate {
    Local.timestamp_opt(secs, 0).unwrap().date_naive()
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
    use crate::types;

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
                .from_local_datetime(&to_date.and_hms_opt(0, 0, 0).unwrap())
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
            let result = expected_duration(&config, &timespan, &holidays, &sick_days, &vacations);
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
        let result = expected_duration(&config, &timespan, &holidays, &sick_days, &vacations);
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
        let result = expected_duration(&config, &timespan, &holidays, &sick_days, &vacations);
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
        let result = expected_duration(&config, &timespan, &holidays, &sick_days, &vacations);
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
        let result = expected_duration(&config, &timespan, &holidays, &sick_days, &vacations);
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
        let result = expected_duration(&config, &timespan, &holidays, &sick_days, &vacations);
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
        let result = expected_duration(&config, &timespan, &holidays, &sick_days, &vacations);
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
        let result = expected_duration(&config, &timespan, &holidays, &sick_days, &vacations);
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
        let result = expected_duration(&config, &timespan, &holidays, &sick_days, &vacations);
        assert_eq!(result, expected_seconds, "for end date {end_date}")
    }
}
