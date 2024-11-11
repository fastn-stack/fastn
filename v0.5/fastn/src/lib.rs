#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn;

pub mod commands;
mod symbols;

pub use symbols::Symbols;

pub enum Action {
    Read,
    Write,
}

pub enum OutputRequested {
    UI,
    Data,
}
