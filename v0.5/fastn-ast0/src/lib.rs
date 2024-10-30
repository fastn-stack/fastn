#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

mod import;
pub use import::{Export, Exposing, Import};

extern crate self as fastn_ast0;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Ast0 {
    Import(fastn_ast0::Import),
}
