#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_unresolved;

mod parser;
mod utils;

pub use parser::parse;

/// Document with imports is our first parser pass.
///
/// We parse a `Vec<fastn_section::Section>` into `DocumentWithImports`. In this `definitions` and
/// `content` may refer to things like `foo.bar` where `foo` is an imported module.
///
/// We keep the names as `foo.bar` etc. We then have a `resolve_imports` phase, where we get a bunch
/// of extra imports (these are the package level `-- fastn.auto-import` imports). We then resolve
/// the imports, and get a `DocumentWithOutImports`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DocumentWithImports {
    pub module_doc: Option<fastn_section::Span>,
    pub imports: Vec<fastn_unresolved::Import>,
    pub definitions: std::collections::HashMap<fastn_section::Identifier, Definition>,
    pub content: Vec<fastn_section::Section>,
    pub errors: Vec<fastn_section::Spanned<fastn_section::Error>>,
    pub warnings: Vec<fastn_section::Spanned<fastn_section::Warning>>,
    pub comments: Vec<fastn_section::Span>,
    pub line_starts: Vec<u32>,
}

/// DocumentWithOutImports has names like `foo`, and `foo.bar` resolved into `full-path/of/foo#bar`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DocumentWithOutImports {
    pub module_doc: Option<fastn_section::Span>,
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
    Variable(Variable),
    Function(fastn_section::Section),
    TypeAlias(fastn_section::Section),
    Record(fastn_section::Section),
    OrType(fastn_section::Section),
    Module(fastn_section::Section),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Import {
    pub package: PackageName,
    pub module: ModuleName,
    pub alias: Option<Identifier>,
    pub exports: Option<Export>,
    pub exposing: Option<Export>,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SymbolName {
    pub package: PackageName,
    pub module: ModuleName,
    /// can name contain dots? after we have `-- module foo:` feature it will, but now?
    pub name: Identifier, // name comes after #
}

// -- integer x: 10
// -- string x: hi, $y
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Variable {
    pub name: Identifier,
    pub kind: Kind,
    pub value: Vec<fastn_section::Tes>,
}

/// We cannot have kinds of like Record(SymbolName), OrType(SymbolName), because they are not
/// yet "resolved", eg `-- foo x:`, we do not know if `foo` is a record or an or-type.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
