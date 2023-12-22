#[cfg(test)]
#[macro_use]
mod test;

mod component;
mod constants;
mod function;
mod import;
mod kind;
mod main;
mod or_type;
mod record;
pub(crate) mod utils;
mod variable;
mod web_component;

pub use component::{
    Argument, Component, ComponentDefinition, Event, Loop, Property, PropertySource,
};
pub use constants::ALWAYS_INCLUDE;
pub use function::Function;
pub use import::{Export, Exposing, Import};
pub use kind::{
    BodyValue, Condition, HeaderValues, ValueSource, VariableKind, VariableModifier, VariableValue,
    NULL,
};
pub use main::AST;
pub use or_type::{OrType, OrTypeVariant};
pub use record::{Field, Record};
pub use variable::{VariableDefinition, VariableFlags, VariableInvocation};
pub use web_component::WebComponentDefinition;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("P1Error: {}", _0)]
    P1(#[from] ftd::p1::Error),

    #[error("ASTParseError: {doc_id}:{line_number} -> {message}")]
    Parse {
        message: String,
        doc_id: String,
        line_number: usize,
    },

    #[error("ParseBoolError: {}", _0)]
    ParseBool(#[from] std::str::ParseBoolError),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn parse_error<T, S1>(m: S1, doc_id: &str, line_number: usize) -> ftd::ast::Result<T>
where
    S1: Into<String>,
{
    Err(Error::Parse {
        message: m.into(),
        doc_id: doc_id.to_string(),
        line_number,
    })
}
