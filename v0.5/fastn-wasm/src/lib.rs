#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_wasm;

pub mod aws;
pub mod crypto;
pub mod ds;
pub mod env;
pub mod helpers;
pub mod send_request;
mod store;

pub use send_request::send_request;
pub use store::Store;
