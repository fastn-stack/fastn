#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_section;

#[cfg(test)]
mod debug;
mod error;
mod parser;
mod scanner;
mod utils;
mod warning;
mod wiggin;

#[cfg(test)]
pub(crate) use debug::JDebug;
pub use error::Error;
pub use fastn_section::parser::identifier::identifier;
pub use fastn_section::parser::kind::kind;
pub use fastn_section::parser::kinded_name::kinded_name;
pub use fastn_section::parser::module_name::module_name;
pub use fastn_section::parser::package_name::package_name;
pub use fastn_section::parser::qualified_identifier::qualified_identifier;
pub use fastn_section::warning::Warning;
pub use scanner::{Scannable, Scanner};

pub type Result<T> = std::result::Result<T, fastn_section::Error>;
pub type Span = std::ops::Range<usize>;
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub module_doc: Option<fastn_section::Span>,
    pub sections: Vec<Section>,
    pub errors: Vec<fastn_section::Spanned<fastn_section::Error>>,
    pub warnings: Vec<fastn_section::Spanned<fastn_section::Warning>>,
    pub comments: Vec<fastn_section::Span>,
    pub line_starts: Vec<usize>,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Section {
    pub init: fastn_section::SectionInit,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub caption: Option<fastn_section::HeaderValue>,
    pub headers: Vec<Header>,
    pub body: Option<fastn_section::HeaderValue>,
    pub children: Vec<Section>, // TODO: this must be `Spanned<Section>`
    pub sub_sections: Vec<fastn_section::Spanned<Section>>,
    pub function_marker: Option<fastn_section::Span>,
    pub is_commented: bool,
    // if the user used `-- end: <section-name>` to end the section
    pub has_end: bool,
}

/// example: `-- list<string> foo:`
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SectionInit {
    pub dashdash: fastn_section::Span, // for syntax highlighting and formatting
    pub name: fastn_section::KindedName,
    pub colon: fastn_section::Span, // for syntax highlighting and formatting
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Header {
    pub name: fastn_section::KindedName,
    pub condition: Option<fastn_section::Span>,
    pub value: fastn_section::HeaderValue,
    pub is_commented: bool,
}

/// identifier is variable or component etc name
///
/// identifier starts with Unicode alphabet and can contain any alphanumeric Unicode character
/// dash (`-`) and underscore (`_`) are also allowed
///
/// TODO: identifiers can't be keywords of the language, e.g., `import`, `record`, `component`.
/// but it can be built in types e.g., `integer` etc.
#[derive(Debug, PartialEq, Clone, Hash, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct Identifier {
    pub name: fastn_section::Span,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AliasableIdentifier {
    pub name: fastn_section::Span,
    pub alias: Option<fastn_section::Span>,
}

/// package names for fastn as domain names.
///
/// domain names usually do not allow Unicode, and you have to use punycode.
/// but we allow Unicode in package names.
///
/// TODO: domain name can contain hyphens.
/// TODO: domain name can’t begin or end with a hyphen.
/// underscore is not permitted in domain names.
///
/// `.` is allowed in domain names.
/// TODO: domain name can't begin or end with a `.`.
/// TODO: `.` can't be repeated.
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PackageName {
    pub name: fastn_section::Span,
    // for foo.com, the alias is `foo` (the first part before the first dot)
    // TODO: unless it is `www`, then its the second part
    pub alias: fastn_section::Span,
}

/// module name looks like <package-name>(/<identifier>)*/?)
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ModuleName {
    pub package: PackageName,
    pub name: AliasableIdentifier,
    pub path: Vec<Identifier>, // rest of the path
}

/// module name looks like <module-name>#<identifier>
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct QualifiedIdentifier {
    // the part comes before `#`
    pub module: Option<ModuleName>,
    // the part comes after `#`
    pub terms: Vec<Identifier>,
}

// Note: doc and visibility technically do not belong to Kind, but we are keeping them here
// because otherwise we will have to put them on KindedName.
// KindedName is used a lot more often (in headers, sections, etc.) than Kind, so it makes sense
// to KindedName smaller and Kind bigger.
/// example: `list<string>` | `foo<a, b>` | `foo<bar<k>>` | `foo<a, b<asd>, c, d>` |
/// `foo<a, b, c, d, e>`
///
/// // |foo<>|
///
/// note that this function is not responsible for parsing the visibility or doc-comments,
/// it only parses the name and args
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Kind {
    // only kinded section / header can have doc
    pub doc: Option<fastn_section::Span>,
    pub visibility: Option<fastn_section::Spanned<fastn_section::Visibility>>,
    pub name: QualifiedIdentifier,
    // during parsing, we can encounter `foo<>`, which needs to be differentiated from `foo`
    // therefore we are using `Option<Vec<>>` here
    pub args: Option<Vec<Kind>>,
}

/// example: `list<string> foo` | `foo bar` | `bar`
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct KindedName {
    pub kind: Option<Kind>,
    pub name: Identifier,
}

pub type HeaderValue = Vec<Tes>;

/// example: `hello` | `hello ${world}` | `hello ${world} ${ -- foo: }` | `{ \n text text \n }`
/// it can even have recursive structure, e.g., `hello ${ { \n text-text \n } }`.
/// each recursion starts with `{` and ends with `}`.
/// if the text inside { starts with `--` then the content is a section,
/// and we should use `fastn_section::parser::section()` parser to unresolved it.
/// otherwise it is a text.
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum Tes {
    Text(fastn_section::Span),
    /// the start and end are the positions of `{` and `}` respectively
    Expression {
        start: usize,
        end: usize,
        content: HeaderValue,
    },
    Section(Vec<Section>),
}

/// public | private | public<package> | public<module>
///
/// TODO: newline is allowed, e.g., public<\n module>
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum Visibility {
    /// visible to everyone
    #[default]
    Public,
    /// visible to current package only
    Package,
    /// visible to current module only
    Module,
    /// can only be accessed from inside the component, etc.
    Private,
}

#[derive(Default, Debug)]
pub struct Fuel {
    #[allow(dead_code)]
    remaining: std::rc::Rc<std::cell::RefCell<usize>>,
}
