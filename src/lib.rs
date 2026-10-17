// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::Commands::{
    Balance, Cancel, Config, DaysOff, GenerateCompletions, GenerateDocs, Holiday, Project, Report,
    Restart, SickDay, Start, Status, Stop, Tag, Vacation,
};
use crate::types::DayPortion;
use anyhow::{Context, Result, anyhow};
use chrono::{
    DateTime, Datelike, Days, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone,
};
use clap::CommandFactory;
use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};
use clap_complete::aot;
use etcetera::base_strategy::{BaseStrategy, choose_base_strategy};
use std::env;
use std::io;
use std::path::{Path, PathBuf};

pub mod cli;
pub mod formatting;
pub mod output;
pub mod persistence;
pub mod serde_utils;
pub mod types;

#[derive(Debug, Parser)]
#[command(name = "ebb")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Set the directory holding the configuration and the data; defaults to
    /// $XDG_DATA_HOME/ebb, or ~/.local/share/ebb when that is not set
    #[arg(
        short = 'c',
        long = "data-dir",
        alias = "config-dir",
        env = "EBB_DATA_DIR",
        global = true
    )]
    data_dir: Option<PathBuf>,
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
    /// Show working hour balance
    Balance(BalanceArgs),
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
    /// Generate shell completions
    #[command(hide = true)]
    GenerateCompletions {
        #[arg(long)]
        shell: clap_complete::Shell,
    },
}

#[derive(Debug, Default, Args)]
pub struct BalanceArgs {
    /// First date (yyyy-mm-dd); for the current day, week, month or year, use --day, --week, --month or --year
    #[arg(long, value_parser=parse_date, conflicts_with = "period")]
    from: Option<NaiveDate>,
    /// Last date, included (yyyy-mm-dd); for the current day, week, month or year, use --day, --week, --month or --year
    #[arg(long, value_parser=parse_date, conflicts_with = "period")]
    to: Option<NaiveDate>,
    #[command(flatten)]
    period: Period,
    /// End at the start of the current day, counting only finished days
    #[arg(long, conflicts_with_all = ["to", "day"])]
    exclude_today: bool,
}

#[derive(Debug, Default, Args)]
#[group(id = "period", multiple = false)]
pub struct Period {
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
}

impl Period {
    pub fn start(&self, today: NaiveDate) -> Option<NaiveDate> {
        let days_back = if self.day {
            0
        } else if self.week {
            today.weekday().num_days_from_monday()
        } else if self.month {
            today.day0()
        } else if self.year {
            today.ordinal0()
        } else {
            return None;
        };

        Some(today - Days::new(days_back.into()))
    }
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommands,
}

#[derive(Debug, Args)]
pub struct DaysOffArgs {
    /// Year; defaults to the current year
    #[arg(short, long)]
    year: Option<i32>,
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
pub struct ReportArgs {
    /// Start date or time (yyyy-mm-dd, hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601); for the current day, week, month or year, use --day, --week, --month or --year
    #[arg(long, value_parser=parse_range_start, conflicts_with = "period")]
    from: Option<DateTime<Local>>,
    /// End date or time (yyyy-mm-dd, hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601); a date includes that day; for the current day, week, month or year, use --day, --week, --month or --year
    #[arg(long, value_parser=parse_range_end, conflicts_with = "period")]
    to: Option<DateTime<Local>>,
    #[command(flatten)]
    period: Period,
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
        .args(&["at", "ago", "no_gap"])
        .required(false)
        .multiple(false),
))]
pub struct RestartArgs {
    /// Time at which the project is restarted (hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601); if omitted, the current time is used
    #[arg(long, value_parser=parse_flexible_datetime)]
    at: Option<DateTime<Local>>,
    /// How long ago the project was restarted, e.g. 30m or "1h 30m"; resolves to the time --at would name
    #[arg(long, value_parser=parse_ago)]
    ago: Option<DateTime<Local>>,
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
        .args(&["at", "ago", "no_gap"])
        .required(false)
        .multiple(false),
))]
pub struct StartArgs {
    /// Name of the project
    project: String,
    /// Any number of additional tags; quote a tag that contains spaces
    #[arg(num_args = 0.., value_parser=parse_tag)]
    tags: Vec<String>,
    /// Time at which the project is started and a running frame is stopped (hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601); if omitted, the current time is used
    #[arg(long, value_parser=parse_flexible_datetime)]
    at: Option<DateTime<Local>>,
    /// How long ago the project was started, e.g. 30m or "1h 30m"; resolves to the time --at would name
    #[arg(long, value_parser=parse_ago)]
    ago: Option<DateTime<Local>>,
    /// Set the start time to the end time of the last saved frame
    #[arg(short = 'G', long)]
    no_gap: bool,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("stop")
        .args(&["at", "ago"])
        .required(false)
        .multiple(false),
))]
pub struct StopArgs {
    /// Time at which the project is stopped (hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601); if omitted, the current time is used
    #[arg(long, value_parser=parse_flexible_datetime)]
    at: Option<DateTime<Local>>,
    /// How long ago the project was stopped, e.g. 30m or "1h 30m"; resolves to the time --at would name
    #[arg(long, value_parser=parse_ago)]
    ago: Option<DateTime<Local>>,
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

    // Resolved per command, so that the two commands that only write to stdout keep
    // working without a home directory and without creating anything.
    let config_path = || -> Result<PathBuf> {
        let path = resolve_data_dir(cli.data_dir.as_deref())?;
        persistence::create_data_dir(&path)?;
        Ok(path)
    };

    match &cli.command {
        Balance(args) => cli::balance::run_balance(args, &config_path()?, format),
        Cancel => cli::tracking::run_cancel(&config_path()?, format),
        Config(args) => cli::config::run_config(args, &config_path()?, format),
        DaysOff(args) => cli::days_off::run_daysoff(args, &config_path()?, format),
        Holiday(args) => cli::calendar::run(
            &cli::calendar::HOLIDAY,
            (&args.command).into(),
            &config_path()?,
            format,
        ),
        Project(args) => cli::project::run_project(args, &config_path()?, format),
        Report(args) => cli::report::run_report(args, &config_path()?, format),
        Restart(args) => cli::tracking::run_restart(args, &config_path()?, format),
        SickDay(args) => cli::calendar::run(
            &cli::calendar::SICK_DAY,
            (&args.command).into(),
            &config_path()?,
            format,
        ),
        Start(args) => cli::tracking::run_start(args, &config_path()?, format),
        Status => cli::tracking::run_status(&config_path()?, format),
        Stop(args) => cli::tracking::run_stop(args, &config_path()?, format),
        Tag(args) => cli::tag::run_tag(args, &config_path()?, format),
        Vacation(args) => cli::calendar::run(
            &cli::calendar::VACATION,
            (&args.command).into(),
            &config_path()?,
            format,
        ),
        GenerateDocs => {
            clap_markdown::print_help_markdown::<Cli>();
            Ok(())
        }
        GenerateCompletions { shell } => {
            let mut cmd = Cli::command();
            aot::generate(*shell, &mut cmd, "ebb", &mut io::stdout());
            Ok(())
        }
    }
}

/// A single directory holds every file. Five of the six are data rather than
/// configuration, so it sits under the XDG Base Directory Specification's data
/// location: `~/.local/share` unless `$XDG_DATA_HOME` names an absolute path.
fn resolve_data_dir(configured: Option<&Path>) -> Result<PathBuf> {
    if let Some(dir) = configured {
        return Ok(expand_tilde(dir));
    }

    // The directory was named after the configuration until the files moved to the
    // data directory. Honouring the old variable keeps an existing shell profile
    // working instead of silently reading an empty directory.
    if let Some(dir) = env::var_os("EBB_CONFIG_DIR") {
        eprintln!("EBB_CONFIG_DIR is deprecated. Rename it to EBB_DATA_DIR.");
        return Ok(expand_tilde(Path::new(&dir)));
    }

    let strategy = choose_base_strategy()
        .context("Cannot determine the home directory. Set HOME, or pass --data-dir.")?;

    Ok(strategy.data_dir().join("ebb"))
}

fn expand_tilde(path: &Path) -> PathBuf {
    PathBuf::from(shellexpand::tilde(&path.to_string_lossy()).to_string())
}

fn check_name(name: &str, argument: &str) -> Result<()> {
    if name.chars().any(char::is_control) {
        return Err(anyhow!("{argument} contains a control character."));
    }

    Ok(())
}

fn parse_tag(input: &str) -> Result<String> {
    if let Some(tag) = input.strip_prefix('+') {
        return Err(anyhow!("the '+' prefix is not used; write '{tag}'"));
    }

    if input.trim().is_empty() {
        return Err(anyhow!("a tag cannot be empty"));
    }

    Ok(input.to_string())
}

fn parse_ago(input: &str) -> Result<DateTime<Local>> {
    let duration = humantime::parse_duration(input)
        .map_err(|_| anyhow!("'{input}' is not a duration. Write it as 30m, 2h or \"1h 30m\"."))?;

    TimeDelta::from_std(duration)
        .ok()
        .and_then(|delta| Local::now().checked_sub_signed(delta))
        .filter(|time| time.timestamp() >= 0)
        .ok_or_else(|| anyhow!("'{input}' reaches back before 1970."))
}

fn parse_flexible_datetime(input: &str) -> Result<DateTime<Local>> {
    parse_local_datetime(input).unwrap_or_else(|| {
        Err(anyhow!(
            "'{input}' is not a time. Write it as hh:mm, hh:mm:ss, \
             yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601."
        ))
    })
}

fn parse_range_start(input: &str) -> Result<DateTime<Local>> {
    parse_range_bound(input, "start", Some)
}

fn parse_range_end(input: &str) -> Result<DateTime<Local>> {
    parse_range_bound(input, "end", |date| date.succ_opt())
}

fn parse_date(input: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(input, "%Y-%m-%d")
        .map_err(|_| anyhow!("'{input}' is not a date. Write it as yyyy-mm-dd."))
}

fn parse_range_bound(
    input: &str,
    edge: &str,
    midnight_of: impl Fn(NaiveDate) -> Option<NaiveDate>,
) -> Result<DateTime<Local>> {
    if let Ok(date) = parse_date(input) {
        return midnight_of(date)
            .and_then(|day| {
                day.and_time(NaiveTime::MIN)
                    .and_local_timezone(Local)
                    .single()
            })
            .ok_or_else(|| {
                anyhow!(
                    "Midnight at the {edge} of {input} does not exist or occurs twice \
                     in the local time zone. Give a date and time instead."
                )
            });
    }

    parse_local_datetime(input).unwrap_or_else(|| {
        Err(anyhow!(
            "'{input}' is not a date or time. Write it as yyyy-mm-dd, hh:mm, hh:mm:ss, \
             yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601."
        ))
    })
}

fn parse_local_datetime(input: &str) -> Option<Result<DateTime<Local>>> {
    if let Ok(dt_fixed) = DateTime::parse_from_rfc3339(input) {
        return Some(Ok(dt_fixed.with_timezone(&Local)));
    }

    let dt_formats = ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M"];

    for fmt in &dt_formats {
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(input, fmt) {
            return Some(
                Local
                    .from_local_datetime(&naive_dt)
                    .single()
                    .ok_or_else(|| anyhow!("Ambiguous local datetime")),
            );
        }
    }

    let time_formats = ["%H:%M:%S", "%H:%M"];

    for fmt in &time_formats {
        if let Ok(naive_time) = NaiveTime::parse_from_str(input, fmt) {
            let today = Local::now().date_naive();
            let naive_dt = NaiveDateTime::new(today, naive_time);
            return Some(
                Local
                    .from_local_datetime(&naive_dt)
                    .single()
                    .ok_or_else(|| anyhow!("Ambiguous local datetime")),
            );
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starts_each_period_on_its_first_date() {
        // A Wednesday.
        let today = NaiveDate::from_ymd_opt(2026, 10, 7).unwrap();
        let cases = [
            (Period::default(), None),
            (
                Period {
                    day: true,
                    ..Default::default()
                },
                NaiveDate::from_ymd_opt(2026, 10, 7),
            ),
            (
                Period {
                    week: true,
                    ..Default::default()
                },
                NaiveDate::from_ymd_opt(2026, 10, 5),
            ),
            (
                Period {
                    month: true,
                    ..Default::default()
                },
                NaiveDate::from_ymd_opt(2026, 10, 1),
            ),
            (
                Period {
                    year: true,
                    ..Default::default()
                },
                NaiveDate::from_ymd_opt(2026, 1, 1),
            ),
        ];

        for (period, expected) in cases {
            assert_eq!(period.start(today), expected, "for {period:?}");
        }
    }
}
