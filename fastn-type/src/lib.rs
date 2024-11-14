#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_type;

mod component;
pub mod evalexpr;
mod expression;
mod function;
mod kind;
mod module_thing;
mod or_type;
mod record;
mod value;
mod variable;
mod web_component;

pub use component::{
    Argument, ComponentDefinition, ComponentInvocation, ComponentSource, Event, EventName, Loop,
    Property, PropertySource,
};
pub use expression::Expression;
pub use function::{Function, FunctionCall, FunctionExpression};
pub use kind::{Kind, KindData};
pub use module_thing::ModuleThing;
pub use or_type::{OrType, OrTypeVariant};
pub use record::{AccessModifier, Field, Record};
pub use value::{PropertyValue, PropertyValueSource, Value};
pub use variable::{ConditionalValue, Variable};
pub use web_component::WebComponentDefinition;
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
