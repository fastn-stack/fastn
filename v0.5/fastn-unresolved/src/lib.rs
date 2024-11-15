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
    pub definitions: Vec<UR<Definition, fastn_type::Definition>>,
    pub content: Vec<UR<ComponentInvocation, fastn_type::ComponentInvocation>>,
    pub errors: Vec<fastn_section::Spanned<fastn_section::Error>>,
    pub warnings: Vec<fastn_section::Spanned<fastn_section::Warning>>,
    pub comments: Vec<fastn_section::Span>,
    pub line_starts: Vec<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Definition {
    pub doc: Option<fastn_section::Span>,
    /// resolving an identifier means making sure it is unique in the document, and performing
    /// other checks.
    pub name: UR<Identifier, Identifier>,
    pub visibility: fastn_section::Visibility,
    pub inner: InnerDefinition,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InnerDefinition {
    Component {
        properties: Vec<UR<Argument, fastn_type::Argument>>,
        body: Vec<UR<ComponentInvocation, fastn_type::ComponentInvocation>>,
    },
    Variable {
        kind: UR<Kind, fastn_type::Kind>,
        properties: Vec<UR<Property, fastn_type::Property>>,
        /// resolved caption goes to properties
        caption: UR<Vec<fastn_section::Tes>, ()>,
        /// resolved body goes to properties
        body: UR<Vec<fastn_section::Tes>, ()>,
    },
    Function {
        arguments: Vec<UR<Argument, fastn_type::Argument>>,
        return_type: Option<UR<Kind, fastn_type::Kind>>,
        /// this one is a little interesting, the number of expressions can be more than the number
        /// of Tes, this because we can have multiple expressions in a single Tes.
        body: Vec<UR<fastn_section::Tes, fastn_type::FunctionExpression>>,
    },
    TypeAlias {
        kind: UR<Kind, fastn_type::Kind>,
        /// ```ftd
        /// -- type foo: person
        /// name: foo                  ;; we are updating / setting the default value
        /// ```
        arguments: Vec<UR<Property, fastn_type::Property>>,
    },
    Record {
        properties: Vec<UR<Argument, fastn_type::Argument>>,
    },
    // TODO: OrType(fastn_section::Section),
    // TODO: Module(fastn_section::Section),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Import {
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
    /// once a caption is resolved, it is set to () here, and moved to properties
    pub caption: UR<Option<fastn_section::HeaderValue>, ()>,
    pub properties: Vec<UR<Property, fastn_type::Property>>,
    /// once the body is resolved, it is set to () here, and moved to properties
    pub body: UR<Vec<fastn_section::Tes>, ()>,
    pub children: Vec<UR<ComponentInvocation, fastn_type::ComponentInvocation>>,
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

#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PackageName(pub String);

#[derive(Debug, Clone, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ModuleName {
    pub name: Identifier,
    pub package: PackageName,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
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

#[derive(Debug, Clone, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SymbolName {
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
