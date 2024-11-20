pub struct ResolutionOutput {
    pub stuck_on: std::collections::HashSet<fastn_unresolved::SymbolName>,
    pub errors: Vec<fastn_section::Spanned<fastn_section::Error>>,
    pub warnings: Vec<fastn_section::Spanned<fastn_section::Warning>>,
    pub comments: Vec<fastn_section::Span>,
}

impl fastn_unresolved::ComponentInvocation {
    pub fn resolve(
        &mut self,
        _bag: &std::collections::HashMap<
            string_interner::DefaultSymbol,
            fastn_unresolved::LookupResult,
        >,
        _auto_imports: &[fastn_section::AutoImport],
    ) -> ResolutionOutput {
        todo!()
    }
}

impl fastn_unresolved::Definition {
    pub fn resolve(
        &mut self,
        _bag: &std::collections::HashMap<
            string_interner::DefaultSymbol,
            fastn_unresolved::LookupResult,
        >,
        _auto_imports: &[fastn_section::AutoImport],
    ) -> ResolutionOutput {
        todo!()
    }
}
