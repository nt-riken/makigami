mod cli;
mod build;
mod search;
mod utils;
mod fastu64set;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> std::io::Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    match &cli.command {
        Commands::Build { input, zst, idx } => {
            // Build subcommand
            build::run_build(input, zst.as_deref(), idx.as_deref())?;
        }
        Commands::Search { zst, idx, pattern } => {
            // Search subcommand
            search::run_search(zst, idx.as_deref(), pattern)?;
        }
    }

    Ok(())
}
