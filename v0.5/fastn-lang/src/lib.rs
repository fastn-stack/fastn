#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_lang;

mod compiler;
pub mod resolved;
pub mod unresolved;

pub use fastn_section::Result;

pub struct UISpec {
    pub title: String,
    pub body: String,
}

pub enum Output {
    UI(UISpec),
    Data(serde_json::Value),
}

pub trait DS {
    async fn source(&mut self, document: &str) -> Result<String>;
    async fn unresolved(
        &mut self,
        qualified_identifier: &str,
    ) -> Result<fastn_lang::unresolved::Definition>;
    async fn resolved(
        &mut self,
        qualified_identifier: &str,
    ) -> Result<fastn_lang::resolved::Definition>;
    async fn add_resolved(
        &mut self,
        qualified_identifier: &str,
        resolved: fastn_lang::resolved::Definition,
    ) -> Result<()>;
    async fn unresolved_document(
        &mut self,
        document: &str,
    ) -> Result<Vec<fastn_lang::unresolved::Document>>;
    async fn resolved_document(
        &mut self,
        document: &str,
    ) -> Result<Vec<fastn_lang::resolved::Document>>;
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
