#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

pub mod commands;
mod ds;

extern crate self as fastn;

pub enum Action {
    Read,
    Write,
}

pub enum OutputRequested {
    UI,
    Data,
}
