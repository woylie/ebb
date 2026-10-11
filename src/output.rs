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
        Format::Text => replace_control_chars(&output.to_text()),
    };
    println!("{}", output_string);
    Ok(())
}

pub fn replace_control_chars(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_control() && c != '\n' {
                char::REPLACEMENT_CHARACTER
            } else {
                c
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_control_chars_keeps_line_breaks() {
        assert_eq!(
            replace_control_chars("a\u{1b}[2J\tb\r\nc\u{7f}\u{85}é"),
            "a\u{fffd}[2J\u{fffd}b\u{fffd}\nc\u{fffd}\u{fffd}é"
        );
    }
}
