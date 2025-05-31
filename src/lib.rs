// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::types::DayPortion;
use crate::Commands::{Holiday, Sickday, Start, Vacation};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
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
    /// Start time tracking
    Start(StartArgs),
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
pub struct StartArgs {
    project: String,
    #[arg(long, value_parser=parse_flexible_datetime)]
    at: Option<DateTime<Local>>,
    #[arg(short = 'G', long)]
    no_gap: bool,
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
        Start(args) => commands::tracking::run_start(args, &config_path),
        Vacation(args) => commands::vacation::run_vacation(args, &config_path),
    }
}

fn parse_flexible_datetime(input: &str) -> Result<DateTime<Local>> {
    if let Ok(dt_fixed) = DateTime::parse_from_rfc3339(input) {
        return Ok(dt_fixed.with_timezone(&Local));
    }

    let dt_formats = ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M"];

    for fmt in &dt_formats {
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(input, fmt) {
            return Local
                .from_local_datetime(&naive_dt)
                .single()
                .ok_or_else(|| anyhow!("Ambiguous local datetime"));
        }
    }

    let time_formats = ["%H:%M:%S", "%H:%M"];

    for fmt in &time_formats {
        if let Ok(naive_time) = NaiveTime::parse_from_str(input, fmt) {
            let today = Local::now().date_naive();
            let naive_dt = NaiveDateTime::new(today, naive_time);
            return Local
                .from_local_datetime(&naive_dt)
                .single()
                .ok_or_else(|| anyhow!("Ambiguous local datetime"));
        }
    }

    if let Ok(secs) = input.parse::<i64>() {
        let naive_dt = Utc
            .timestamp_opt(secs, 0)
            .single()
            .ok_or_else(|| anyhow!("Invalid timestamp"))?;
        return Ok(naive_dt.with_timezone(&Local));
    }

    Err(anyhow!("Could not parse datetime from input: {}", input))
}
