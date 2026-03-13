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

    /// Collection to operate on (default: "default")
    #[arg(long, global = true, default_value = "default")]
    pub collection: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Cli {
        Cli::parse_from(args)
    }

    #[test]
    fn write_with_text() {
        let cli = parse(&["semlocal", "write", "hello"]);
        assert!(matches!(cli.command, Command::Write { text: Some(ref t) } if t == "hello"));
    }

    #[test]
    fn search_defaults() {
        let cli = parse(&["semlocal", "search", "query"]);
        match cli.command {
            Command::Search { top, json, .. } => {
                assert_eq!(top, 5);
                assert!(!json);
            }
            _ => panic!("expected Search"),
        }
    }

    #[test]
    fn search_with_options() {
        let cli = parse(&["semlocal", "search", "query", "--top", "3", "--json"]);
        match cli.command {
            Command::Search { top, json, .. } => {
                assert_eq!(top, 3);
                assert!(json);
            }
            _ => panic!("expected Search"),
        }
    }

    #[test]
    fn delete_with_id() {
        let cli = parse(&["semlocal", "delete", "abc-123"]);
        assert!(matches!(cli.command, Command::Delete { ref id } if id == "abc-123"));
    }

    #[test]
    fn default_src_and_collection() {
        let cli = parse(&["semlocal", "write", "text"]);
        assert_eq!(cli.src, PathBuf::from(".semlocal"));
        assert_eq!(cli.collection, "default");
    }

    #[test]
    fn custom_src_and_collection() {
        let cli = parse(&["semlocal", "--src", "/tmp/idx", "--collection", "notes", "write", "text"]);
        assert_eq!(cli.src, PathBuf::from("/tmp/idx"));
        assert_eq!(cli.collection, "notes");
    }

    #[test]
    fn global_flags_after_subcommand() {
        let cli = parse(&["semlocal", "search", "query", "--collection", "docs", "--src", "/data"]);
        assert_eq!(cli.collection, "docs");
        assert_eq!(cli.src, PathBuf::from("/data"));
    }
}
