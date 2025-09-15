#![warn(unused_extern_crates)]
#![deny(unused_crate_dependencies)]

extern crate self as fastn_daemon;

use tokio as _; // only main uses this for now

mod cli;

pub use cli::{Cli, Commands, add_subcommands, handle_cli, handle_daemon_commands};
