//! resolver is the module that contains the logic for resolving unresolved components into
//! resolved components.
//!
//! why is it in the `fastn-unresolved` crate?
//!
//! so that we can add methods on `fastn_unresolved::ComponentInvocations` etc.

mod component_invocation;
mod definition;

pub struct Input<'a> {
    pub definitions: &'a std::collections::HashMap<fastn_unresolved::Symbol, fastn_unresolved::URD>,
    pub builtins: &'a indexmap::IndexMap<String, fastn_resolved::Definition>,
    pub interner: &'a string_interner::DefaultStringInterner,
}

pub struct Output {
    pub stuck_on: std::collections::HashSet<fastn_unresolved::Symbol>,
    pub errors: Vec<fastn_section::Spanned<fastn_section::Error>>,
    pub warnings: Vec<fastn_section::Spanned<fastn_section::Warning>>,
    pub comments: Vec<fastn_section::Span>,
}
