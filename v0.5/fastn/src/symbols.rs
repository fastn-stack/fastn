#[derive(Debug, Default)]
pub struct Symbols {
    #[expect(unused)]
    resolved: std::collections::HashMap<String, fastn_type::Definition>,
    #[expect(unused)]
    failed: std::collections::HashMap<String, Vec<fastn_section::Error>>,
    #[expect(unused)]
    unresolved: std::collections::HashMap<
        fastn_unresolved::PackageName,
        std::collections::HashMap<fastn_unresolved::Identifier, fastn_unresolved::Definition>,
    >,
}

#[async_trait::async_trait]
impl<'input> fastn_lang::SymbolStore<'input> for Symbols {
    async fn lookup(
        &'input mut self,
        _qualified_identifier: &str,
    ) -> fastn_lang::LookupResult<'input> {
        todo!()
    }
}
