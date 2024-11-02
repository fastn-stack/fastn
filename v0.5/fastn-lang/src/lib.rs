#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_lang;

mod ast;
mod compiler;
#[cfg(test)]
mod debug;
mod error;
mod parse;
mod scanner;
mod token;
mod warning;

pub use error::Error;
pub use scanner::{Scannable, Scanner};
pub use warning::Warning;
// fastn_lang::Section is used in more than one place, so it is at the top level.
pub use token::Section;

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

pub type Span = std::ops::Range<usize>;
#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

#[derive(Default, Debug)]
pub struct Fuel {
    #[allow(dead_code)]
    remaining: std::rc::Rc<std::cell::RefCell<usize>>,
}
