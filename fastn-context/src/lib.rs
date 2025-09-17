#![warn(unused_extern_crates)]
#![deny(unused_crate_dependencies)]

use tokio as _; // used by main macro
use tokio_util as _; // used for cancellation tokens

mod context;
mod status;

pub use context::{Context, ContextBuilder, global};
pub use status::{ContextStatus, Status, status, status_with_latest};

// Re-export main macro
pub use fastn_context_macros::main;
