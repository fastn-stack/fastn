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

mod variable;
pub use variable::{ConditionalValue, Variable};

mod web_component;
pub use web_component::WebComponentDefinition;

mod or_type;
pub use or_type::{OrType, OrTypeVariant};

pub type Map<T> = std::collections::BTreeMap<String, T>;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Thing {
    Record(fastn_type::Record),
    OrType(fastn_type::OrType),
    OrTypeWithVariant {
        or_type: String,
        variant: fastn_type::OrTypeVariant,
    },
    Variable(fastn_type::Variable),
    Component(fastn_type::ComponentDefinition),
    WebComponent(fastn_type::WebComponentDefinition),
    Function(fastn_type::Function),
    Export {
        from: String,
        to: String,
        line_number: usize,
    },
}

impl Thing {
    pub fn name(&self) -> String {
        match self {
            fastn_type::Thing::Record(r) => r.name.clone(),
            fastn_type::Thing::OrType(o) => o.name.clone(),
            fastn_type::Thing::OrTypeWithVariant { or_type, .. } => or_type.clone(),
            fastn_type::Thing::Variable(v) => v.name.to_string(),
            fastn_type::Thing::Component(c) => c.name.to_string(),
            fastn_type::Thing::Function(f) => f.name.to_string(),
            fastn_type::Thing::WebComponent(w) => w.name.to_string(),
            fastn_type::Thing::Export { to, .. } => to.to_string(),
        }
    }

    pub fn line_number(&self) -> usize {
        match self {
            Thing::Record(r) => r.line_number,
            Thing::Variable(v) => v.line_number,
            Thing::Component(c) => c.line_number,
            Thing::Function(f) => f.line_number,
            Thing::OrType(o) => o.line_number,
            Thing::OrTypeWithVariant { variant, .. } => variant.line_number(),
            Thing::WebComponent(w) => w.line_number,
            Thing::Export { line_number, .. } => *line_number,
        }
    }

    pub fn component(self) -> Option<fastn_type::ComponentDefinition> {
        match self {
            fastn_type::Thing::Component(v) => Some(v),
            _ => None,
        }
    }
}
