#![warn(unused_extern_crates)]
#![deny(unused_crate_dependencies)]

extern crate self as fastn_daemon;

use fastn_id52 as _;
use fastn_p2p as _; // used by main for macro
use tokio as _; // only main uses this for now
use tracing_subscriber as _; // used by main macro for logging // used by remote module

mod cli;
mod init;
mod run;
mod ssh;
mod status;

pub use cli::{Cli, Commands, add_subcommands, handle_cli, handle_daemon_commands};
pub use init::init;
pub use run::run;
pub use ssh::ssh;
pub use status::status;
