use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "semlocal", about = "Local semantic search — no backend required.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Directory to store the index (default: .semlocal in current directory)
    #[arg(long, global = true, default_value = ".semlocal")]
    pub src: PathBuf,
}

#[derive(Subcommand)]
pub enum Command {
    /// Store a piece of text and return its ID
    Write {
        /// The text to store (reads from stdin if omitted or "-")
        text: Option<String>,
    },
    /// Search for semantically similar text
    Search {
        /// The query text (reads from stdin if omitted or "-")
        text: Option<String>,

        /// Number of results to return
        #[arg(long, default_value_t = 5)]
        top: usize,

        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },
    /// Delete an entry by ID
    Delete {
        /// The UUID of the entry to delete
        id: String,
    },
}
