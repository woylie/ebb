use crate::Commands::Sickday;
use anyhow::{Result};
use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand};
use std::{collections::BTreeMap, fs, path::PathBuf};

#[derive(Debug, Parser)]
#[command(name = "Ebb")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Set the configuration directory
    #[arg(
        short = 'c',
        long = "config-dir",
        env = "EBB_CONFIG_DIR",
        global = true,
        default_value = "~/.config/ebb"
    )]
    config_dir: std::path::PathBuf,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Sickday(SickdayArgs),
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
struct SickdayArgs {
    #[command(subcommand)]
    command: SickdayCommands,
}

#[derive(Debug, Subcommand)]
enum SickdayCommands {
    /// Add a new sick day
    Add {
        date: NaiveDate,
        #[arg(default_value = "sick")]
        description: String,
    },
    /// Edit the description of an existing sick day
    Edit {
        date: NaiveDate,
        description: String,
    },
    /// List all sick days
    List {},
    /// Remove a sick day
    Remove {
        #[arg(required = true)]
        date: NaiveDate,
    },
}

type Sickdays = BTreeMap<NaiveDate, String>;

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

pub fn run(cli: &Cli) -> Result<()> {
    let config_dir = shellexpand::tilde(&cli.config_dir.to_string_lossy()).to_string();
    let config_path = PathBuf::from(config_dir);
    let sickdays_file = config_path.join("sickdays.toml");

    match &cli.command {
        Sickday(SickdayArgs { command }) => match command {
            SickdayCommands::Add { date, description } => {
                fs::create_dir_all(&config_path)?;
                let mut sickdays = load_sickdays(&sickdays_file)?;

                if sickdays.contains_key(date) {
                    anyhow::bail!("A sick day already exists for {}", date);
                }

                sickdays.insert(*date, description.clone());
                save_sickdays(&sickdays_file, &sickdays)?;
                println!("Added sick day: {} - {}", date, description);
            }

            _ =>
            println!("Subcommand: {:?}", command)
        }
    }
    Ok(())
}
