pub enum LookupResult {
    /// the unresolved symbol and the file source it was resolved from.
    ///
    /// the resolved and unresolved symbols contain spans, so we need the source to translate them
    /// to names.
    Unresolved(string_interner::DefaultSymbol, fastn_unresolved::Definition),
    /// the resolved symbol and the file source it was resolved from.
    Resolved(string_interner::DefaultSymbol, fastn_type::Definition),
    NotFound(string_interner::DefaultSymbol),
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
    LastResolutionFailed(string_interner::DefaultSymbol, Vec<fastn_section::Error>),
}

impl LookupResult {
    pub fn symbol(&self) -> string_interner::DefaultSymbol {
        match self {
            LookupResult::Unresolved(s, _) => *s,
            LookupResult::Resolved(s, _) => *s,
            LookupResult::NotFound(s) => *s,
            LookupResult::LastResolutionFailed(s, _) => *s,
        }
    }
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
