#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]

extern crate self as fastn_p1;

mod parse;
pub use parse::parse;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Section<'a> {
    pub name: KindedName<'a>,
    pub caption: Option<HeaderValue<'a>>,
    pub headers: Vec<(KindedName<'a>, HeaderValue<'a>)>,
    pub body: Option<HeaderValue<'a>>,
    pub sub_sections: Vec<Sourced<Section<'a>>>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum Visibility {
    // visible to everyone
    #[default]
    Public,
    // visible to current package only
    Package,
    // visible to current module only
    Module,
    // can only be accessed from inside the component etc
    Private,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Kind<'a> {
    // only kinded section / header can have doc
    doc: Option<Sourced<&'a str>>,
    visibility: Visibility,
    kind: Sourced<&'a str>,
    // // -- void foo(x, y):, x and y are args
    // args: Option<Vec<Sourced<&'a str>>>,
    is_function: bool,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct KindedName<'a> {
    pub kind: Option<Kind<'a>>,
    pub name: Sourced<&'a str>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Sourced<T> {
    /// position of this symbol from the beginning of the source file
    pub from: usize,
    /// end of this symbol from the beginning of source file
    pub to: usize,
    pub is_commented: bool,
    pub value: T,
}

pub type HeaderValue<'a> = Sourced<Vec<StringOrSection<'a>>>;

#[derive(Debug, PartialEq, Clone)]
pub enum StringOrSection<'a> {
    // This is a `Cow<_>` because we will be escaping \{ and \} in the string, and also trimming
    // de-indenting the string, further string is cow because we remove comments, further we may
    // de-indent the string
    String(Sourced<std::borrow::Cow<'a, &'a str>>),
    // from expression as well we will remove all the comments, so it has to be a cow
    Expression(Sourced<std::borrow::Cow<'a, &'a str>>),
    Section(Sourced<Section<'a>>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Item<'a> {
    Section(Section<'a>),
    Comment(&'a str),
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ParseOutput<'a> {
    module_doc: Option<Sourced<&'a str>>,
    items: Vec<Sourced<Item<'a>>>,
    /// length of each line in the source
    line_lengths: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SingleError {
    // SectionNotFound,
    // MoreThanOneCaption,
    // ParseError,
    // MoreThanOneHeader,
    // HeaderNotFound,
}

// should we base this on https://docs.rs/ariadne/ or https://docs.rs/miette/?
#[derive(Debug, PartialEq, Clone, Default)]
pub struct ParseError<'a> {
    partial: ParseOutput<'a>,
    errors: Vec<Sourced<SingleError>>,
}
