#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]

extern crate self as fastn_p1;

// mod lexer;
// pub mod parse_v1;
mod debug;
pub mod parse_v2;
// mod parser_v3;
mod parser_v4;
mod section;
// mod token;
mod utils;
// use lalrpop_util::lalrpop_mod;

// pub use token::Token;

// lalrpop_mod!(grammar);

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Section {
    pub dashdash: Span, // for syntax highlighting and formatting
    pub name: KindedName,
    pub colon: Span, // for syntax highlighting and formatting
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub caption: Option<HeaderValue>,
    pub headers: Vec<Header>,
    pub body: Option<HeaderValue>,
    pub sub_sections: Vec<Spanned<Section>>,
    pub function_marker: Option<Span>,
    pub is_commented: bool,
}

pub type Span = std::ops::Range<usize>;

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Header {
    pub name: KindedName,
    pub condition: Option<Span>,
    pub value: HeaderValue,
    pub is_commented: bool,
}

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

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Identifier {
    name: fastn_p1::Span,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct PackageName {
    name: fastn_p1::Span,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct ModuleName {
    pub package: PackageName,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<Identifier>,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct QualifiedIdentifier {
    // the part comes before `#`
    module: Option<ModuleName>,
    // the part comes after `#`
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    terms: Vec<Identifier>,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Kind {
    // only kinded section / header can have doc
    pub doc: Option<Span>,
    pub visibility: Spanned<Visibility>,
    pub name: QualifiedIdentifier,
    pub angle_text: Option<AngleText>,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct AngleText {
    pub start: usize,                    // position of <
    pub identifier: QualifiedIdentifier, // the actual text
    pub end: usize,                      // position of >
    pub inner: Option<Box<AngleText>>,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct KindedName {
    pub kind: Option<Kind>,
    pub name: Span,
}

pub type HeaderValue = Spanned<Vec<StringOrSection>>;

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum StringOrSection {
    String(Span),
    Expression(Span),
    Section(Box<Spanned<Section>>),
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
pub struct ParseOutput {
    pub module_doc: Option<fastn_p1::Span>,
    pub items: Vec<fastn_p1::Spanned<fastn_p1::Item>>,
    pub line_starts: Vec<usize>,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum Item {
    Section(Box<fastn_p1::Section>),
    Error(fastn_p1::SingleError),
    Comment,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum SingleError {
    /// doc comments should either come at the beginning of the file as a contiguous chunk
    /// or right before a section or a header.
    UnexpectedDocComment,
    /// we found some text when we were not expecting, eg at the beginning of the file before
    /// any section started, or inside a section that does not expect any text. this second part
    /// I am not sure right now as we are planning ot convert all text to text nodes inside a
    /// section. so by the end maybe this will only contain the first part.
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

#[cfg(test)]
mod test {
    // #[test]
    // fn grammar_test() {
    //     let input = "-- foo bar():";
    //     let lexer = fastn_p1::lexer::Lexer::new(input);
    //     let parser = fastn_p1::grammar::SectionParser::new();
    //     let ast = parser.parse(input, lexer).unwrap();
    //     dbg!(ast);
    // }

    // #[test]
    // fn test_parse_output() {
    //     use fastn_p1::debug::JDebug;
    //
    //     insta::glob!("..", "t/*.ftd", |path| {
    //         let s = {
    //             let mut s = std::fs::read_to_string(path).unwrap();
    //             s.push('\n');
    //             s
    //         };
    //         insta::assert_yaml_snapshot!(fastn_p1::ParseOutput::parse_v3(&s).debug(&s));
    //     })
    // }
}
