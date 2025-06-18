// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::output::{DisplayOutput, print_output};
use crate::persistence::{load_frames, save_frames};
use crate::{Format, TagArgs, TagCommands};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct ListOutput {
    tags: Vec<String>,
}

impl DisplayOutput for ListOutput {
    fn to_text(&self) -> String {
        self.tags.join("\n")
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct RemoveOutput {
    tag: String,
}

impl DisplayOutput for RemoveOutput {
    fn to_text(&self) -> String {
        format!("Tag '{}' removed from all frames.", self.tag)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct RenameOutput {
    old_name: String,
    new_name: String,
}

impl DisplayOutput for RenameOutput {
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
            print_output(&output, format)?;
        }
        TagCommands::Remove { tag } => {
            frames.remove_tag(tag);
            save_frames(config_path, &frames)?;

            let output = RemoveOutput {
                tag: tag.to_string(),
            };

            print_output(&output, format)?;
        }
        TagCommands::Rename { old_name, new_name } => {
            frames.rename_tag(old_name, new_name);
            save_frames(config_path, &frames)?;

            let output = RenameOutput {
                old_name: old_name.to_string(),
                new_name: new_name.to_string(),
            };

            print_output(&output, format)?;
        }
    };

    Ok(())
}
