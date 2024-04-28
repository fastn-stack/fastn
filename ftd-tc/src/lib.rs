#![allow(unused)]
#![deny(unused_crate_dependencies)]

extern crate self as ftd_tc;

mod component_invocation;
mod error;
mod parser;
mod state;
mod types;

pub use component_invocation::{ComponentResolvable, CI};
pub use error::*;
pub use parser::parse_document_to_ast;
pub use state::{State, TCState};
pub use types::*;
