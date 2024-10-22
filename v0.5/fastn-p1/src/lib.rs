#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]

extern crate self as fastn_p1;

mod lexer;
// pub mod parse_v1;
pub mod parse_v2;
mod parser_v3;
mod section;
#[cfg(test)]
mod test;
mod token;

use lalrpop_util::lalrpop_mod;

pub use token::Token;

lalrpop_mod!(grammar);

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Section {
    pub name: KindedName,
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
    /// can only be accessed from inside the component etc
    Private,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Kind {
    // only kinded section / header can have doc
    pub doc: Option<Span>,
    pub visibility: Spanned<Visibility>,
    pub kind: Span,
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

impl ParserEngine {
    pub fn new(doc_name: String) -> Self {
        Self {
            doc_name,
            edits: vec![],
        }
    }

    pub fn add_edit(&mut self, from: usize, to: usize, text: String) -> &Edit {
        self.edits.push(Edit {
            from,
            to,
            text: text.chars().collect(),
        });
        self.edits.last().unwrap()
    }
}

pub struct Edit {
    pub from: usize,
    pub to: usize,
    pub text: Vec<char>,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct ParseOutput {
    pub doc_name: String,
    pub module_doc: Option<fastn_p1::Span>,
    pub items: Vec<fastn_p1::Spanned<fastn_p1::Item>>,
    pub line_starts: Vec<usize>,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum Item {
    Section(Box<fastn_p1::Section>),
    Error(fastn_p1::Spanned<fastn_p1::SingleError>),
    Comment,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum SingleError {
    /// we found some text when we were not expecting, eg at the beginning of the file before
    /// any section started, or inside a section that does not expect any text. this second part
    /// I am not sure right now as we are planning ot convert all text to text nodes inside a
    /// section. so by the end maybe this will only contain the first part.
    UnwantedTextFound(fastn_p1::Span),
    // SectionNotFound(&'a str),
    // MoreThanOneCaption,
    // ParseError,
    // MoreThanOneHeader,
    // HeaderNotFound,
}

#[test]
fn grammar_test() {
    let input = "-- foo bar():";
    let lexer = fastn_p1::lexer::Lexer::new(input);
    let parser = fastn_p1::grammar::SectionParser::new();
    let ast = parser.parse(input, lexer).unwrap();
    dbg!(ast);
    panic!();
}
