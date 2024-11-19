impl fastn_unresolved::ComponentInvocation {
    pub fn resolve(
        &mut self,
        _bag: &std::collections::HashMap<
            string_interner::DefaultSymbol,
            fastn_unresolved::LookupResult,
        >,
    ) -> Vec<fastn_unresolved::SymbolName> {
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
    ) -> Vec<fastn_unresolved::SymbolName> {
        todo!()
    }
}
