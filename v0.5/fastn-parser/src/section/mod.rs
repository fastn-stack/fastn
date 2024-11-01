mod parser;
mod utils;
mod wiggin;

pub use crate::section::parser::identifier::identifier;
pub use crate::section::parser::kind::kind;
pub use crate::section::parser::kinded_name::kinded_name;
pub use crate::section::parser::module_name::module_name;
pub use crate::section::parser::package_name::package_name;
pub use crate::section::parser::qualified_identifier::qualified_identifier;
pub use parser::Scanner;

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Section {
    pub init: fastn_parser::SectionInit,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub caption: Option<fastn_parser::HeaderValue>,
    pub headers: Vec<Header>,
    pub body: Option<fastn_parser::HeaderValue>,
    pub children: Vec<Section>, // TODO: this must be `Spanned<Section>`
    pub sub_sections: Vec<Spanned<Section>>,
    pub function_marker: Option<Span>,
    pub is_commented: bool,
}

/// example: `-- list<string> foo:`
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SectionInit {
    pub dashdash: fastn_parser::Span, // for syntax highlighting and formatting
    pub name: fastn_parser::KindedName,
    pub colon: fastn_parser::Span, // for syntax highlighting and formatting
}

pub type Span = std::ops::Range<usize>;

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Header {
    pub name: fastn_parser::KindedName,
    pub condition: Option<fastn_parser::Span>,
    pub value: fastn_parser::HeaderValue,
    pub is_commented: bool,
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
    pub name: fastn_parser::Span,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AliasableIdentifier {
    pub name: fastn_parser::Span,
    pub alias: Option<fastn_parser::Span>,
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
    pub name: fastn_parser::Span,
    // for foo.com, the alias is `foo` (the first part before the first dot)
    // TODO: unless it is `www`, then its the second part
    pub alias: fastn_parser::Span,
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
    pub doc: Option<Span>,
    pub visibility: Option<Spanned<Visibility>>,
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

pub type HeaderValue = Vec<SES>;

/// example: `hello` | `hello ${world}` | `hello ${world} ${ -- foo: }` | `{ \n text text \n }`
/// it can even have recursive structure, e.g., `hello ${ { \n text-text \n } }`.
/// each recursion starts with `{` and ends with `}`.
/// if the text inside { starts with `--` then the content is a section,
/// and we should use `fastn_parser::parser::section()` parser to parse it.
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
