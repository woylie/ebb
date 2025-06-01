// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::persistence::{load_sickdays, save_sickdays};
use crate::types::{DayPortion, Sickday, SickdayEntry};
use crate::{Format, SickdayArgs, SickdayCommands};
use chrono::Datelike;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct AddOutput {
    sickday: Sickday,
}

impl AddOutput {
    fn to_text(&self) -> String {
        format!(
            "Sick day '{}' added on {}.",
            self.sickday.description,
            self.sickday.date.format("%Y-%m-%d"),
        )
    }
}

#[derive(Serialize)]
struct EditOutput {
    sickday: Sickday,
}

impl EditOutput {
    fn to_text(&self) -> String {
        format!(
            "Updated sick day '{}' on {}.",
            self.sickday.description,
            self.sickday.date.format("%Y-%m-%d"),
        )
    }
}

#[derive(Serialize)]
struct ListOutput {
    sickdays: Vec<Sickday>,
    filters: Filters,
}

#[derive(Serialize)]
struct Filters {
    year: Option<i32>,
}

impl ListOutput {
    fn to_text(&self) -> String {
        if self.sickdays.is_empty() {
            match self.filters.year {
                Some(y) => format!("No sick days found for {}.", y),
                None => "No sick days found.".to_string(),
            }
        } else {
            let lines: Vec<String> = self
                .sickdays
                .iter()
                .map(|sickday| {
                    if sickday.portion == DayPortion::Full {
                        format!(
                            "{} — {}",
                            sickday.date.format("%Y-%m-%d"),
                            sickday.description
                        )
                    } else {
                        format!(
                            "{} — {} ({})",
                            sickday.date.format("%Y-%m-%d"),
                            sickday.description,
                            sickday.portion
                        )
                    }
                })
                .collect();

            lines.join("\n")
        }
    }
}

#[derive(Serialize)]
struct RemoveOutput {
    sickday: Sickday,
}

impl RemoveOutput {
    fn to_text(&self) -> String {
        format!(
            "Removed sick day '{}' on {}.",
            self.sickday.description,
            self.sickday.date.format("%Y-%m-%d"),
        )
    }
}

pub fn run_sickday(args: &SickdayArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    let mut sickdays = load_sickdays(config_path)?;

    match &args.command {
        SickdayCommands::Add {
            date,
            description,
            portion,
        } => {
            if sickdays.contains_key(date) {
                anyhow::bail!("A sick day already exists on {}", date);
            }

            let entry = SickdayEntry {
                description: description.clone(),
                portion: portion.clone().unwrap_or(DayPortion::Full),
            };

            sickdays.insert(*date, entry.clone());
            save_sickdays(config_path, &sickdays)?;

            let output = AddOutput {
                sickday: Sickday {
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

        SickdayCommands::Edit {
            date,
            description,
            portion,
        } => {
            if !sickdays.contains_key(date) {
                anyhow::bail!("No sick day found on {}", date);
            }

            let entry = SickdayEntry {
                description: description.clone(),
                portion: portion.clone().unwrap_or(DayPortion::Full),
            };

            sickdays.insert(*date, entry.clone());
            save_sickdays(config_path, &sickdays)?;

            let output = EditOutput {
                sickday: Sickday {
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

        SickdayCommands::List { year } => {
            let filtered: Vec<Sickday> = sickdays
                .iter()
                .filter(|(date, _)| year.is_none() || date.year() == year.unwrap())
                .map(|(date, entry)| Sickday {
                    date: *date,
                    description: entry.description.clone(),
                    portion: entry.portion.clone(),
                })
                .collect();

            let output = ListOutput {
                sickdays: filtered,
                filters: Filters { year: *year },
            };

            let output_string = match format {
                Format::Json => serde_json::to_string_pretty(&output)?,
                Format::Text => output.to_text(),
            };

            println!("{}", output_string);
        }

        SickdayCommands::Remove { date } => {
            let entry = match sickdays.remove(date) {
                Some(entry) => entry,
                None => anyhow::bail!("No sick day found on {}.", date),
            };

            save_sickdays(config_path, &sickdays)?;

            let output = RemoveOutput {
                sickday: Sickday {
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
