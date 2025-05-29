// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::types::DayPortion;
use crate::Commands::{Holiday, Sickday, Vacation};
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
    /// Manage holidays
    Holiday(HolidayArgs),
    /// Manage sick days
    Sickday(SickdayArgs),
    /// Manage vacation days
    Vacation(VacationArgs),
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct HolidayArgs {
    #[command(subcommand)]
    command: HolidayCommands,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct SickdayArgs {
    #[command(subcommand)]
    command: SickdayCommands,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct VacationArgs {
    #[command(subcommand)]
    command: VacationCommands,
}

#[derive(Debug, Subcommand)]
pub enum HolidayCommands {
    /// Add a new holiday
    Add {
        date: NaiveDate,
        #[arg(default_value = "Holiday")]
        description: String,
        #[arg(short, long, default_value = "full")]
        portion: Option<DayPortion>,
    },
    /// Edit the description of an existing holiday
    Edit {
        date: NaiveDate,
        description: String,
        #[arg(short, long)]
        portion: Option<DayPortion>,
    },
    /// List all holidays
    List {
        /// Filter by year
        #[arg(short, long)]
        year: Option<i32>,
    },
    /// Remove a holiday
    Remove {
        #[arg(required = true)]
        date: NaiveDate,
    },
}

#[derive(Debug, Subcommand)]
pub enum SickdayCommands {
    /// Add a new sick day
    Add {
        date: NaiveDate,
        #[arg(default_value = "Sick")]
        description: String,
        #[arg(short, long, default_value = "full")]
        portion: Option<DayPortion>,
    },
    /// Edit the description of an existing sick day
    Edit {
        date: NaiveDate,
        description: String,
        #[arg(short, long)]
        portion: Option<DayPortion>,
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

#[derive(Debug, Subcommand)]
pub enum VacationCommands {
    /// Add a new vacation day
    Add {
        date: NaiveDate,
        #[arg(default_value = "Vacation")]
        description: String,
        #[arg(short, long, default_value = "full")]
        portion: Option<DayPortion>,
    },
    /// Edit the description of an existing vacation day
    Edit {
        date: NaiveDate,
        description: String,
        #[arg(short, long)]
        portion: Option<DayPortion>,
    },
    /// List all vacation days
    List {
        /// Filter by year
        #[arg(short, long)]
        year: Option<i32>,
    },
    /// Remove a vacation day
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
        Holiday(args) => commands::holiday::run_holiday(args, &config_path),
        Sickday(args) => commands::sickday::run_sickday(args, &config_path),
        Vacation(args) => commands::vacation::run_vacation(args, &config_path),
    }
}
