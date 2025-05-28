use crate::Commands::Sickday;
use anyhow::Result;
use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand};
use std::{fs, path::PathBuf};

pub mod types;

mod commands;

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
pub enum Commands {
    Sickday(SickdayArgs),
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct SickdayArgs {
    #[command(subcommand)]
    command: SickdayCommands,
}

#[derive(Debug, Subcommand)]
pub enum SickdayCommands {
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
    List {
        /// Filter by year
        #[arg(short, long)]
        year: Option<i32>,
    },
    /// Remove a sick day
    Remove {
        #[arg(required = true)]
        date: NaiveDate,
    },
}

pub fn run(cli: &Cli) -> Result<()> {
    let config_dir = shellexpand::tilde(&cli.config_dir.to_string_lossy()).to_string();
    let config_path = PathBuf::from(config_dir);
    fs::create_dir_all(&config_path)?;

    match &cli.command {
        Sickday(args) => commands::sickday::run_sickday(args, &config_path),
    }
}
