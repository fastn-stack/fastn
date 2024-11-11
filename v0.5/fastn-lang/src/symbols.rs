pub enum LookupResult {
    Unresolved(fastn_unresolved::Definition),
    Resolved(fastn_unresolved::Definition),
    NotFound,
    /// if the resolution failed, we need not try to resolve it again, unless dependencies change.
    ///
    /// say when we are processing x.ftd we found out that symbol foo is invalid, so when we are
    /// processing y.ftd, and we find foo, we can directly say that it is invalid.
    ///
    /// this is the goal, but we do not know why is foo not valid, meaning on what other symbol does
    /// it depend on, so when do we "revalidate" the symbol?
    ///
    /// what if we store the dependencies it failed on, so when any of them changes, we can
    /// revalidate?
    LastResolutionFailed(Vec<fastn_section::Error>),
}

#[async_trait::async_trait]
pub trait SymbolStore {
    async fn lookup(&mut self, qualified_identifier: &str) -> LookupResult;
}
