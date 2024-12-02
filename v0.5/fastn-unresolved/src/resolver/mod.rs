//! resolver is the module that contains the logic for resolving unresolved components into
//! resolved components.
//!
//! why is it in the `fastn-unresolved` crate?
//!
//! so that we can add methods on `fastn_unresolved::ComponentInvocations` etc.

mod component_invocation;
mod definition;
mod symbol;

use symbol::symbol;

#[derive(Debug, Default)]
pub struct Output {
    pub stuck_on: std::collections::HashSet<fastn_unresolved::Symbol>,
    pub errors: Vec<fastn_section::Spanned<fastn_section::Error>>,
    pub warnings: Vec<fastn_section::Spanned<fastn_section::Warning>>,
    pub comments: Vec<fastn_section::Span>,
}
