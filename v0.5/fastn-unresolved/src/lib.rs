#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_unresolved;

mod parser;
mod utils;

pub use parser::parse;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub module_doc: Option<fastn_section::Span>,
    pub imports: Vec<fastn_unresolved::Import>,
    pub definitions: std::collections::HashMap<fastn_section::Identifier, Definition>,
    pub content: Vec<fastn_section::Section>,
    pub errors: Vec<fastn_section::Spanned<fastn_section::Error>>,
    pub warnings: Vec<fastn_section::Spanned<fastn_section::Warning>>,
    pub comments: Vec<fastn_section::Span>,
    pub line_starts: Vec<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Definition {
    Component(fastn_section::Section),
    Variable(fastn_section::Section),
    Function(fastn_section::Section),
    TypeAlias(fastn_section::Section),
    Record(fastn_section::Section),
    OrType(fastn_section::Section),
    Module(fastn_section::Section),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Import {
    pub module: fastn_section::ModuleName,
    pub exports: Option<Export>,
    pub exposing: Option<Export>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Export {
    All,
    Things(Vec<fastn_section::AliasableIdentifier>),
}
