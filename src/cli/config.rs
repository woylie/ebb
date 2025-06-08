// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::persistence::load_config;
use crate::types::Config;
use crate::{ConfigArgs, ConfigCommands, Format};
use humantime::{format_duration, FormattedDuration};
use serde::Serialize;
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

pub fn run_config(args: &ConfigArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    let config = load_config(config_path)?;

    match &args.command {
        ConfigCommands::List => {
            let output = ListOutput { config: config };

            let output_string = match format {
                Format::Json => serde_json::to_string_pretty(&output)?,
                Format::Text => output.to_text(),
            };

            println!("{}", output_string);
        }
    };

    Ok(())
}
