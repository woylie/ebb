use anyhow::Result;
use clap::Parser;
use ebb::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    ebb::run(&cli)
}
