#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_unresolved;

#[cfg(test)]
mod debug;
mod parser;
mod utils;

pub use parser::parse;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub module_doc: Option<fastn_section::Span>,
    pub imports: Vec<fastn_unresolved::Import>,
    pub definitions: Vec<Definition>,
    pub content: Vec<ComponentInvocation>,
    pub errors: Vec<fastn_section::Spanned<fastn_section::Error>>,
    pub warnings: Vec<fastn_section::Spanned<fastn_section::Warning>>,
    pub comments: Vec<fastn_section::Span>,
    pub line_starts: Vec<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Definition {
    pub doc: Option<fastn_section::Span>,
    pub name: Identifier,
    pub visibility: fastn_section::Visibility,
    pub kind: Kind,
    pub inner: InnerDefinition,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InnerDefinition {
    Component {
        properties: Vec<Argument>,
        body: Vec<ComponentInvocation>,
    },
    Variable {
        arguments: Vec<Property>,
        caption: Vec<fastn_section::Tes>,
    },
    Function {
        properties: Vec<Argument>,
        return_type: Option<Kind>,
        body: Vec<fastn_section::Tes>,
    },
    // -- type foo: person
    // name: foo ;; we are updating / setting the default value
    TypeAlias {
        kind: Kind,
        arguments: Vec<Property>,
    },
    Record {
        properties: Vec<Argument>,
    },
    // TODO: OrType(fastn_section::Section),
    // TODO: Module(fastn_section::Section),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Import {
    pub package: PackageName,
    pub module: ModuleName,
    pub alias: Option<Identifier>,
    pub export: Option<Export>,
    pub exposing: Option<Export>,
}

// #[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
// pub struct ComponentInvocation {
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub id: Option<String>,
//     pub name: String,
//     pub properties: Vec<Property>,
//     pub iteration: Box<Option<Loop>>,
//     pub condition: Box<Option<fastn_type::Expression>>,
//     pub events: Vec<Event>,
//     pub children: Vec<ComponentInvocation>,
//     pub source: ComponentSource,
//     pub line_number: usize,
// }

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum UR<U, R> {
    Resolved(R),
    UnResolved(U),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ComponentInvocation {
    pub name: UR<Identifier, Identifier>,
    /// once a caption is resolved it is set to () here, and moved to properties
    pub caption: UR<Option<fastn_section::HeaderValue>, ()>,
    pub properties: Vec<UR<Property, ()>>,
    pub body: Vec<fastn_section::Tes>,
    pub children: Vec<ComponentInvocation>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Property {
    pub name: Identifier,
    pub value: Vec<fastn_section::Tes>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Argument {
    pub name: Identifier,
    pub kind: Kind,
    pub visibility: fastn_section::Visibility,
    pub default: Option<fastn_section::Tes>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PackageName(pub String);

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ModuleName(pub String);

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Identifier(pub String);

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Export {
    All,
    Things(Vec<AliasableIdentifier>),
}

/// is this generic enough?
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AliasableIdentifier {
    pub alias: Option<Identifier>,
    pub name: Identifier,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SymbolName {
    pub package: PackageName,
    pub module: ModuleName,
    /// can name contain dots? after we have `-- module foo:` feature it will, but now?
    pub name: Identifier, // name comes after #
}

/// We cannot have kinds of like Record(SymbolName), OrType(SymbolName), because they are not
/// yet "resolved", eg `-- foo x:`, we do not know if `foo` is a record or an or-type.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Kind {
    Integer,
    Decimal,
    String,
    Boolean,
    Option(Box<Kind>),
    // TODO: Map(Kind, Kind),
    List(Box<Kind>),
    Caption(Box<Kind>),
    Body(Box<Kind>),
    CaptionOrBody(Box<Kind>),
    // TODO: Future(Kind),
    // TODO: Result(Kind, Kind),
    Custom(SymbolName),
}
