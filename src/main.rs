use anyhow::Result;
use clap::Parser;
use ebb::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    println!("cli: {:?}", cli);
    ebb::run(&cli)
}
