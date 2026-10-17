// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::output::{DisplayOutput, print_output};
use crate::persistence::{
    load_holidays, load_sick_days, load_vacations, save_holidays, save_sick_days, save_vacations,
};
use crate::types::{DayOff, DayOffCalendar, DayOffEntry, DayPortion};
use crate::{Format, HolidayCommands, SickDayCommands, VacationCommands};
use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::path::Path;
use tabled::{Table, settings::Style};

pub(crate) struct Kind {
    title: &'static str,
    noun: &'static str,
    plural: &'static str,
    key: &'static str,
    list_key: &'static str,
    load: fn(&Path) -> Result<DayOffCalendar>,
    save: fn(&Path, &DayOffCalendar) -> Result<()>,
}

pub(crate) const HOLIDAY: Kind = Kind {
    title: "Holiday",
    noun: "holiday",
    plural: "holidays",
    key: "holiday",
    list_key: "holidays",
    load: load_holidays,
    save: save_holidays,
};

pub(crate) const SICK_DAY: Kind = Kind {
    title: "Sick day",
    noun: "sick day",
    plural: "sick days",
    key: "sick_day",
    list_key: "sick_days",
    load: load_sick_days,
    save: save_sick_days,
};

pub(crate) const VACATION: Kind = Kind {
    title: "Vacation",
    noun: "vacation",
    plural: "vacations",
    key: "vacation",
    list_key: "vacations",
    load: load_vacations,
    save: save_vacations,
};

pub(crate) enum Command {
    Add {
        date: NaiveDate,
        description: String,
        portion: Option<DayPortion>,
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
    },
}

macro_rules! command_from {
    ($commands:ident) => {
        impl From<&$commands> for Command {
            fn from(command: &$commands) -> Self {
                match command {
                    $commands::Add {
                        date,
                        description,
                        portion,
                    } => Command::Add {
                        date: *date,
                        description: description.clone(),
                        portion: portion.clone(),
                    },
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
                    $commands::Remove { date } => Command::Remove { date: *date },
                }
            }
        }
    };
}

command_from!(HolidayCommands);
command_from!(SickDayCommands);
command_from!(VacationCommands);

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
            description,
            portion,
        } => {
            if calendar.contains_key(&date) {
                anyhow::bail!("A {} already exists on {date}.", kind.noun);
            }
            let day_off =
                write_entry(kind, &mut calendar, date, description, portion, config_path)?;
            print_entry(kind, Action::Added, day_off, format)
        }

        Command::Edit {
            date,
            description,
            portion,
        } => {
            if !calendar.contains_key(&date) {
                anyhow::bail!("No {} found on {date}.", kind.noun);
            }
            let day_off =
                write_entry(kind, &mut calendar, date, description, portion, config_path)?;
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

        Command::Remove { date } => {
            let Some(entry) = calendar.remove(&date) else {
                anyhow::bail!("No {} found on {date}.", kind.noun);
            };
            (kind.save)(config_path, &calendar)?;
            print_entry(kind, Action::Removed, day_off(date, entry), format)
        }
    }
}

fn write_entry(
    kind: &Kind,
    calendar: &mut DayOffCalendar,
    date: NaiveDate,
    description: String,
    portion: Option<DayPortion>,
    config_path: &Path,
) -> Result<DayOff> {
    let entry = DayOffEntry {
        description,
        portion: portion.unwrap_or(DayPortion::Full),
    };
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
