use clap::{Parser, Subcommand};

/// Command-line interface definition
#[derive(Parser)]
#[command(
    name = "mytool",
    version = "0.1.0",
    about = "Splits large logs into .zst chunks with a BinaryFuse index, then searches them."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Build an index from a large log
    Build {
        /// Path to input log file
        input: String,

        /// Optional output .zst file
        #[arg(long = "zst", short = 'z')]
        zst: Option<String>,

        /// Optional output .idx file
        #[arg(long = "idx", short = 'i')]
        idx: Option<String>,
    },
    /// Search within existing .zst + .idx files
    Search {
        /// Path to .zst file (local path or gs://bucket/path)
        zst: String,

        /// Optional path to .idx file (local path or gs://bucket/path)
        #[arg(long = "idx", short = 'i')]
        idx: Option<String>,

        /// The search pattern (string or bytes)
        pattern: String,
    },
}
