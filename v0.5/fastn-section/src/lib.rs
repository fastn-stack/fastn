#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_section;

mod debug;
mod error;
mod parser;
mod scanner;
mod utils;
mod warning;
mod wiggin;

pub use error::Error;
pub use fastn_section::warning::Warning;
pub use scanner::{ECey, Scanner};

/// TODO: span has to keep track of the document as well now.
/// TODO: demote usize to u32.
///
/// the document would be document id as stored in sqlite documents table.
///
/// Note: instead of Range, we will use a custom struct, we can use a single 32bit data to store
/// both start, and length. or we keep our life simple, we have can have sections that are really
/// long, eg a long ftd file. lets assume this is the decision for v0.5. we can demote usize to u32
/// as we do not expect individual documents to be larger than few GBs.
#[derive(PartialEq, Hash, Debug, Eq, Clone, Default)]
pub struct Span {
    // TODO: store file name here
    inner: arcstr::Substr, // this is currently a 32-byte struct.
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

pub trait JDebug: std::fmt::Debug {
    fn debug(&self) -> serde_json::Value;
}

#[derive(Debug)]
pub enum Diagnostic {
    Error(Error),
    Warning(Warning),
}

pub type Result<T> = std::result::Result<T, fastn_section::Error>;

#[derive(Debug, Clone, Default)]
pub struct Document {
    pub module_doc: Option<fastn_section::Span>,
    pub sections: Vec<Section>,
    pub errors: Vec<fastn_section::Spanned<fastn_section::Error>>,
    pub warnings: Vec<fastn_section::Spanned<fastn_section::Warning>>,
    pub comments: Vec<fastn_section::Span>,
    pub line_starts: Vec<u32>,
}

/// identifier is variable or component etc name
///
/// identifier starts with Unicode alphabet and can contain any alphanumeric Unicode character
/// dash (`-`) and underscore (`_`) are also allowed
///
/// TODO: identifiers can't be keywords of the language, e.g., `import`, `record`, `component`.
/// but it can be built in types e.g., `integer` etc.
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Identifier {
    pub name: fastn_section::Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IdentifierReference {
    // foo
    Local(fastn_section::Span), // -- foo:
    // bar.foo: module = bar, name: foo
    Imported {
        // -- foo.bar: (foo/bar#bar)
        module: fastn_section::Span,
        name: fastn_section::Span,
    },
    // bar#foo: component using the absolute path.
    Absolute {
        // -- foo#bar:
        package: fastn_section::Span,
        module: Option<fastn_section::Span>,
        name: fastn_section::Span,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Section {
    pub init: fastn_section::SectionInit,
    pub caption: Option<fastn_section::HeaderValue>,
    pub headers: Vec<Header>,
    pub body: Option<fastn_section::HeaderValue>,
    pub children: Vec<Section>,
    pub is_commented: bool,
    // if the user used `-- end: <section-name>` to end the section
    pub has_end: bool,
}

/// example: `-- list<string> foo:`
#[derive(Debug, PartialEq, Clone)]
pub struct SectionInit {
    pub dashdash: fastn_section::Span, // for syntax highlighting and formatting
    pub name: fastn_section::IdentifierReference,
    pub kind: Option<fastn_section::Kind>,
    pub doc: Option<fastn_section::Span>,
    pub visibility: Option<fastn_section::Spanned<fastn_section::Visibility>>,
    pub colon: fastn_section::Span, // for syntax highlighting and formatting
    pub function_marker: Option<fastn_section::Span>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Header {
    pub name: fastn_section::Identifier,
    pub kind: Option<fastn_section::Kind>,
    pub doc: Option<fastn_section::Span>,
    pub visibility: Option<fastn_section::Spanned<fastn_section::Visibility>>,
    pub condition: Option<fastn_section::Span>,
    pub value: fastn_section::HeaderValue,
    pub is_commented: bool,
}

// Note: doc and visibility technically do not belong to Kind, but we are keeping them here
// because otherwise we will have to put them on KindedName.
// KindedName is used a lot more often (in headers, sections, etc.) than Kind, so it makes sense
// to KindedName smaller and Kind bigger.
/// example: `list<string>` | `foo<a, b>` | `foo<bar<k>>` | `foo<a, b<asd>, c, d>` |
/// `foo<a, b, c, d, e>`
///
/// ```ftd
/// -- list<
///     ;; foo
///     integer
/// > string:
/// ```
///
/// // |foo<>|
///
/// note that this function is not responsible for parsing the visibility or doc-comments,
/// it only parses the name and args
#[derive(Debug, PartialEq, Clone)]
pub struct Kind {
    pub name: IdentifierReference,
    // during parsing, we can encounter `foo<>`, which needs to be differentiated from `foo`
    // therefore we are using `Option<Vec<>>` here
    pub args: Option<Vec<Kind>>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct HeaderValue(pub Vec<Tes>);

/// example: `hello` | `hello ${world}` | `hello ${world} ${ -- foo: }` | `{ \n text text \n }`
/// it can even have recursive structure, e.g., `hello ${ { \n text-text \n } }`.
/// each recursion starts with `{` and ends with `}`.
/// if the text inside { starts with `--` then the content is a section,
/// and we should use `fastn_section::parser::section()` parser to unresolved it.
/// otherwise it is a text.
#[derive(Debug, PartialEq, Clone)]
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
#[derive(Debug, PartialEq, Clone, Default)]
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
