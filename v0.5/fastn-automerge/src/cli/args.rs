use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fastn-automerge")]
#[command(about = "A CLI for managing Automerge CRDT documents")]
#[command(version)]
pub struct Cli {
    /// Use custom database path
    #[arg(long, global = true, default_value = "fastn-automerge.sqlite")]
    pub db: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new database
    Init,
    /// Create a new document
    Create {
        /// Document path
        path: String,
        /// JSON data (if not using --file)
        json: Option<String>,
        /// Read JSON from file
        #[arg(long)]
        file: Option<String>,
    },
    /// Read a document as JSON
    Get {
        /// Document path
        path: String,
        /// Pretty-print JSON output
        #[arg(long)]
        pretty: bool,
        /// Write output to file
        #[arg(long)]
        output: Option<String>,
    },
    /// Update an existing document
    Update {
        /// Document path
        path: String,
        /// JSON data
        json: String,
    },
    /// Create or replace a document
    Set {
        /// Document path
        path: String,
        /// JSON data
        json: String,
    },
    /// Delete a document
    Delete {
        /// Document path
        path: String,
        /// Skip confirmation prompt
        #[arg(long)]
        confirm: bool,
    },
    /// List all documents
    List {
        /// Filter by path prefix
        #[arg(long)]
        prefix: Option<String>,
        /// Show detailed information
        #[arg(long)]
        details: bool,
    },
    /// Clean up database (remove all documents)
    Clean {
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Show document edit history
    History {
        /// Document path
        path: String,
        /// Specific commit hash (optional)
        commit_hash: Option<String>,
        /// Show short format
        #[arg(long)]
        short: bool,
    },
    /// Show document metadata
    Info {
        /// Document path
        path: String,
    },
}
