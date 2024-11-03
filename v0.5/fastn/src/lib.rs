#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn;

pub enum Action {
    Read,
    Write,
}

pub enum OutputRequested {
    UI,
    Data,
}

async fn serve(
    _config: fastn_core::Config,
    _path: &str,
    _data: serde_json::Value,
) -> fastn_lang::Output {
    todo!()
}
