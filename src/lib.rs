use crate::Commands::Sickday;
use anyhow::Result;
use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand};

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

pub fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Sickday(SickdayArgs { command }) => {
            println!("Subcommand: {:?}", command)
        }
    }
    Ok(())
}
