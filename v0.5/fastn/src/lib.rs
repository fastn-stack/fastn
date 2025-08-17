#![deny(unused_crate_dependencies)]
#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn;

// we are adding this so we can continue to use unused_crate_dependencies, as only
// main depends on tokio, and if we do not have the following unused_crate_dependencies
// complains
use fastn_observer as _;
use fastn_rig as _;

pub mod commands;
mod definition_provider;
mod section_provider;

pub use section_provider::SectionProvider;

pub enum Action {
    Read,
    Write,
}

pub enum OutputRequested {
    UI,
    Data,
}
