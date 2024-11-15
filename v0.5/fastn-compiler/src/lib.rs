#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_compiler;

mod compiler;
mod symbols;

pub use compiler::compile;
pub use fastn_section::Result;
pub use symbols::{LookupResult, SymbolStore};

pub struct UISpec {
    pub title: String,
    pub body: String,
}

pub enum Output {
    UI(UISpec),
    Data(serde_json::Value),
}
