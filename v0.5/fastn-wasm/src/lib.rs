#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_wasm;

pub mod aws;
pub mod crypto;
pub mod ds;
pub mod env;
pub mod helpers;
pub mod http;
pub mod macros;
pub mod register;
mod sqlite;
mod store;

pub use http::send_request::send_request;
pub use store::{Conn, ConnectionExt, Store, StoreExt};

pub static WASM_ENGINE: once_cell::sync::Lazy<wasmtime::Engine> =
    once_cell::sync::Lazy::new(|| {
        wasmtime::Engine::new(wasmtime::Config::new().async_support(true)).unwrap()
    });
