// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::Parser;
use ebb::Cli;
use ebb::output::replace_control_chars;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match ebb::run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {}", replace_control_chars(&format!("{error:?}")));
            ExitCode::FAILURE
        }
    }
}
