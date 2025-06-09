// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::persistence::load_frames;
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

pub fn run_tag(args: &TagArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    let frames = load_frames(config_path)?;

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
    };

    Ok(())
}
