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

impl<'input> fastn_lang::SymbolStore<'input> for Symbols {
    fn lookup(
        &'input mut self,
        symbol: &fastn_unresolved::SymbolName,
    ) -> fastn_lang::LookupResult<'input> {
        // using if let Some(v) is shorter, but borrow checker doesn't like it
        if self.failed.contains_key(symbol) {
            return fastn_lang::LookupResult::LastResolutionFailed(
                self.failed.get(symbol).unwrap(),
            );
        }
        if self.resolved.contains_key(symbol) {
            return fastn_lang::LookupResult::Resolved(self.resolved.get(symbol).unwrap());
        }
        if self.unresolved.contains_key(&symbol.module) {
            let (source, symbols) = self.unresolved.get(&symbol.module).unwrap();
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
