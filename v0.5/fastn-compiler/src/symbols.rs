pub struct LookupResult {
    #[expect(dead_code)]
    symbol: Symbol,
    #[expect(dead_code)]
    source: string_interner::DefaultSymbol,
    // package: string_interner::DefaultBackend,
    // module: string_interner::DefaultBackend,
}

pub enum Symbol {
    /// the unresolved symbol and the file source it was resolved from
    ///
    /// the resolved and unresolved symbols contain spans so we need source to translate them to
    /// names
    #[expect(dead_code)]
    Unresolved(fastn_unresolved::Definition),
    /// the resolved symbol and the file source it was resolved from
    #[expect(dead_code)]
    Resolved(fastn_type::Definition),
    #[expect(dead_code)]
    NotFound,
    /// if the resolution failed, we need not try to resolve it again, unless dependencies change.
    ///
    /// say when we are processing x.ftd we found out that symbol foo is invalid, so when we are
    /// processing y.ftd, and we find foo, we can directly say that it is invalid.
    ///
    /// this is the goal, but we do not know why isn't `foo` valid, meaning on what another symbol
    /// does it depend on, so when do we "revalidate" the symbol?
    ///
    /// what if we store the dependencies it failed on, so when any of them changes, we can
    /// revalidate?
    #[expect(dead_code)]
    LastResolutionFailed(Vec<fastn_section::Error>),
}

pub trait SymbolStore {
    fn lookup(
        &mut self,
        interner: &string_interner::DefaultStringInterner,
        symbols: &[fastn_unresolved::SymbolName],
    ) -> Vec<LookupResult>;
}
