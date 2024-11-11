#[derive(Debug, Default)]
pub struct Symbols {
    resolved: std::collections::HashMap<String, fastn_unresolved::Definition>,
    unresolved: std::collections::HashMap<String, fastn_unresolved::Definition>,
}

#[async_trait::async_trait]
impl fastn_lang::SymbolStore for Symbols {
    async fn lookup(&mut self, _qualified_identifier: &str) -> fastn_lang::LookupResult {
        todo!()
    }
}
