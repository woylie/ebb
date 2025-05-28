use crate::types::{DayPortion, SickdayEntry, Sickdays};
use crate::SickdayArgs;
use crate::SickdayCommands;
use anyhow::Result;
use chrono::Datelike;
use chrono::NaiveDate;
use std::path::Path;
use std::{collections::BTreeMap, fs};

pub fn run_sickday(args: &SickdayArgs, config_path: &Path) -> anyhow::Result<()> {
    let sickdays_file = config_path.join("sickdays.toml");
    let mut sickdays = load_sickdays(&sickdays_file)?;

    match &args.command {
        SickdayCommands::Add {
            date,
            description,
            portion,
        } => {
            if sickdays.contains_key(date) {
                anyhow::bail!("A sick day already exists for {}", date);
            }

            let entry = SickdayEntry {
                description: description.clone(),
                portion: portion.clone().unwrap_or(DayPortion::Full),
            };

            sickdays.insert(*date, entry.clone());

            save_sickdays(&sickdays_file, &sickdays)?;
            println!("Added sick day: {}", fmt_entry(date, &entry));
        }

        SickdayCommands::Edit {
            date,
            description,
            portion,
        } => {
            if !sickdays.contains_key(date) {
                anyhow::bail!("No sick day exists on {}", date);
            }

            let entry = SickdayEntry {
                description: description.clone(),
                portion: portion.clone().unwrap_or(DayPortion::Full),
            };

            sickdays.insert(*date, entry.clone());

            save_sickdays(&sickdays_file, &sickdays)?;
            println!("Edited sick day: {}", fmt_entry(date, &entry));
        }

        SickdayCommands::List { year } => {
            let filtered: Vec<_> = sickdays
                .iter()
                .filter(|(date, _)| year.map_or(true, |y| date.year() == y))
                .collect();

            if filtered.is_empty() {
                match year {
                    Some(y) => println!("No sick days found for {}.", y),
                    None => println!("No sick days recorded."),
                }
            } else {
                println!(
                    "Sick days{}:",
                    year.map_or(String::new(), |y| format!(" in {}", y))
                );
                for (date, entry) in filtered {
                    println!("{}", fmt_entry(date, entry));
                }
            }
        }

        SickdayCommands::Remove { date } => {
            if !sickdays.contains_key(date) {
                anyhow::bail!("No sick day exists on {}", date);
            }

            sickdays.remove(date);
            save_sickdays(&sickdays_file, &sickdays)?;
            println!("Removed sick day: {}", date);
        }
    };

    Ok(())
}

fn load_sickdays(path: &Path) -> Result<Sickdays> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let contents = fs::read_to_string(path)?;
    Ok(toml::from_str(&contents)?)
}

fn save_sickdays(path: &Path, sickdays: &Sickdays) -> Result<()> {
    let toml = toml::to_string(&sickdays)?;
    fs::write(path, toml)?;
    Ok(())
}

fn fmt_date(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn fmt_entry(date: &NaiveDate, entry: &SickdayEntry) -> String {
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
