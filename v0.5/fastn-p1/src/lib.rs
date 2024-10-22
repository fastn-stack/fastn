#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]

extern crate self as fastn_p1;

pub mod parse_v1;
pub mod parse_v2;
mod parse_v3;
#[cfg(test)]
mod test;
mod tokens;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(grammar);

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Section<'a> {
    pub name: KindedName<'a>,
    pub caption: Option<HeaderValue<'a>>,
    pub headers: Vec<Header<'a>>,
    pub body: Option<HeaderValue<'a>>,
    pub sub_sections: Vec<Sourced<Section<'a>>>,
    pub is_function: bool,
    pub is_commented: bool,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Header<'a> {
    pub name: KindedName<'a>,
    pub condition: Option<Sourced<&'a str>>,
    pub value: HeaderValue<'a>,
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
pub struct Kind<'a> {
    // only kinded section / header can have doc
    pub doc: Option<Sourced<&'a str>>,
    pub visibility: Visibility,
    pub kind: Sourced<&'a str>,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct KindedName<'a> {
    pub kind: Option<Kind<'a>>,
    pub name: Sourced<&'a str>,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Sourced<T> {
    /// position of this symbol from the beginning of the source file
    pub from: usize,
    /// end of this symbol from the beginning of source file
    pub to: usize,
    pub value: T,
}

pub type HeaderValue<'a> = Sourced<Vec<StringOrSection<'a>>>;

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum StringOrSection<'a> {
    // This is a `Cow<_>` because we will be escaping \{ and \} in the string, and also trimming
    // de-indenting the string, further string is cow because we remove comments, further we may
    // de-indent the string
    String(Sourced<std::borrow::Cow<'a, &'a str>>),
    // from expression as well we will remove all the comments, so it has to be a cow
    Expression(Sourced<std::borrow::Cow<'a, &'a str>>),
    Section(Box<Sourced<Section<'a>>>),
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
pub struct ParseOutput<'a> {
    pub doc_name: &'a str,
    pub module_doc: Option<fastn_p1::Sourced<std::borrow::Cow<'a, &'a str>>>,
    pub items: Vec<fastn_p1::Sourced<fastn_p1::Item<'a>>>,
    pub line_starts: Vec<usize>,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum Item<'a> {
    Section(fastn_p1::Section<'a>),
    Error(fastn_p1::Sourced<fastn_p1::SingleError<'a>>),
    Comment(&'a str),
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum SingleError<'a> {
    /// we found some text when we were not expecting, eg at the beginning of the file before
    /// any section started, or inside a section that does not expect any text. this second part
    /// I am not sure right now as we are planning ot convert all text to text nodes inside a
    /// section. so by the end maybe this will only contain the first part.
    UnwantedTextFound(fastn_p1::Sourced<&'a [char]>),
    // SectionNotFound(&'a str),
    // MoreThanOneCaption,
    // ParseError,
    // MoreThanOneHeader,
    // HeaderNotFound,
}

#[test]
fn grammar_test() {
    let input = "-- foo:";
    let lexer = fastn_p1::parse_v3::Lexer::new(input);
    let parser = grammar::SectionParser::new();
    let ast = parser.parse(input, lexer).unwrap();
    dbg!(ast);
    panic!();
}
