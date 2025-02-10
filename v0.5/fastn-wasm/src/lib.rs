#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_wasm;

pub(crate) mod aws;
pub(crate) mod crypto;
pub(crate) mod ds;
mod email;
pub(crate) mod env;
pub(crate) mod helpers;
pub(crate) mod http;
pub(crate) mod macros;
pub mod pg;
mod process_http_request;
pub(crate) mod register;
mod sqlite;
mod store;

pub use process_http_request::{handle, process_http_request, WasmError};
pub(crate) use store::Conn;
pub use store::{ConnectionExt, SQLError, Store, StoreExt, StoreImpl};
pub use store::{
    FASTN_APP_URLS_HEADER, FASTN_APP_URL_HEADER, FASTN_MAIN_PACKAGE_HEADER,
    FASTN_WASM_PACKAGE_HEADER,
};

pub static WASM_ENGINE: once_cell::sync::Lazy<wasmtime::Engine> =
    once_cell::sync::Lazy::new(|| {
        wasmtime::Engine::new(wasmtime::Config::new().async_support(true)).unwrap()
    });

pub fn insert_or_update<K, V>(map: &scc::HashMap<K, V>, key: K, value: V)
where
    K: std::hash::Hash,
    K: std::cmp::Eq,
{
    match map.entry(key) {
        scc::hash_map::Entry::Occupied(mut ov) => {
            ov.insert(value);
        }
        scc::hash_map::Entry::Vacant(vv) => {
            vv.insert_entry(value);
        }
    }
}
