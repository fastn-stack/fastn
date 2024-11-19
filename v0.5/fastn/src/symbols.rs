#[derive(Debug, Default)]
pub struct Symbols {}

impl Symbols {
    fn find_all_definitions_in_a_module(
        &mut self,
        interner: &mut string_interner::DefaultStringInterner,
        module: &fastn_unresolved::ModuleName,
    ) -> Vec<fastn_compiler::LookupResult> {
        // we need to fetch the symbol from the store
        let source = match std::fs::File::open(format!("{}.ftd", module.name.str()))
            .and_then(std::io::read_to_string)
        {
            Ok(v) => v,
            Err(_e) => {
                return vec![];
            }
        };

        let d = fastn_unresolved::parse(module, &source);

        d.definitions
            .into_iter()
            .map(|d| match d {
                fastn_unresolved::UR::UnResolved(v) => fastn_compiler::LookupResult::Unresolved(
                    interner.get_or_intern(format!(
                        "{}/{}#{}",
                        module.package.str(),
                        module.name.str(),
                        &v.name.unresolved().unwrap().str()
                    )),
                    v,
                ),
                fastn_unresolved::UR::Resolved(_) => {
                    unreachable!(
                        "resolved definitions should not be present in the unresolved document"
                    )
                }
            })
            .collect()
    }
}

impl fastn_compiler::SymbolStore for Symbols {
    fn lookup(
        &mut self,
        interner: &mut string_interner::DefaultStringInterner,
        symbols: &[fastn_unresolved::SymbolName],
    ) -> Vec<fastn_compiler::LookupResult> {
        let unique_modules = symbols
            .iter()
            .map(|s| &s.module)
            .collect::<std::collections::HashSet<_>>();

        unique_modules
            .into_iter()
            .flat_map(|m| self.find_all_definitions_in_a_module(interner, m))
            .collect()
    }
}
