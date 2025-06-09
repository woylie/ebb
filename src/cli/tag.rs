// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::persistence::{load_frames, save_frames};
use crate::{Format, TagArgs, TagCommands};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct ListOutput {
    pub tags: Vec<String>,
}

impl ListOutput {
    fn to_text(&self) -> String {
        self.tags.join("\n")
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RemoveOutput {
    pub tag: String,
}

impl RemoveOutput {
    fn to_text(&self) -> String {
        format!("Tag '{}' removed from all frames.", self.tag)
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
            "Tag renamed from '{}' to '{}'.",
            self.old_name, self.new_name
        )
    }
}

pub fn run_tag(args: &TagArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    let mut frames = load_frames(config_path)?;

    match &args.command {
        TagCommands::List => {
            let tags = frames.all_tags();
            let output = ListOutput { tags };

            let output_string = match format {
                Format::Json => serde_json::to_string_pretty(&output)?,
                Format::Text => output.to_text(),
            };

            println!("{}", output_string);
        }
        TagCommands::Remove { tag } => {
            frames.remove_tag(tag);
            save_frames(config_path, &frames)?;

            let output = RemoveOutput {
                tag: tag.to_string(),
            };

            let output_string = match format {
                Format::Json => serde_json::to_string_pretty(&output)?,
                Format::Text => output.to_text(),
            };

            println!("{}", output_string);
        }
        TagCommands::Rename { old_name, new_name } => {
            frames.rename_tag(old_name, new_name);
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
