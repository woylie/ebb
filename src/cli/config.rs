// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::persistence::{load_config, save_config};
use crate::types::Config;
use crate::{ConfigArgs, ConfigCommands, Format};
use anyhow::bail;
use humantime::{format_duration, parse_duration, FormattedDuration};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;
use tabled::Tabled;
use tabled::{settings::Style, Table};

#[derive(Serialize)]
struct ListOutput {
    config: Config,
}

#[derive(Tabled)]
struct ConfigRow<'a> {
    #[tabled(rename = "Config Key")]
    key: &'a str,
    #[tabled(rename = "Value")]
    value: FormattedDuration,
}

impl ListOutput {
    fn to_text(&self) -> String {
        let rows = vec![
            ConfigRow {
                key: "working_hours.monday",
                value: format_duration(self.config.working_hours.monday),
            },
            ConfigRow {
                key: "working_hours.tuesday",
                value: format_duration(self.config.working_hours.tuesday),
            },
            ConfigRow {
                key: "working_hours.wednesday",
                value: format_duration(self.config.working_hours.wednesday),
            },
            ConfigRow {
                key: "working_hours.thursday",
                value: format_duration(self.config.working_hours.thursday),
            },
            ConfigRow {
                key: "working_hours.friday",
                value: format_duration(self.config.working_hours.friday),
            },
            ConfigRow {
                key: "working_hours.saturday",
                value: format_duration(self.config.working_hours.saturday),
            },
            ConfigRow {
                key: "working_hours.sunday",
                value: format_duration(self.config.working_hours.sunday),
            },
        ];

        Table::new(rows).with(Style::sharp()).to_string()
    }
}

#[derive(Serialize)]
struct SetOutput<'a> {
    key: &'a String,
    old_value: Value,
    new_value: Value,
}

impl SetOutput<'_> {
    fn to_text(&self) -> String {
        format!(
            "Key: {}\nOld value: {}\nNew value: {}",
            self.key, self.old_value, self.new_value
        )
    }
}

pub fn run_config(args: &ConfigArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    let mut config = load_config(config_path)?;

    match &args.command {
        ConfigCommands::List => {
            let output = ListOutput { config };

            let output_string = match format {
                Format::Json => serde_json::to_string_pretty(&output)?,
                Format::Text => output.to_text(),
            };

            println!("{}", output_string);
        }

        ConfigCommands::Set { key, value } => {
            let (old_value, new_value) = set_config_value(&mut config, key, value)?;
            save_config(config_path, &config)?;

            let output = SetOutput {
                key,
                old_value,
                new_value,
            };

            let output_string = match format {
                Format::Json => serde_json::to_string_pretty(&output)?,
                Format::Text => output.to_text(),
            };

            println!("{}", output_string);
        }
    };

    Ok(())
}

fn set_config_value(
    config: &mut Config,
    key: &str,
    value_str: &str,
) -> anyhow::Result<(Value, Value)> {
    match key {
        "working_hours.monday" => {
            let old_value = Value::String(format_duration(config.working_hours.monday).to_string());
            let new_duration = parse_duration(value_str)
                .map_err(|e| anyhow::anyhow!("Invalid duration for {}: {}", key, e))?;
            config.working_hours.monday = new_duration;
            let new_value = Value::String(format_duration(new_duration).to_string());
            Ok((old_value, new_value))
        }
        "working_hours.tuesday" => {
            let old_value =
                Value::String(format_duration(config.working_hours.tuesday).to_string());
            let new_duration = parse_duration(value_str)
                .map_err(|e| anyhow::anyhow!("Invalid duration for {}: {}", key, e))?;
            config.working_hours.tuesday = new_duration;
            let new_value = Value::String(format_duration(new_duration).to_string());
            Ok((old_value, new_value))
        }
        "working_hours.wednesday" => {
            let old_value =
                Value::String(format_duration(config.working_hours.wednesday).to_string());
            let new_duration = parse_duration(value_str)
                .map_err(|e| anyhow::anyhow!("Invalid duration for {}: {}", key, e))?;
            config.working_hours.wednesday = new_duration;
            let new_value = Value::String(format_duration(new_duration).to_string());
            Ok((old_value, new_value))
        }
        "working_hours.thursday" => {
            let old_value =
                Value::String(format_duration(config.working_hours.thursday).to_string());
            let new_duration = parse_duration(value_str)
                .map_err(|e| anyhow::anyhow!("Invalid duration for {}: {}", key, e))?;
            config.working_hours.thursday = new_duration;
            let new_value = Value::String(format_duration(new_duration).to_string());
            Ok((old_value, new_value))
        }
        "working_hours.friday" => {
            let old_value = Value::String(format_duration(config.working_hours.friday).to_string());
            let new_duration = parse_duration(value_str)
                .map_err(|e| anyhow::anyhow!("Invalid duration for {}: {}", key, e))?;
            config.working_hours.friday = new_duration;
            let new_value = Value::String(format_duration(new_duration).to_string());
            Ok((old_value, new_value))
        }
        "working_hours.saturday" => {
            let old_value =
                Value::String(format_duration(config.working_hours.saturday).to_string());
            let new_duration = parse_duration(value_str)
                .map_err(|e| anyhow::anyhow!("Invalid duration for {}: {}", key, e))?;
            config.working_hours.monday = new_duration;
            let new_value = Value::String(format_duration(new_duration).to_string());
            Ok((old_value, new_value))
        }
        "working_hours.sunday" => {
            let old_value = Value::String(format_duration(config.working_hours.sunday).to_string());
            let new_duration = parse_duration(value_str)
                .map_err(|e| anyhow::anyhow!("Invalid duration for {}: {}", key, e))?;
            config.working_hours.sunday = new_duration;
            let new_value = Value::String(format_duration(new_duration).to_string());
            Ok((old_value, new_value))
        }
        _ => bail!("Unknown config key: {}", key),
    }
}
