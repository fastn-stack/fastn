#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_p1;

#[cfg(test)]
mod debug;
mod parser;
mod section;
mod utils;
mod wiggin;

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Section {
    pub init: SectionInit,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub caption: Option<HeaderValue>,
    pub headers: Vec<Header>,
    pub body: Option<HeaderValue>,
    pub children: Vec<Section>, // TODO: this must be `Spanned<Section>`
    pub sub_sections: Vec<Spanned<Section>>,
    pub function_marker: Option<Span>,
    pub is_commented: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Definition {
    Component(Section),
    Variable(Section),
    Function(Section),
    TypeAlias(Section),
    Record(Section),
    OrType(Section),
    Module(Section),
}

/// example: `-- list<string> foo:`
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SectionInit {
    pub dashdash: Span, // for syntax highlighting and formatting
    pub name: KindedName,
    pub colon: Span, // for syntax highlighting and formatting
}

pub type Span = std::ops::Range<usize>;

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Header {
    pub name: KindedName,
    pub condition: Option<Span>,
    pub value: HeaderValue,
    pub is_commented: bool,
}

#[derive(Default, Debug)]
pub struct Fuel {
    #[allow(dead_code)]
    remaining: std::rc::Rc<std::cell::RefCell<usize>>,
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

/// identifier is variable or component etc name
///
/// identifier starts with Unicode alphabet and can contain any alphanumeric Unicode character
/// dash (`-`) and underscore (`_`) are also allowed
///
/// TODO: identifiers can't be keywords of the language, e.g., `import`, `record`, `component`.
/// but it can be built in types e.g., `integer` etc.
#[derive(Debug, PartialEq, Clone, Hash, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct Identifier {
    name: fastn_p1::Span,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AliasableIdentifier {
    name: fastn_p1::Span,
    alias: Option<fastn_p1::Span>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Import {
    pub module: fastn_p1::ModuleName,
    pub exports: Option<Export>,
    pub exposing: Option<Exposing>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Export {
    All,
    Things(Vec<fastn_p1::AliasableIdentifier>),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Exposing {
    All,
    Things(Vec<fastn_p1::AliasableIdentifier>),
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
    name: fastn_p1::Span,
    // for foo.com, the alias is `foo` (the first part before the first dot)
    // TODO: unless it is `www`, then its the second part
    alias: fastn_p1::Span,
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
    module: Option<ModuleName>,
    // the part comes after `#`
    terms: Vec<Identifier>,
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
    pub doc: Option<Span>,
    pub visibility: Option<Spanned<Visibility>>,
    pub name: QualifiedIdentifier,
    // during parsing, we can encounter `foo<>`, which needs to be differentiated from `foo`
    // therefore we are using `Option<Vec<>>` here
    pub args: Option<Vec<Kind>>,
}

pub enum PResult<T> {
    NotFound,
    Found(T),
    Error(SingleError),
    Errors(Vec<SingleError>),
    FoundWithErrors {
        partial: T,
        errors: Vec<SingleError>,
    },
}

/// example: `list<string> foo` | `foo bar` | `bar`
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct KindedName {
    pub kind: Option<Kind>,
    pub name: Identifier,
}

pub type HeaderValue = Vec<SES>;

/// example: `hello` | `hello ${world}` | `hello ${world} ${ -- foo: }` | `{ \n text text \n }`
/// it can even have recursive structure, e.g., `hello ${ { \n text-text \n } }`.
/// each recursion starts with `{` and ends with `}`.
/// if the text inside { starts with `--` then the content is a section,
/// and we should use `fastn_p1::parser::section()` parser to parse it.
/// otherwise it is a text.
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum SES {
    String(Span),
    /// the start and end are the positions of `{` and `}` respectively
    Expression {
        start: usize,
        end: usize,
        content: HeaderValue,
    },
    Section(Vec<Section>),
}

#[derive(Default)]
pub struct ParserEngine {
    pub doc_name: String,
    pub edits: Vec<Edit>,
}

pub struct Edit {
    pub from: usize,
    pub to: usize,
    pub text: Vec<char>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub module_doc: Option<fastn_p1::Span>,
    pub imports: Vec<fastn_p1::Import>,
    pub definitions: std::collections::HashMap<fastn_p1::Identifier, Definition>,
    pub content: Vec<fastn_p1::Section>,
    pub errors: Vec<Spanned<fastn_p1::SingleError>>,
    pub comments: Vec<fastn_p1::Span>,
    pub line_starts: Vec<usize>,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum SingleError {
    /// doc comments should either come at the beginning of the file as a contiguous chunk
    /// or right before a section or a header.
    UnexpectedDocComment,
    /// we found some text when we were not expecting, e.g., at the beginning of the file before
    /// any section started, or inside a section that does not expect any text. this second part,
    /// I am not sure right now as we are planning to convert all text to text nodes inside a
    /// section. so by the end, maybe this will only contain the first part.
    UnwantedTextFound,
    /// we found something like `-- list<> foo:`, type is not specified
    EmptyAngleText,
    /// we are looking for dash-dash, but found something else
    DashDashNotFound,
    KindedNameNotFound,
    ColonNotFound,
    SectionNameNotFoundForEnd,
    EndContainsData,
    EndWithoutStart,
    // SectionNotFound(&'a str),
    // MoreThanOneCaption,
    // ParseError,
    // MoreThanOneHeader,
    // HeaderNotFound,
}
