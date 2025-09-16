#![warn(unused_extern_crates)]
#![deny(unused_crate_dependencies)]

extern crate self as fastn_ssh;

use clap as _; // used by main for CLI
use fastn_p2p as _; // used by main for macro
use tokio as _; // used by main for macro
use tracing_subscriber as _; // used by main macro for logging

mod cli;
mod init;
mod run;

pub use cli::{Cli, handle_cli};
pub use init::init;
pub use run::run;

/// Start SSH listener (for developer testing)
pub async fn listen(private_key_path: &std::path::Path, allowed: &str) {
    todo!("Start SSH listener with key {private_key_path:?} and allowed {allowed}");
}

/// Connect to SSH server (for developer testing)
pub async fn connect(private_key_path: &std::path::Path, target: &str) {
    todo!("Connect to {target} using key {private_key_path:?}");
}
