#![warn(unused_extern_crates)]
#![deny(unused_crate_dependencies)]

extern crate self as fastn_remote;

use clap as _; // used by main for CLI
use fastn_p2p as _; // used by main for macro
use tokio as _; // used by main for macro
use tracing_subscriber as _; // used by main macro for logging

mod cli;
mod init;
mod listen;
mod rexec;
mod rshell;
mod run;

pub use cli::{Cli, handle_cli, rexec_cli, rshell_cli};
pub use init::init;
pub use listen::{listen, listen_cli};
pub use rexec::rexec;
pub use rshell::rshell;
pub use run::run;
