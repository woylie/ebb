// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::persistence::{load_frames, save_frames};
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

#[derive(Debug, Deserialize, Serialize)]
pub struct RenameOutput {
    pub old_name: String,
    pub new_name: String,
}

impl RenameOutput {
    fn to_text(&self) -> String {
        format!(
            "Project renamed from '{}' to '{}'.",
            self.old_name, self.new_name
        )
    }
}

pub fn run_project(args: &ProjectArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    let mut frames = load_frames(config_path)?;

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
        ProjectCommands::Rename { old_name, new_name } => {
            frames.rename_project(old_name, new_name);
            save_frames(config_path, &frames)?;

            let output = RenameOutput {
                old_name: old_name.to_string(),
                new_name: new_name.to_string(),
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
