// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::persistence::{load_vacations, save_vacations};
use crate::types::{DayPortion, Vacation, VacationEntry};
use crate::{Format, VacationArgs, VacationCommands};
use chrono::Datelike;
use serde::Serialize;
use std::path::Path;
use tabled::{Table, settings::Style};

#[derive(Serialize)]
struct AddOutput {
    vacation: Vacation,
}

impl AddOutput {
    fn to_text(&self) -> String {
        format!(
            "Vacation '{}' added on {}.",
            self.vacation.description,
            self.vacation.date.format("%Y-%m-%d"),
        )
    }
}

#[derive(Serialize)]
struct EditOutput {
    vacation: Vacation,
}

impl EditOutput {
    fn to_text(&self) -> String {
        format!(
            "Updated vacation '{}' on {}.",
            self.vacation.description,
            self.vacation.date.format("%Y-%m-%d"),
        )
    }
}

#[derive(Serialize)]
struct ListOutput {
    vacations: Vec<Vacation>,
    filters: Filters,
}

#[derive(Serialize)]
struct Filters {
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<i32>,
}

impl ListOutput {
    fn to_text(&self) -> String {
        if self.vacations.is_empty() {
            match self.filters.year {
                Some(y) => format!("No vacations found for {}.", y),
                None => "No vacations found.".to_string(),
            }
        } else {
            let mut table = Table::new(self.vacations.clone());
            table.with(Style::sharp()).to_string()
        }
    }
}

#[derive(Serialize)]
struct RemoveOutput {
    vacation: Vacation,
}

impl RemoveOutput {
    fn to_text(&self) -> String {
        format!(
            "Removed vacation '{}' on {}.",
            self.vacation.description,
            self.vacation.date.format("%Y-%m-%d"),
        )
    }
}

pub fn run_vacation(
    args: &VacationArgs,
    config_path: &Path,
    format: &Format,
) -> anyhow::Result<()> {
    let mut vacations = load_vacations(config_path)?;

    match &args.command {
        VacationCommands::Add {
            date,
            description,
            portion,
        } => {
            if vacations.contains_key(date) {
                anyhow::bail!("A vacation already exists on {}", date);
            }

            let entry = VacationEntry {
                description: description.clone(),
                portion: portion.clone().unwrap_or(DayPortion::Full),
            };

            vacations.insert(*date, entry.clone());
            save_vacations(config_path, &vacations)?;

            let output = AddOutput {
                vacation: Vacation {
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

        VacationCommands::Edit {
            date,
            description,
            portion,
        } => {
            if !vacations.contains_key(date) {
                anyhow::bail!("No vacation exists on {}", date);
            }

            let entry = VacationEntry {
                description: description.clone(),
                portion: portion.clone().unwrap_or(DayPortion::Full),
            };

            vacations.insert(*date, entry.clone());
            save_vacations(config_path, &vacations)?;

            let output = EditOutput {
                vacation: Vacation {
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

        VacationCommands::List { year } => {
            let filtered: Vec<Vacation> = vacations
                .iter()
                .filter(|(date, _)| year.is_none() || date.year() == year.unwrap())
                .map(|(date, entry)| Vacation {
                    date: *date,
                    description: entry.description.clone(),
                    portion: entry.portion.clone(),
                })
                .collect();

            let output = ListOutput {
                vacations: filtered,
                filters: Filters { year: *year },
            };

            let output_string = match format {
                Format::Json => serde_json::to_string_pretty(&output)?,
                Format::Text => output.to_text(),
            };

            println!("{}", output_string);
        }

        VacationCommands::Remove { date } => {
            let entry = match vacations.remove(date) {
                Some(entry) => entry,
                None => anyhow::bail!("No vacation found on {}.", date),
            };

            save_vacations(config_path, &vacations)?;

            let output = RemoveOutput {
                vacation: Vacation {
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
