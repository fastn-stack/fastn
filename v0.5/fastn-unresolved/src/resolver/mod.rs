mod component_invocation;
mod definition;

pub struct Output {
    pub stuck_on: std::collections::HashSet<fastn_unresolved::SymbolName>,
    pub errors: Vec<fastn_section::Spanned<fastn_section::Error>>,
    pub warnings: Vec<fastn_section::Spanned<fastn_section::Warning>>,
    pub comments: Vec<fastn_section::Span>,
}

pub struct Input<'a> {
    pub bag: &'a std::collections::HashMap<
        string_interner::DefaultSymbol,
        fastn_unresolved::LookupResult,
    >,
    pub auto_imports: &'a [fastn_section::AutoImport],
    // TODO: use interned string instead of String below
    pub builtins: &'a indexmap::IndexMap<String, fastn_resolved::Definition>,
}
