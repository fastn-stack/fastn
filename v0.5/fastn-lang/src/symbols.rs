pub enum LookupResult<'input> {
    /// the unresolved symbol and the file source it was resolved from
    ///
    /// the resolved and unresolved symbols contain spans so we need source to translate them to
    /// names
    Unresolved(&'input fastn_unresolved::Definition, &'input str),
    /// the resolved symbol and the file source it was resolved from
    Resolved(&'input fastn_type::Definition),
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
    LastResolutionFailed(Vec<fastn_section::Error>),
}

#[async_trait::async_trait]
pub trait SymbolStore<'input> {
    async fn lookup(&'input mut self, qualified_identifier: &str) -> LookupResult<'input>;
}
