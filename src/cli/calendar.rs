// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::cli::days_off::halves_to_work;
use crate::output::{DisplayOutput, print_output};
use crate::persistence::{
    load_config, load_holidays, load_sick_days, load_vacations, save_holidays, save_sick_days,
    save_vacations,
};
use crate::types::{DayOff, DayOffCalendar, DayOffEntry, DayPortion};
use crate::{
    DayOffAddArgs, DayOffRemoveArgs, Format, HolidayAddArgs, HolidayCommands, HolidayRemoveArgs,
    SickDayCommands, VacationCommands, parse_date,
};
use anyhow::Result;
use chrono::{Datelike, Months, NaiveDate};
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::path::Path;
use tabled::{Table, settings::Style};

pub(crate) struct Kind {
    title: &'static str,
    noun: &'static str,
    plural: &'static str,
    day: &'static str,
    command: &'static str,
    key: &'static str,
    list_key: &'static str,
    default_description: &'static str,
    load: fn(&Path) -> Result<DayOffCalendar>,
    save: fn(&Path, &DayOffCalendar) -> Result<()>,
    other: Option<&'static Kind>,
}

pub(crate) static HOLIDAY: Kind = Kind {
    title: "Holiday",
    noun: "holiday",
    day: "holiday",
    command: "holiday",
    plural: "holidays",
    key: "holiday",
    list_key: "holidays",
    default_description: "Holiday",
    load: load_holidays,
    save: save_holidays,
    other: None,
};

pub(crate) static SICK_DAY: Kind = Kind {
    title: "Sick day",
    noun: "sick day",
    day: "sick day",
    command: "sickday",
    plural: "sick days",
    key: "sick_day",
    list_key: "sick_days",
    default_description: "Sick",
    load: load_sick_days,
    save: save_sick_days,
    other: Some(&VACATION),
};

pub(crate) static VACATION: Kind = Kind {
    title: "Vacation",
    noun: "vacation",
    day: "vacation day",
    command: "vacation",
    plural: "vacations",
    key: "vacation",
    list_key: "vacations",
    default_description: "Vacation",
    load: load_vacations,
    save: save_vacations,
    other: Some(&SICK_DAY),
};

pub(crate) enum Command {
    Add {
        date: NaiveDate,
        last_day: Option<NaiveDate>,
        description: Option<String>,
        portion: DayPortion,
    },
    Edit {
        date: NaiveDate,
        description: String,
        portion: Option<DayPortion>,
    },
    List {
        year: Option<i32>,
    },
    Remove {
        date: NaiveDate,
        last_day: Option<NaiveDate>,
    },
}

macro_rules! command_from {
    ($commands:ident) => {
        impl TryFrom<&$commands> for Command {
            type Error = anyhow::Error;

            fn try_from(command: &$commands) -> Result<Self> {
                Ok(match command {
                    $commands::Add(args) => args.command()?,
                    $commands::Edit {
                        date,
                        description,
                        portion,
                    } => Command::Edit {
                        date: *date,
                        description: description.clone(),
                        portion: portion.clone(),
                    },
                    $commands::List { year } => Command::List { year: *year },
                    $commands::Remove(args) => args.command(),
                })
            }
        }
    };
}

command_from!(HolidayCommands);
command_from!(SickDayCommands);
command_from!(VacationCommands);

impl HolidayAddArgs {
    fn command(&self) -> Result<Command> {
        Ok(Command::Add {
            date: self.date,
            last_day: None,
            description: self.description.clone(),
            portion: self.portion.clone(),
        })
    }
}

impl DayOffAddArgs {
    /// second argument is last day of a range if it looks like a date and
    /// description otherwise
    fn command(&self) -> Result<Command> {
        let (last_day, description) = match &self.last_day_or_description {
            Some(second) if looks_like_date(second) => {
                (Some(parse_date(second)?), self.description.clone())
            }
            Some(second) => {
                match &self.description {
                    Some(extra) if looks_like_date(extra) => anyhow::bail!(
                        "Too many arguments. To specify a range, use the argument order \
                         <start-date> <end-date> <description>."
                    ),
                    Some(_) => {
                        anyhow::bail!("Too many arguments. Quote descriptions that contain spaces.")
                    }
                    None => {}
                }
                (None, Some(second.clone()))
            }
            None => (None, None),
        };

        Ok(Command::Add {
            date: self.date,
            last_day,
            description,
            portion: self.portion.clone(),
        })
    }
}

impl HolidayRemoveArgs {
    fn command(&self) -> Command {
        Command::Remove {
            date: self.date,
            last_day: None,
        }
    }
}

impl DayOffRemoveArgs {
    fn command(&self) -> Command {
        Command::Remove {
            date: self.date,
            last_day: self.end_date,
        }
    }
}

fn looks_like_date(input: &str) -> bool {
    input.len() == 10
        && input.char_indices().all(|(index, char)| match index {
            4 | 7 => char == '-',
            _ => char.is_ascii_digit(),
        })
}

enum Action {
    Added,
    Updated,
    Removed,
}

struct EntryOutput<'a> {
    kind: &'a Kind,
    action: Action,
    day_off: DayOff,
}

impl Serialize for EntryOutput<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(self.kind.key, &self.day_off)?;
        map.end()
    }
}

impl DisplayOutput for EntryOutput<'_> {
    fn to_text(&self) -> String {
        let noun = self.kind.noun;
        let description = &self.day_off.description;
        let date = self.day_off.date.format("%Y-%m-%d");

        match self.action {
            Action::Added => format!("{} '{description}' added on {date}.", self.kind.title),
            Action::Updated => format!("Updated {noun} '{description}' on {date}."),
            Action::Removed => format!("Removed {noun} '{description}' on {date}."),
        }
    }
}

struct ListOutput<'a> {
    kind: &'a Kind,
    days_off: Vec<DayOff>,
    year: Option<i32>,
}

#[derive(serde::Serialize)]
struct Filters {
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<i32>,
}

impl Serialize for ListOutput<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry(self.kind.list_key, &self.days_off)?;
        map.serialize_entry("filters", &Filters { year: self.year })?;
        map.end()
    }
}

impl DisplayOutput for ListOutput<'_> {
    fn to_text(&self) -> String {
        if self.days_off.is_empty() {
            match self.year {
                Some(year) => format!("No {} found for {year}.", self.kind.plural),
                None => format!("No {} found.", self.kind.plural),
            }
        } else {
            let mut table = Table::new(self.days_off.clone());
            table.with(Style::sharp()).to_string()
        }
    }
}

pub(crate) fn run(
    kind: &Kind,
    command: Command,
    config_path: &Path,
    format: &Format,
) -> Result<()> {
    let mut calendar = (kind.load)(config_path)?;

    match command {
        Command::Add {
            date,
            last_day,
            description,
            portion,
        } => {
            let entry = DayOffEntry {
                description: description.unwrap_or_else(|| kind.default_description.to_string()),
                portion,
            };
            if let Some(last_day) = last_day {
                return add_range(
                    kind,
                    &mut calendar,
                    date,
                    last_day,
                    entry,
                    config_path,
                    format,
                );
            }
            if calendar.contains_key(&date) {
                anyhow::bail!("A {} already exists on {date}.", kind.noun);
            }
            let day_off = write_entry(kind, &mut calendar, date, entry, config_path)?;
            warn_about_overlaps(kind, std::slice::from_ref(&day_off), config_path);
            print_entry(kind, Action::Added, day_off, format)
        }

        Command::Edit {
            date,
            description,
            portion,
        } => {
            let Some(existing) = calendar.get(&date) else {
                anyhow::bail!("No {} found on {date}.", kind.noun);
            };
            let entry = DayOffEntry {
                description,
                portion: portion.unwrap_or_else(|| existing.portion.clone()),
            };
            let day_off = write_entry(kind, &mut calendar, date, entry, config_path)?;
            print_entry(kind, Action::Updated, day_off, format)
        }

        Command::List { year } => {
            let days_off = calendar
                .iter()
                .filter(|(date, _)| year.is_none_or(|year| date.year() == year))
                .map(|(date, entry)| day_off(*date, entry.clone()))
                .collect();

            print_output(
                &ListOutput {
                    kind,
                    days_off,
                    year,
                },
                format,
            )
        }

        Command::Remove {
            date,
            last_day: Some(last_day),
        } => remove_range(kind, &mut calendar, date, last_day, config_path, format),

        Command::Remove {
            date,
            last_day: None,
        } => {
            let Some(entry) = calendar.remove(&date) else {
                anyhow::bail!("No {} found on {date}.", kind.noun);
            };
            (kind.save)(config_path, &calendar)?;
            print_entry(kind, Action::Removed, day_off(date, entry), format)
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum SkipReason {
    NoWorkingHours,
    Holiday,
    Existing,
}

#[derive(serde::Serialize)]
struct Skipped {
    date: NaiveDate,
    reason: SkipReason,
}

struct RangeOutput<'a> {
    kind: &'a Kind,
    added: Vec<DayOff>,
    skipped: Vec<Skipped>,
}

impl Serialize for RangeOutput<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry(self.kind.list_key, &self.added)?;
        map.serialize_entry("skipped", &self.skipped)?;
        map.end()
    }
}

impl DisplayOutput for RangeOutput<'_> {
    fn to_text(&self) -> String {
        let description = self
            .added
            .first()
            .map_or("", |day_off| day_off.description.as_str());
        let added = join(self.added.iter().map(|day_off| day_off.date.to_string()));
        let mut text = format!("{} '{description}' added on {added}.", self.kind.title);

        if !self.skipped.is_empty() {
            text.push_str(&format!(
                "\nSkipped {}.",
                describe_skipped(self.kind, &self.skipped)
            ));
        }

        text
    }
}

fn describe_skipped(kind: &Kind, skipped: &[Skipped]) -> String {
    join(skipped.iter().map(|skipped| {
        let reason = match skipped.reason {
            SkipReason::NoWorkingHours => "no working hours".to_string(),
            SkipReason::Holiday => "holiday".to_string(),
            SkipReason::Existing => format!("already a {}", kind.noun),
        };
        format!("{} ({reason})", skipped.date)
    }))
}

fn join(items: impl Iterator<Item = String>) -> String {
    items.collect::<Vec<_>>().join(", ")
}

fn add_range(
    kind: &Kind,
    calendar: &mut DayOffCalendar,
    first: NaiveDate,
    last: NaiveDate,
    entry: DayOffEntry,
    config_path: &Path,
    format: &Format,
) -> Result<()> {
    check_range(first, last)?;

    let dates: Vec<NaiveDate> = first.iter_days().take_while(|date| *date <= last).collect();

    let working_hours = load_config(config_path)?.working_hours;
    let holidays = load_holidays(config_path)?;

    let mut added = Vec::new();
    let mut skipped = Vec::new();
    for date in dates {
        let reason = if calendar.contains_key(&date) {
            Some(SkipReason::Existing)
        } else if working_hours.on(date.weekday()).is_zero() {
            Some(SkipReason::NoWorkingHours)
        } else if halves_to_work(date, &working_hours, &holidays) == 0 {
            Some(SkipReason::Holiday)
        } else {
            None
        };

        match reason {
            Some(reason) => skipped.push(Skipped { date, reason }),
            None => {
                calendar.insert(date, entry.clone());
                added.push(day_off(date, entry.clone()));
            }
        }
    }

    if added.is_empty() {
        anyhow::bail!(
            "No date from {first} to {last} was added. Skipped {}.",
            describe_skipped(kind, &skipped)
        );
    }

    (kind.save)(config_path, calendar)?;
    warn_about_overlaps(kind, &added, config_path);
    print_output(
        &RangeOutput {
            kind,
            added,
            skipped,
        },
        format,
    )
}

fn check_range(first: NaiveDate, last: NaiveDate) -> Result<()> {
    if last < first {
        anyhow::bail!("The last day ({last}) is before the first day ({first}).");
    }
    if first
        .checked_add_months(Months::new(12))
        .is_none_or(|year_later| last >= year_later)
    {
        anyhow::bail!("The range from {first} to {last} is longer than a year.");
    }

    Ok(())
}

fn remove_range(
    kind: &Kind,
    calendar: &mut DayOffCalendar,
    first: NaiveDate,
    last: NaiveDate,
    config_path: &Path,
    format: &Format,
) -> Result<()> {
    check_range(first, last)?;

    let dates: Vec<NaiveDate> = calendar
        .range(first..=last)
        .map(|(date, _)| *date)
        .collect();
    if dates.is_empty() {
        anyhow::bail!("No {} found from {first} to {last}.", kind.plural);
    }

    let removed = dates
        .into_iter()
        .filter_map(|date| calendar.remove(&date).map(|entry| day_off(date, entry)))
        .collect();
    (kind.save)(config_path, calendar)?;
    print_output(&RemovedOutput { kind, removed }, format)
}

struct RemovedOutput<'a> {
    kind: &'a Kind,
    removed: Vec<DayOff>,
}

impl Serialize for RemovedOutput<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(self.kind.list_key, &self.removed)?;
        map.end()
    }
}

impl DisplayOutput for RemovedOutput<'_> {
    fn to_text(&self) -> String {
        self.removed
            .iter()
            .map(|day_off| {
                EntryOutput {
                    kind: self.kind,
                    action: Action::Removed,
                    day_off: day_off.clone(),
                }
                .to_text()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn warn_about_overlaps(kind: &Kind, added: &[DayOff], config_path: &Path) {
    let Some(other) = kind.other else {
        return;
    };

    match overlaps(other, added, config_path) {
        Ok(dates) => {
            for date in dates {
                eprintln!(
                    "Warning: {date} already has a {}. To remove it, run `ebb {} remove {date}`.",
                    other.day, other.command
                );
            }
        }
        Err(error) => eprintln!(
            "Warning: Could not check the {} on the same dates: {error:#}",
            other.plural
        ),
    }
}

fn overlaps(other: &Kind, added: &[DayOff], config_path: &Path) -> Result<Vec<NaiveDate>> {
    let other_calendar = (other.load)(config_path)?;
    let working_hours = load_config(config_path)?.working_hours;
    let holidays = load_holidays(config_path)?;

    Ok(added
        .iter()
        .filter_map(|day_off| {
            let entry = other_calendar.get(&day_off.date)?;
            let to_work = halves_to_work(day_off.date, &working_hours, &holidays);
            let counted =
                day_off.portion.halves().min(to_work) + entry.portion.halves().min(to_work);
            (counted > to_work).then_some(day_off.date)
        })
        .collect())
}

fn write_entry(
    kind: &Kind,
    calendar: &mut DayOffCalendar,
    date: NaiveDate,
    entry: DayOffEntry,
    config_path: &Path,
) -> Result<DayOff> {
    calendar.insert(date, entry.clone());
    (kind.save)(config_path, calendar)?;
    Ok(day_off(date, entry))
}

fn print_entry(kind: &Kind, action: Action, day_off: DayOff, format: &Format) -> Result<()> {
    print_output(
        &EntryOutput {
            kind,
            action,
            day_off,
        },
        format,
    )
}

fn day_off(date: NaiveDate, entry: DayOffEntry) -> DayOff {
    DayOff {
        date,
        description: entry.description,
        portion: entry.portion,
    }
}
