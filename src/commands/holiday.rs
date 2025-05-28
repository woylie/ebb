use crate::types::{DayPortion, HolidayEntry, Holidays};
use crate::HolidayArgs;
use crate::HolidayCommands;
use anyhow::Result;
use chrono::Datelike;
use chrono::NaiveDate;
use std::path::Path;
use std::{collections::BTreeMap, fs};

pub fn run_holiday(args: &HolidayArgs, config_path: &Path) -> anyhow::Result<()> {
    let holidays_file = config_path.join("holidays.toml");
    let mut holidays = load_holidays(&holidays_file)?;

    match &args.command {
        HolidayCommands::Add {
            date,
            description,
            portion,
        } => {
            if holidays.contains_key(date) {
                anyhow::bail!("A holiday already exists for {}", date);
            }

            let entry = HolidayEntry {
                description: description.clone(),
                portion: portion.clone().unwrap_or(DayPortion::Full),
            };

            holidays.insert(*date, entry.clone());

            save_holidays(&holidays_file, &holidays)?;
            println!("Added holiday: {}", fmt_entry(date, &entry));
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

            save_holidays(&holidays_file, &holidays)?;
            println!("Edited holiday: {}", fmt_entry(date, &entry));
        }

        HolidayCommands::List { year } => {
            let filtered: Vec<_> = holidays
                .iter()
                .filter(|(date, _)| year.is_none_or(|y| date.year() == y))
                .collect();

            if filtered.is_empty() {
                match year {
                    Some(y) => println!("No holidays found for {}.", y),
                    None => println!("No holidays recorded."),
                }
            } else {
                println!(
                    "Holidays{}:",
                    year.map_or(String::new(), |y| format!(" in {}", y))
                );
                for (date, entry) in filtered {
                    println!("{}", fmt_entry(date, entry));
                }
            }
        }

        HolidayCommands::Remove { date } => {
            if !holidays.contains_key(date) {
                anyhow::bail!("No holiday exists on {}", date);
            }

            holidays.remove(date);
            save_holidays(&holidays_file, &holidays)?;
            println!("Removed holiday: {}", date);
        }
    };

    Ok(())
}

fn load_holidays(path: &Path) -> Result<Holidays> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let contents = fs::read_to_string(path)?;
    Ok(toml::from_str(&contents)?)
}

fn save_holidays(path: &Path, holidays: &Holidays) -> Result<()> {
    let toml = toml::to_string(&holidays)?;
    fs::write(path, toml)?;
    Ok(())
}

fn fmt_date(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn fmt_entry(date: &NaiveDate, entry: &HolidayEntry) -> String {
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
