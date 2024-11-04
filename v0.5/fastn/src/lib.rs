#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn;

pub mod commands;
mod ds;

pub use ds::DS;

pub enum Action {
    Read,
    Write,
}

pub enum OutputRequested {
    UI,
    Data,
}
