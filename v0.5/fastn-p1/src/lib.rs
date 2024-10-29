#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]

extern crate self as fastn_p1;

mod debug;
mod parser;
mod section;
mod utils;
mod wiggins;

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
#[serde(default)]
pub struct Section<'input> {
    pub init: SectionInit<'input>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub caption: Option<HeaderValue<'input>>,
    pub headers: Vec<Header<'input>>,
    pub body: Option<HeaderValue<'input>>,
    pub sub_sections: Vec<Spanned<'input, Section<'input>>>,
    pub function_marker: Option<Span<'input>>,
    pub is_commented: bool,
}

/// example: `-- list<string> foo:`
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct SectionInit<'input> {
    pub dashdash: Span<'input>, // for syntax highlighting and formatting
    pub name: KindedName<'input>,
    pub colon: Span<'input>, // for syntax highlighting and formatting
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Span<'input> {
    pub start: usize,
    pub end: usize,
    pub text: &'input str,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Spanned<'input, T> {
    pub span: Span<'input>,
    pub value: T,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Header<'input> {
    pub name: KindedName<'input>,
    pub condition: Option<Span<'input>>,
    pub value: HeaderValue<'input>,
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
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
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
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Identifier<'input> {
    name: fastn_p1::Span<'input>,
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
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct PackageName<'input> {
    name: fastn_p1::Span<'input>,
}

/// module name looks like <package-name>(/<identifier>)*/?)
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct ModuleName<'input> {
    pub package: PackageName<'input>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<Identifier<'input>>,
}

/// module name looks like <module-name>#<identifier>
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct QualifiedIdentifier<'input> {
    // the part comes before `#`
    module: Option<ModuleName<'input>>,
    // the part comes after `#`
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    terms: Vec<Identifier<'input>>,
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
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Kind<'input> {
    // only kinded section / header can have doc
    pub doc: Option<Span<'input>>,
    pub visibility: Option<Spanned<'input, Visibility>>,
    pub name: QualifiedIdentifier<'input>,
    // during parsing, we can encounter `foo<>`, which needs to be differentiated from `foo`
    // therefore we are using `Option<Vec<>>` here
    pub args: Option<Vec<Kind<'input>>>,
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
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct KindedName<'input> {
    pub kind: Option<Kind<'input>>,
    pub name: Identifier<'input>,
}

pub type HeaderValue<'input> = Vec<SES<'input>>;

/// example: `hello` | `hello ${world}` | `hello ${world} ${ -- foo: }` | `{ \n text text \n }`
/// it can even have recursive structure, e.g., `hello ${ { \n text-text \n } }`.
/// each recursion starts with `{` and ends with `}`.
/// if the text inside { starts with `--` then the content is a section,
/// and we should use `fastn_p1::parser::section()` parser to parse it.
/// otherwise it is a text.
#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum SES<'input> {
    String(Span<'input>),
    /// the start and end are the positions of `{` and `}` respectively
    Expression {
        start: usize,
        end: usize,
        content: Vec<SES<'input>>,
    },
    Section(Box<Section<'input>>),
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

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct ParseOutput<'input> {
    pub module_doc: Option<fastn_p1::Span<'input>>,
    pub items: Vec<fastn_p1::Spanned<'input, fastn_p1::Item<'input>>>,
    pub line_starts: Vec<usize>,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum Item<'input> {
    Section(Box<fastn_p1::Section<'input>>),
    Error(fastn_p1::SingleError),
    Comment,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
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
    // SectionNotFound(&'a str),
    // MoreThanOneCaption,
    // ParseError,
    // MoreThanOneHeader,
    // HeaderNotFound,
}
