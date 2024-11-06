#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_lang;

mod compile;
mod ds;
pub mod resolved;

pub use compile::compile;
pub use ds::DocumentStore;
pub use fastn_section::Result;

pub struct UISpec {
    pub title: String,
    pub body: String,
}

pub enum Output {
    UI(UISpec),
    Data(serde_json::Value),
}
