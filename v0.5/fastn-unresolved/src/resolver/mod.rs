impl fastn_unresolved::ComponentInvocation {
    pub fn resolve(
        &mut self,
        _bag: &std::collections::HashMap<
            string_interner::DefaultSymbol,
            fastn_unresolved::LookupResult,
        >,
        _document: &mut fastn_unresolved::Document,
        _auto_imports: &[fastn_section::AutoImport],
    ) -> std::collections::HashSet<fastn_unresolved::SymbolName> {
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
        _document: &mut fastn_unresolved::Document,
        _auto_imports: &[fastn_section::AutoImport],
    ) -> std::collections::HashSet<fastn_unresolved::SymbolName> {
        todo!()
    }
}
