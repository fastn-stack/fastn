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
pub enum Definition {
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
    /// what is this?
    Export {
        from: String,
        to: String,
        line_number: usize,
    },
}

impl Definition {
    pub fn name(&self) -> String {
        match self {
            fastn_type::Definition::Record(r) => r.name.clone(),
            fastn_type::Definition::OrType(o) => o.name.clone(),
            fastn_type::Definition::OrTypeWithVariant { or_type, .. } => or_type.clone(),
            fastn_type::Definition::Variable(v) => v.name.to_string(),
            fastn_type::Definition::Component(c) => c.name.to_string(),
            fastn_type::Definition::Function(f) => f.name.to_string(),
            fastn_type::Definition::WebComponent(w) => w.name.to_string(),
            fastn_type::Definition::Export { to, .. } => to.to_string(),
        }
    }

    pub fn line_number(&self) -> usize {
        match self {
            Definition::Record(r) => r.line_number,
            Definition::Variable(v) => v.line_number,
            Definition::Component(c) => c.line_number,
            Definition::Function(f) => f.line_number,
            Definition::OrType(o) => o.line_number,
            Definition::OrTypeWithVariant { variant, .. } => variant.line_number(),
            Definition::WebComponent(w) => w.line_number,
            Definition::Export { line_number, .. } => *line_number,
        }
    }

    pub fn component(self) -> Option<fastn_type::ComponentDefinition> {
        match self {
            fastn_type::Definition::Component(v) => Some(v),
            _ => None,
        }
    }
}
