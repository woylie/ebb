// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::persistence::{load_config, save_config};
use crate::types::Config;
use crate::{ConfigArgs, ConfigCommands, Format};
use serde::Serialize;
use serde_json::{Map, Value};
use std::path::Path;
use tabled::{settings::Style, Table, Tabled};

#[derive(Serialize)]
struct GetOutput<'a> {
    key: &'a String,
    value: Value,
}

impl GetOutput<'_> {
    fn to_text(&self) -> String {
        match &self.value {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        }
    }
}

#[derive(Serialize)]
struct ListOutput {
    config: Config,
}

#[derive(Tabled)]
struct ConfigRow {
    #[tabled(rename = "Key")]
    key: String,
    #[tabled(rename = "Value")]
    value: String,
}

impl ListOutput {
    fn to_text(&self) -> String {
        let json_value = serde_json::to_value(&self.config).expect("Serialization should succeed");
        let mut flat = vec![];
        flatten_value("".to_string(), &json_value, &mut flat);

        let rows: Vec<ConfigRow> = flat
            .into_iter()
            .map(|(key, value)| ConfigRow { key, value })
            .collect();

        Table::new(rows).with(Style::sharp()).to_string()
    }
}

#[derive(Serialize)]
struct SetOutput {
    key: String,
    old_value: Value,
    new_value: Value,
}

impl SetOutput {
    fn to_text(&self) -> String {
        let old_value = match &self.old_value {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };

        let new_value = match &self.new_value {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };

        format!(
            "Key: {}\nOld value: {}\nNew value: {}",
            self.key, old_value, new_value
        )
    }
}

pub fn run_config(args: &ConfigArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    let mut config = load_config(config_path)?;

    match &args.command {
        ConfigCommands::Get { key } => {
            let value = get_config_value(&config, key)?;
            let output = GetOutput { key, value };

            let output_string = match format {
                Format::Json => serde_json::to_string_pretty(&output)?,
                Format::Text => output.to_text(),
            };

            println!("{}", output_string);
        }

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
                key: key.to_string(),
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

fn flatten_value(prefix: String, value: &Value, output: &mut Vec<(String, String)>) {
    if prefix == "vacation_days_per_year" || prefix == "sick_days_per_year" {
        if let Value::Object(map) = value {
            let mut sorted: Vec<_> = map.iter().collect();
            sorted.sort_by_key(|(k, _)| k.parse::<i32>().unwrap_or_default());
            for (year, days) in sorted {
                let key = format!("{}.{}", prefix, year);
                flatten_value(key, days, output);
            }
        }
        return;
    }

    if prefix == "working_hours" {
        if let Value::Object(map) = value {
            let ordered_days = [
                "monday",
                "tuesday",
                "wednesday",
                "thursday",
                "friday",
                "saturday",
                "sunday",
            ];
            for day in &ordered_days {
                if let Some(v) = map.get(*day) {
                    let key = format!("{}.{}", prefix, day);
                    flatten_value(key, v, output);
                }
            }
        }
        return;
    }

    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let new_prefix = if prefix.is_empty() {
                    k.to_string()
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten_value(new_prefix, v, output);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                flatten_value(format!("{}[{}]", prefix, i), v, output);
            }
        }
        _ => {
            let val = match value {
                Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            output.push((prefix, val));
        }
    }
}

fn get_config_value(config: &Config, key: &str) -> anyhow::Result<Value> {
    let json_value = serde_json::to_value(config)?;
    let mut current = &json_value;

    for part in key.split('.') {
        match current {
            Value::Object(map) => {
                if let Some(next) = map.get(part) {
                    current = next;
                } else {
                    anyhow::bail!("Key '{}' not found", key);
                }
            }
            _ => anyhow::bail!("Key '{}' not found", key),
        }
    }

    Ok(current.clone())
}

fn set_config_value(
    config: &mut Config,
    key: &str,
    value_str: &str,
) -> anyhow::Result<(Value, Value)> {
    let mut json_value = serde_json::to_value(&config)?;
    let parts: Vec<&str> = key.split('.').collect();
    let parent = get_mut_parent(&mut json_value, &parts)?;
    let last_key = parts.last().unwrap();
    let old_value = parent.get(*last_key).cloned().unwrap_or(Value::Null);
    let json_val =
        if key.starts_with("vacation_days_per_year.") || key.starts_with("sick_days_per_year.") {
            let n = value_str
                .parse::<i32>()
                .map_err(|e| anyhow::anyhow!("Invalid integer for {}: {}", key, e))?;
            serde_json::Value::Number(n.into())
        } else {
            serde_json::Value::String(value_str.to_string())
        };

    parent.insert(last_key.to_string(), json_val);
    *config = serde_json::from_value(json_value)?;
    Ok((old_value, Value::String(value_str.to_string())))
}

fn get_mut_parent<'a>(
    root: &'a mut Value,
    parts: &[&str],
) -> anyhow::Result<&'a mut Map<String, Value>> {
    let mut current = root;

    for part in &parts[..parts.len() - 1] {
        let obj = current
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("Expected object while traversing key path"))?;

        current = obj
            .get_mut(*part)
            .ok_or_else(|| anyhow::anyhow!("Key part '{}' not found", part))?;
    }

    current
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("Expected object at key path parent"))
}
