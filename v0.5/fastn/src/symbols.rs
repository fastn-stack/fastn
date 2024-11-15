#[derive(Debug, Default)]
pub struct Symbols {
    resolved: std::collections::HashMap<fastn_unresolved::SymbolName, fastn_type::Definition>,
    failed: std::collections::HashMap<fastn_unresolved::SymbolName, Vec<fastn_section::Error>>,
    unresolved: std::collections::HashMap<
        fastn_unresolved::ModuleName,
        (
            String,
            std::collections::HashMap<fastn_unresolved::Identifier, fastn_unresolved::Definition>,
        ),
    >,
}

#[async_trait::async_trait]
impl<'input> fastn_lang::SymbolStore<'input> for Symbols {
    async fn lookup(
        &'input mut self,
        symbol: &fastn_unresolved::SymbolName,
    ) -> fastn_lang::LookupResult<'input> {
        if let Some(v) = self.failed.get(symbol) {
            return fastn_lang::LookupResult::LastResolutionFailed(v);
        }
        if let Some(v) = self.resolved.get(symbol) {
            return fastn_lang::LookupResult::Resolved(v);
        }
        if let Some((source, symbols)) = self.unresolved.get(&symbol.module) {
            // since we read all unresolved symbols defined in a module in one go, we can just check
            // if the symbol is present in the hashmap
            return match symbols.get(&symbol.name) {
                Some(v) => fastn_lang::LookupResult::Unresolved(v, source),
                None => {
                    self.failed.insert(symbol.clone(), vec![]);
                    fastn_lang::LookupResult::NotFound
                }
            };
        }

        // we need to fetch the symbol from the store
        todo!()
    }
}
