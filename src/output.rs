// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::Format;
use serde::Serialize;

pub trait DisplayOutput: Serialize {
    fn to_text(&self) -> String;
}

pub fn print_output<O: DisplayOutput>(output: &O, format: &Format) -> anyhow::Result<()> {
    let output_string = match format {
        Format::Json => serde_json::to_string_pretty(output)?,
        Format::Text => output.to_text(),
    };
    println!("{}", output_string);
    Ok(())
}
