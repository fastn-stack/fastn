#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_lang;

mod ast;
mod compiler;
mod parse;

pub use fastn_section::Result;

pub trait DS {
    async fn source(&mut self, document: &str) -> Result<String>;
    async fn parse(&mut self, qualified_identifier: &str) -> Result<fastn_lang::parse::Definition>;
    async fn ast(&mut self, qualified_identifier: &str) -> Result<fastn_lang::ast::Definition>;
    async fn add_ast(
        &mut self,
        qualified_identifier: &str,
        ast: fastn_lang::ast::Definition,
    ) -> Result<()>;
    async fn parse_tree(&mut self, document: &str) -> Result<Vec<fastn_lang::parse::Definition>>;
    async fn ast_tree(&mut self, document: &str) -> Result<Vec<fastn_lang::parse::Definition>>;
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
