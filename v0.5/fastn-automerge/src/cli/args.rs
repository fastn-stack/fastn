use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fastn-automerge")]
#[command(about = "A CLI for managing Automerge CRDT documents")]
#[command(version)]
pub struct Cli {
    /// Use custom database path
    #[arg(long, global = true, default_value = "automerge.sqlite")]
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
    // TODO: Add actor ID management commands for proper multi-device support:
    // - set-actor-id <entity_id52> <device_number> - Set the database actor ID
    // - next-actor-id <entity_id52> - Get next device number for entity
    // - get-actor-id - Show current database actor ID
    // - actor-info - Show database identity and actor counter status
    // These commands are needed to replace the dummy "cli-dummy-entity" usage
}
