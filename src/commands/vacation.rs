use crate::types::{DayPortion, VacationEntry, Vacations};
use crate::VacationArgs;
use crate::VacationCommands;
use anyhow::Result;
use chrono::Datelike;
use chrono::NaiveDate;
use std::path::Path;
use std::{collections::BTreeMap, fs};

pub fn run_vacation(args: &VacationArgs, config_path: &Path) -> anyhow::Result<()> {
    let vacations_file = config_path.join("vacations.toml");
    let mut vacations = load_vacations(&vacations_file)?;

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

            save_vacations(&vacations_file, &vacations)?;
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

            save_vacations(&vacations_file, &vacations)?;
            println!("Edited vacation: {}", fmt_entry(date, &entry));
        }

        VacationCommands::List { year } => {
            let filtered: Vec<_> = vacations
                .iter()
                .filter(|(date, _)| year.map_or(true, |y| date.year() == y))
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
            save_vacations(&vacations_file, &vacations)?;
            println!("Removed vacation: {}", date);
        }
    };

    Ok(())
}

fn load_vacations(path: &Path) -> Result<Vacations> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let contents = fs::read_to_string(path)?;
    Ok(toml::from_str(&contents)?)
}

fn save_vacations(path: &Path, vacations: &Vacations) -> Result<()> {
    let toml = toml::to_string(&vacations)?;
    fs::write(path, toml)?;
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
