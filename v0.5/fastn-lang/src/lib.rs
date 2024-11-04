#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_lang;

mod compiler;
pub mod resolved;
pub mod unresolved;

pub use compiler::compile::compile;
pub use fastn_section::Result;

pub struct UISpec {
    pub title: String,
    pub body: String,
}

pub enum Output {
    UI(UISpec),
    Data(serde_json::Value),
}

#[async_trait::async_trait]
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
