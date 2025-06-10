// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::persistence::{load_sick_days, save_sick_days};
use crate::types::{DayPortion, SickDay, SickDayEntry};
use crate::{Format, SickDayArgs, SickDayCommands};
use chrono::Datelike;
use serde::Serialize;
use std::path::Path;
use tabled::{settings::Style, Table};

#[derive(Serialize)]
struct AddOutput {
    sick_day: SickDay,
}

impl AddOutput {
    fn to_text(&self) -> String {
        format!(
            "Sick day '{}' added on {}.",
            self.sick_day.description,
            self.sick_day.date.format("%Y-%m-%d"),
        )
    }
}

#[derive(Serialize)]
struct EditOutput {
    sick_day: SickDay,
}

impl EditOutput {
    fn to_text(&self) -> String {
        format!(
            "Updated sick day '{}' on {}.",
            self.sick_day.description,
            self.sick_day.date.format("%Y-%m-%d"),
        )
    }
}

#[derive(Serialize)]
struct ListOutput {
    sick_days: Vec<SickDay>,
    filters: Filters,
}

#[derive(Serialize)]
struct Filters {
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<i32>,
}

impl ListOutput {
    fn to_text(&self) -> String {
        if self.sick_days.is_empty() {
            match self.filters.year {
                Some(y) => format!("No sick days found for {}.", y),
                None => "No sick days found.".to_string(),
            }
        } else {
            let mut table = Table::new(self.sick_days.clone());
            table.with(Style::sharp()).to_string()
        }
    }
}

#[derive(Serialize)]
struct RemoveOutput {
    sick_day: SickDay,
}

impl RemoveOutput {
    fn to_text(&self) -> String {
        format!(
            "Removed sick day '{}' on {}.",
            self.sick_day.description,
            self.sick_day.date.format("%Y-%m-%d"),
        )
    }
}

pub fn run_sick_day(args: &SickDayArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    let mut sick_days = load_sick_days(config_path)?;

    match &args.command {
        SickDayCommands::Add {
            date,
            description,
            portion,
        } => {
            if sick_days.contains_key(date) {
                anyhow::bail!("A sick day already exists on {}", date);
            }

            let entry = SickDayEntry {
                description: description.clone(),
                portion: portion.clone().unwrap_or(DayPortion::Full),
            };

            sick_days.insert(*date, entry.clone());
            save_sick_days(config_path, &sick_days)?;

            let output = AddOutput {
                sick_day: SickDay {
                    date: *date,
                    description: entry.description,
                    portion: entry.portion,
                },
            };

            let output_string = match format {
                Format::Json => serde_json::to_string_pretty(&output)?,
                Format::Text => output.to_text(),
            };

            println!("{}", output_string);
        }

        SickDayCommands::Edit {
            date,
            description,
            portion,
        } => {
            if !sick_days.contains_key(date) {
                anyhow::bail!("No sick day found on {}", date);
            }

            let entry = SickDayEntry {
                description: description.clone(),
                portion: portion.clone().unwrap_or(DayPortion::Full),
            };

            sick_days.insert(*date, entry.clone());
            save_sick_days(config_path, &sick_days)?;

            let output = EditOutput {
                sick_day: SickDay {
                    date: *date,
                    description: entry.description,
                    portion: entry.portion,
                },
            };

            let output_string = match format {
                Format::Json => serde_json::to_string_pretty(&output)?,
                Format::Text => output.to_text(),
            };

            println!("{}", output_string);
        }

        SickDayCommands::List { year } => {
            let filtered: Vec<SickDay> = sick_days
                .iter()
                .filter(|(date, _)| year.is_none() || date.year() == year.unwrap())
                .map(|(date, entry)| SickDay {
                    date: *date,
                    description: entry.description.clone(),
                    portion: entry.portion.clone(),
                })
                .collect();

            let output = ListOutput {
                sick_days: filtered,
                filters: Filters { year: *year },
            };

            let output_string = match format {
                Format::Json => serde_json::to_string_pretty(&output)?,
                Format::Text => output.to_text(),
            };

            println!("{}", output_string);
        }

        SickDayCommands::Remove { date } => {
            let entry = match sick_days.remove(date) {
                Some(entry) => entry,
                None => anyhow::bail!("No sick day found on {}.", date),
            };

            save_sick_days(config_path, &sick_days)?;

            let output = RemoveOutput {
                sick_day: SickDay {
                    date: *date,
                    description: entry.description,
                    portion: entry.portion,
                },
            };

            let output_string = match format {
                Format::Json => serde_json::to_string_pretty(&output)?,
                Format::Text => output.to_text(),
            };

            println!("{}", output_string);
        }
    };

    Ok(())
}
