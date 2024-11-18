pub enum LookupResult {
    /// the unresolved symbol and the file source it was resolved from.
    ///
    /// the resolved and unresolved symbols contain spans, so we need the source to translate them
    /// to names.
    Unresolved(Symbol, fastn_unresolved::Definition),
    /// the resolved symbol and the file source it was resolved from.
    Resolved(Symbol, fastn_type::Definition),
    NotFound,
    /// if the resolution failed, we need not try to resolve it again, unless dependencies change.
    ///
    /// say when we are processing x.ftd we found out that the symbol foo is invalid, so when we are
    /// processing y.ftd, and we find foo, we can directly say that it is invalid.
    ///
    /// this is the goal, but we do not know why isn't `foo` valid, meaning on what another symbol
    /// does it depend on, so when do we "revalidate" the symbol?
    ///
    /// what if we store the dependencies it failed on, so when any of them changes, we can
    /// revalidate?
    LastResolutionFailed(Vec<fastn_section::Error>),
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Symbol {
    pub package: string_interner::DefaultSymbol,
    pub module: string_interner::DefaultSymbol,
    pub identity: string_interner::DefaultSymbol,
    pub source: string_interner::DefaultSymbol,
}

pub trait SymbolStore {
    /// it is okay / acceptable to return more symbols than asked.
    ///
    /// this is because if we are fetching symbols by parsing a ftd file, it makes sense to store
    /// all the symbols found in that file in one go.
    /// instead of parsing the file multiple times, or storing the symbols on the type implementing
    /// this trait.
    ///
    /// or maybe the system can predict that if you asked for one symbol, you are going to ask
    /// for some related symbols soon.
    fn lookup(
        &mut self,
        interner: &mut string_interner::DefaultStringInterner,
        symbols: &[fastn_unresolved::SymbolName],
    ) -> Vec<LookupResult>;
}
