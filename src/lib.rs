// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::types::DayPortion;
use crate::Commands::{
    Cancel, Config, DaysOff, GenerateDocs, Holiday, Project, Report, Restart, SickDay, Start,
    Status, Stop, Tag, Vacation,
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};
use std::{fs, path::PathBuf};

pub mod cli;
pub mod persistence;
pub mod types;

#[derive(Debug, Parser)]
#[command(name = "ebb")]
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
    /// Set the output format
    #[arg(
        short = 'f',
        long = "format",
        global = true,
        default_value = "text",
        value_enum
    )]
    format: Format,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Format {
    Text,
    Json,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Cancel the current time tracking frame
    Cancel,
    /// Manage the configuration
    Config(ConfigArgs),
    /// Print overview of remaining vacation and sick days
    #[command(name = "daysoff")]
    DaysOff(DaysOffArgs),
    /// Manage holidays
    Holiday(HolidayArgs),
    /// Manage projects
    Project(ProjectArgs),
    /// Return the total time and time spent per project
    Report(ReportArgs),
    /// Restart the last project
    Restart(RestartArgs),
    /// Manage sick days
    #[command(name = "sickday")]
    SickDay(SickDayArgs),
    /// Start time tracking
    Start(StartArgs),
    /// Show current time tracking status
    Status,
    /// Stop time tracking
    Stop(StopArgs),
    /// Manage tags
    Tag(TagArgs),
    /// Manage vacation days
    Vacation(VacationArgs),
    /// Generate the Markdown documentation
    #[command(hide = true)]
    GenerateDocs,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommands,
}

#[derive(Debug, Args)]
pub struct DaysOffArgs {
    /// Year
    #[arg(short, long, default_value_t = default_year())]
    year: i32,
}

fn default_year() -> i32 {
    Local::now().year()
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct HolidayArgs {
    #[command(subcommand)]
    command: HolidayCommands,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct ProjectArgs {
    #[command(subcommand)]
    command: ProjectCommands,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("time_filter_from")
        .args(&["from", "day", "week", "month", "year"])
        .required(false)
        .multiple(false),
))]
#[command(group(
    ArgGroup::new("time_filter_to")
        .args(&["to", "day", "week", "month", "year"])
        .required(false)
        .multiple(false),
))]
pub struct ReportArgs {
    /// Start time (hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601)
    #[arg(long, value_parser=parse_flexible_datetime)]
    from: Option<DateTime<Local>>,
    /// End time (hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601)
    #[arg(long, value_parser=parse_flexible_datetime)]
    to: Option<DateTime<Local>>,
    /// Report time spent in the current year
    #[arg(short, long)]
    year: bool,
    /// Report time spent in the current month
    #[arg(short, long)]
    month: bool,
    /// Report time spent in the current week
    #[arg(short, long)]
    week: bool,
    /// Report time spent on the current day
    #[arg(short, long)]
    day: bool,
    /// Filter by project
    #[arg(short, long)]
    project: Option<String>,
    /// Filter by tag
    #[arg(short, long)]
    tag: Option<String>,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("start")
        .args(&["at", "no_gap"])
        .required(false)
        .multiple(false),
))]
pub struct RestartArgs {
    /// Time at which the project is restarted (hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601); if omitted, the current time is used
    #[arg(long, value_parser=parse_flexible_datetime)]
    at: Option<DateTime<Local>>,
    /// Set the start time to the end time of the last saved frame
    #[arg(short = 'G', long)]
    no_gap: bool,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct SickDayArgs {
    #[command(subcommand)]
    command: SickDayCommands,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("start")
        .args(&["at", "no_gap"])
        .required(false)
        .multiple(false),
))]
pub struct StartArgs {
    /// Name of the project
    project: String,
    /// Any number of additional tags
    #[arg(num_args = 0.., trailing_var_arg = true)]
    tags: Vec<String>,
    /// Time at which the project is started (hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601); if omitted, the current time is used
    #[arg(long, value_parser=parse_flexible_datetime)]
    at: Option<DateTime<Local>>,
    /// Set the start time to the end time of the last saved frame
    #[arg(short = 'G', long)]
    no_gap: bool,
}

#[derive(Debug, Args)]
pub struct StopArgs {
    /// Time at which the project is stopped (hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601); if omitted, the current time is used
    #[arg(long, value_parser=parse_flexible_datetime)]
    at: Option<DateTime<Local>>,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct TagArgs {
    #[command(subcommand)]
    command: TagCommands,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct VacationArgs {
    #[command(subcommand)]
    command: VacationCommands,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Get a single configuration value
    Get { key: String },
    /// List all configuration values
    List,
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum HolidayCommands {
    /// Add a new holiday
    Add {
        /// Date of the holiday (yyyy-mm-dd, e.g. 2025-08-11)
        date: NaiveDate,
        /// Name of the holiday (e.g. Mountain Day)
        #[arg(default_value = "Holiday")]
        description: String,
        /// Switch between full-day and half-day holiday
        #[arg(short, long, default_value = "full")]
        portion: Option<DayPortion>,
    },
    /// Edit the description of an existing holiday
    Edit {
        /// Date of the holiday to edit
        date: NaiveDate,
        /// New name for the holiday
        description: String,
        /// Switch between full-day and half-day holiday
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
        /// Date of the holiday to remove
        #[arg(required = true)]
        date: NaiveDate,
    },
}

#[derive(Debug, Subcommand)]
pub enum ProjectCommands {
    /// List all projects
    List,
    /// Renames a project
    Rename {
        /// Current project name
        #[arg(required = true)]
        old_name: String,
        /// New project name
        #[arg(required = true)]
        new_name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum SickDayCommands {
    /// Add a new sick day
    Add {
        /// Day of the sick day
        date: NaiveDate,
        /// Description for the sick day
        #[arg(default_value = "Sick")]
        description: String,
        /// Switch between full-day and half-day holiday
        #[arg(short, long, default_value = "full")]
        portion: Option<DayPortion>,
    },
    /// Edit the description of an existing sick day
    Edit {
        /// Date of the sick day to edit
        date: NaiveDate,
        /// New description for the sick day
        description: String,
        /// Switch between full-day and half-day holiday
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
        /// Date of the sick day to remove
        #[arg(required = true)]
        date: NaiveDate,
    },
}

#[derive(Debug, Subcommand)]
pub enum TagCommands {
    /// List all tags
    List,
    /// Remove a tag
    Remove {
        /// Name of the tag
        #[arg(required = true)]
        tag: String,
    },
    /// Renames a tag
    Rename {
        /// Current tag name
        #[arg(required = true)]
        old_name: String,
        /// New tag name
        #[arg(required = true)]
        new_name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum VacationCommands {
    /// Add a new vacation day
    Add {
        /// Date of the vacation day
        date: NaiveDate,
        /// Name of the vacation day
        #[arg(default_value = "Vacation")]
        description: String,
        /// Switch between full-day and half-day holiday
        #[arg(short, long, default_value = "full")]
        portion: Option<DayPortion>,
    },
    /// Edit the description of an existing vacation day
    Edit {
        /// Date of the vacation day to edit
        date: NaiveDate,
        /// New name for the vacation day
        description: String,
        /// Switch between full-day and half-day holiday
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
        /// Date of the vacation day to remove
        #[arg(required = true)]
        date: NaiveDate,
    },
}

pub fn run(cli: &Cli) -> Result<()> {
    let format = &cli.format;
    let config_dir = shellexpand::tilde(&cli.config_dir.to_string_lossy()).to_string();
    let config_path = PathBuf::from(config_dir);
    fs::create_dir_all(&config_path)?;

    match &cli.command {
        Cancel => cli::tracking::run_cancel(&config_path, format),
        Config(args) => cli::config::run_config(args, &config_path, format),
        DaysOff(args) => cli::days_off::run_daysoff(args, &config_path, format),
        Holiday(args) => cli::holiday::run_holiday(args, &config_path, format),
        Project(args) => cli::project::run_project(args, &config_path, format),
        Report(args) => cli::report::run_report(args, &config_path, format),
        Restart(args) => cli::tracking::run_restart(args, &config_path, format),
        SickDay(args) => cli::sick_day::run_sick_day(args, &config_path, format),
        Start(args) => cli::tracking::run_start(args, &config_path, format),
        Status => cli::tracking::run_status(&config_path, format),
        Stop(args) => cli::tracking::run_stop(args, &config_path, format),
        Tag(args) => cli::tag::run_tag(args, &config_path, format),
        Vacation(args) => cli::vacation::run_vacation(args, &config_path, format),
        GenerateDocs => {
            clap_markdown::print_help_markdown::<Cli>();
            Ok(())
        }
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
