#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_type;

mod function;
pub use function::{Function, FunctionCall, FunctionExpression};
mod value;
pub use value::{PropertyValue, PropertyValueSource, Value};
mod kind;
pub use kind::{Kind, KindData};

mod component;
pub use component::{
    Argument, Component, ComponentDefinition, ComponentSource, Event, EventName, Loop, Property,
    PropertySource,
};

mod expression;
pub use expression::Expression;

mod record;
pub use record::{AccessModifier, Field, Record};

mod module_thing;
pub use module_thing::ModuleThing;

pub type Map<T> = std::collections::BTreeMap<String, T>;
