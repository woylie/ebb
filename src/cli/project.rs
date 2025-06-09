// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::persistence::load_frames;
use crate::{Format, ProjectArgs, ProjectCommands};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct ListOutput {
    pub projects: Vec<String>,
}

impl ListOutput {
    fn to_text(&self) -> String {
        self.projects.join("\n")
    }
}

pub fn run_project(args: &ProjectArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    let frames = load_frames(config_path)?;

    match &args.command {
        ProjectCommands::List => {
            let projects = frames.all_projects();
            let output = ListOutput { projects };

            let output_string = match format {
                Format::Json => serde_json::to_string_pretty(&output)?,
                Format::Text => output.to_text(),
            };

            println!("{}", output_string);
        }
    };

    Ok(())
}
