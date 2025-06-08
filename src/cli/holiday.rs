// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::persistence::{load_holidays, save_holidays};
use crate::types::{DayPortion, Holiday, HolidayEntry};
use crate::{Format, HolidayArgs, HolidayCommands};
use chrono::Datelike;
use serde::Serialize;
use std::path::Path;
use tabled::{settings::Style, Table};

#[derive(Serialize)]
struct AddOutput {
    holiday: Holiday,
}

impl AddOutput {
    fn to_text(&self) -> String {
        format!(
            "Holiday '{}' added on {}.",
            self.holiday.description,
            self.holiday.date.format("%Y-%m-%d"),
        )
    }
}

#[derive(Serialize)]
struct EditOutput {
    holiday: Holiday,
}

impl EditOutput {
    fn to_text(&self) -> String {
        format!(
            "Updated holiday '{}' on {}.",
            self.holiday.description,
            self.holiday.date.format("%Y-%m-%d"),
        )
    }
}

#[derive(Serialize)]
struct ListOutput {
    holidays: Vec<Holiday>,
    filters: Filters,
}

#[derive(Serialize)]
struct Filters {
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<i32>,
}

impl ListOutput {
    fn to_text(&self) -> String {
        if self.holidays.is_empty() {
            match self.filters.year {
                Some(y) => format!("No holidays found for {}.", y),
                None => "No holidays found.".to_string(),
            }
        } else {
            let mut table = Table::new(self.holidays.clone());
            table.with(Style::sharp()).to_string()
        }
    }
}

#[derive(Serialize)]
struct RemoveOutput {
    holiday: Holiday,
}

impl RemoveOutput {
    fn to_text(&self) -> String {
        format!(
            "Removed holiday '{}' on {}.",
            self.holiday.description,
            self.holiday.date.format("%Y-%m-%d"),
        )
    }
}

pub fn run_holiday(args: &HolidayArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    let mut holidays = load_holidays(config_path)?;

    match &args.command {
        HolidayCommands::Add {
            date,
            description,
            portion,
        } => {
            if holidays.contains_key(date) {
                anyhow::bail!("A holiday already exists on {}", date);
            }

            let entry = HolidayEntry {
                description: description.clone(),
                portion: portion.clone().unwrap_or(DayPortion::Full),
            };

            holidays.insert(*date, entry.clone());
            save_holidays(config_path, &holidays)?;

            let output = AddOutput {
                holiday: Holiday {
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

        HolidayCommands::Edit {
            date,
            description,
            portion,
        } => {
            if !holidays.contains_key(date) {
                anyhow::bail!("No holiday exists on {}", date);
            }

            let entry = HolidayEntry {
                description: description.clone(),
                portion: portion.clone().unwrap_or(DayPortion::Full),
            };

            holidays.insert(*date, entry.clone());
            save_holidays(config_path, &holidays)?;

            let output = EditOutput {
                holiday: Holiday {
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

        HolidayCommands::List { year } => {
            let filtered: Vec<Holiday> = holidays
                .iter()
                .filter(|(date, _)| year.is_none() || date.year() == year.unwrap())
                .map(|(date, entry)| Holiday {
                    date: *date,
                    description: entry.description.clone(),
                    portion: entry.portion.clone(),
                })
                .collect();

            let output = ListOutput {
                holidays: filtered,
                filters: Filters { year: *year },
            };

            let output_string = match format {
                Format::Json => serde_json::to_string_pretty(&output)?,
                Format::Text => output.to_text(),
            };

            println!("{}", output_string);
        }

        HolidayCommands::Remove { date } => {
            let entry = match holidays.remove(date) {
                Some(entry) => entry,
                None => anyhow::bail!("No holiday found on {}.", date),
            };

            save_holidays(config_path, &holidays)?;

            let output = RemoveOutput {
                holiday: Holiday {
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
