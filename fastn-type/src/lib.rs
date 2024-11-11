#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_type;

mod function;
pub use function::FunctionCall;
mod value;
pub use value::{PropertyValue, PropertyValueSource, Value};
mod kind;
pub use kind::{Kind, KindData};

pub type Map<T> = std::collections::BTreeMap<String, T>;
