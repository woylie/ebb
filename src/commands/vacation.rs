// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::persistence::{load_vacations, save_vacations};
use crate::types::{DayPortion, VacationEntry};
use crate::VacationArgs;
use crate::VacationCommands;
use chrono::Datelike;
use chrono::NaiveDate;
use std::path::Path;

pub fn run_vacation(args: &VacationArgs, config_path: &Path) -> anyhow::Result<()> {
    let mut vacations = load_vacations(config_path)?;

    match &args.command {
        VacationCommands::Add {
            date,
            description,
            portion,
        } => {
            if vacations.contains_key(date) {
                anyhow::bail!("A vacation already exists for {}", date);
            }

            let entry = VacationEntry {
                description: description.clone(),
                portion: portion.clone().unwrap_or(DayPortion::Full),
            };

            vacations.insert(*date, entry.clone());

            save_vacations(config_path, &vacations)?;
            println!("Added vacation: {}", fmt_entry(date, &entry));
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
            println!("Edited vacation: {}", fmt_entry(date, &entry));
        }

        VacationCommands::List { year } => {
            let filtered: Vec<_> = vacations
                .iter()
                .filter(|(date, _)| year.is_none_or(|y| date.year() == y))
                .collect();

            if filtered.is_empty() {
                match year {
                    Some(y) => println!("No vacations found for {}.", y),
                    None => println!("No vacations recorded."),
                }
            } else {
                println!(
                    "Vacations{}:",
                    year.map_or(String::new(), |y| format!(" in {}", y))
                );
                for (date, entry) in filtered {
                    println!("{}", fmt_entry(date, entry));
                }
            }
        }

        VacationCommands::Remove { date } => {
            if !vacations.contains_key(date) {
                anyhow::bail!("No vacation exists on {}", date);
            }

            vacations.remove(date);
            save_vacations(config_path, &vacations)?;
            println!("Removed vacation: {}", date);
        }
    };

    Ok(())
}

fn fmt_date(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn fmt_entry(date: &NaiveDate, entry: &VacationEntry) -> String {
    if entry.portion == DayPortion::Full {
        format!("{} — {}", fmt_date(date), entry.description)
    } else {
        format!(
            "{} — {} ({})",
            fmt_date(date),
            entry.description,
            entry.portion
        )
    }
}
