#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_resolved;

mod component;
pub mod evalexpr;
mod expression;
mod function;
mod kind;
mod module_thing;
mod or_type;
mod record;
pub mod tdoc;
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
    // SymbolAlias {
    //     symbol: String,
    //     alias: String,
    //     line_number: usize,
    // },
    // ModuleAlias {
    //     module: String,
    //     alias: String,
    //     line_number: usize,
    // },
    Record(fastn_resolved::Record),
    OrType(fastn_resolved::OrType),
    OrTypeWithVariant {
        or_type: String,
        variant: fastn_resolved::OrTypeVariant,
    },
    Variable(fastn_resolved::Variable),
    Component(fastn_resolved::ComponentDefinition),
    WebComponent(fastn_resolved::WebComponentDefinition),
    Function(fastn_resolved::Function),
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
            fastn_resolved::Definition::Record(r) => r.name.clone(),
            fastn_resolved::Definition::OrType(o) => o.name.clone(),
            fastn_resolved::Definition::OrTypeWithVariant { or_type, .. } => or_type.clone(),
            fastn_resolved::Definition::Variable(v) => v.name.to_string(),
            fastn_resolved::Definition::Component(c) => c.name.to_string(),
            fastn_resolved::Definition::Function(f) => f.name.to_string(),
            fastn_resolved::Definition::WebComponent(w) => w.name.to_string(),
            fastn_resolved::Definition::Export { to, .. } => to.to_string(),
            // TODO: check if the following two are valid
            // Definition::SymbolAlias { alias, .. } => alias.to_string(),
            // Definition::ModuleAlias { alias, .. } => alias.to_string(),
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
            // Definition::SymbolAlias { line_number, .. } => *line_number,
            // Definition::ModuleAlias { line_number, .. } => *line_number,
        }
    }

    pub fn component(self) -> Option<fastn_resolved::ComponentDefinition> {
        match self {
            fastn_resolved::Definition::Component(v) => Some(v),
            _ => None,
        }
    }
}

pub struct CompiledDocument {
    pub content: Vec<fastn_resolved::ComponentInvocation>,
    pub definitions: indexmap::IndexMap<String, fastn_resolved::Definition>,
}
