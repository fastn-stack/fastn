#[derive(Debug, Default)]
pub struct Symbols {
    failed: std::collections::HashMap<fastn_unresolved::SymbolName, Vec<fastn_section::Error>>,
    unresolved: std::collections::HashMap<
        fastn_unresolved::ModuleName,
        (
            String, // source
            std::collections::HashMap<fastn_unresolved::Identifier, fastn_unresolved::Definition>,
        ),
    >,
}

impl<'input> fastn_compiler::SymbolStore<'input> for Symbols {
    fn lookup(
        &'input mut self,
        symbol: &fastn_unresolved::SymbolName,
    ) -> fastn_compiler::LookupResult<'input> {
        // using if let Some(v) is shorter, but borrow checker doesn't like it
        if self.failed.contains_key(symbol) {
            return fastn_compiler::LookupResult::LastResolutionFailed(
                self.failed.get(symbol).unwrap(),
            );
        }
        if self.unresolved.contains_key(&symbol.module) {
            let (source, symbols) = self.unresolved.get(&symbol.module).unwrap();
            // since we read all unresolved symbols defined in a module in one go, we can just check
            // if the symbol is present in the hashmap
            return match symbols.get(&symbol.name) {
                Some(v) => fastn_compiler::LookupResult::Unresolved(v, source),
                None => {
                    self.failed.insert(symbol.clone(), vec![]);
                    fastn_compiler::LookupResult::NotFound
                }
            };
        }

        // we need to fetch the symbol from the store
        let source = match std::fs::File::open(format!("{}.ftd", symbol.module.name.0))
            .and_then(std::io::read_to_string)
        {
            Ok(v) => v,
            Err(_e) => {
                self.failed
                    .insert(symbol.clone(), vec![fastn_section::Error::SymbolNotFound]);
                return fastn_compiler::LookupResult::NotFound;
            }
        };

        let d = fastn_unresolved::parse(&symbol.module, &source);
        let definitions = d
            .definitions
            .into_iter()
            .map(|d| match d {
                fastn_unresolved::UR::UnResolved(v) => (v.name().into(), v),
                fastn_unresolved::UR::Resolved(_) => {
                    unreachable!()
                }
            })
            .collect();

        self.unresolved
            .insert(symbol.module.clone(), (source, definitions));

        self.unresolved
            .get(&symbol.module)
            .unwrap()
            .1
            .get(&symbol.name)
            .map_or(
                {
                    self.failed
                        .insert(symbol.clone(), vec![fastn_section::Error::SymbolNotFound]);
                    fastn_compiler::LookupResult::NotFound
                },
                |v| {
                    fastn_compiler::LookupResult::Unresolved(
                        v,
                        &self.unresolved.get(&symbol.module).unwrap().0,
                    )
                },
            )
    }
}
