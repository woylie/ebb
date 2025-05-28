use crate::types::Sickdays;
use crate::SickdayArgs;
use crate::SickdayCommands;
use anyhow::Result;
use chrono::Datelike;
use std::path::Path;
use std::{collections::BTreeMap, fs, path::PathBuf};

pub fn run_sickday(args: &SickdayArgs, config_path: &Path) -> anyhow::Result<()> {
    let sickdays_file = config_path.join("sickdays.toml");
    let mut sickdays = load_sickdays(&sickdays_file)?;

    match &args.command {
        SickdayCommands::Add { date, description } => {
            if sickdays.contains_key(date) {
                anyhow::bail!("A sick day already exists for {}", date);
            }

            sickdays.insert(*date, description.clone());
            save_sickdays(&sickdays_file, &sickdays)?;
            println!("Added sick day: {} - {}", date, description);
        }

        SickdayCommands::Edit { date, description } => {
            if !sickdays.contains_key(date) {
                anyhow::bail!("No sick day exists on {}", date);
            }

            sickdays.insert(*date, description.clone());
            save_sickdays(&sickdays_file, &sickdays)?;
            println!("Edited sick day: {} - {}", date, description);
        }

        SickdayCommands::List { year } => {
            let mut filtered: Vec<_> = sickdays
                .iter()
                .filter(|(date, _)| year.map_or(true, |y| date.year() == y))
                .collect();

            filtered.sort_by_key(|(date, _)| *date);

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
                for (date, description) in filtered {
                    println!("• {} — {}", date.format("%Y-%m-%d"), description);
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

fn load_sickdays(path: &PathBuf) -> Result<Sickdays> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let contents = fs::read_to_string(path)?;
    Ok(toml::from_str(&contents)?)
}

fn save_sickdays(path: &PathBuf, sickdays: &Sickdays) -> Result<()> {
    let toml = toml::to_string(&sickdays)?;
    fs::write(path, toml)?;
    Ok(())
}
