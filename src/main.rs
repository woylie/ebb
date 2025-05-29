// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::Parser;
use ebb::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    ebb::run(&cli)
}
