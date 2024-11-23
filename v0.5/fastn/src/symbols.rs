#[derive(Debug, Default)]
pub struct Symbols {
    auto_imports: Vec<fastn_section::AutoImport>,
}

impl Symbols {
    fn find_all_definitions_in_a_module(
        &mut self,
        interner: &mut string_interner::DefaultStringInterner,
        (file, symbol): (String, fastn_unresolved::Symbol),
    ) -> Vec<fastn_unresolved::LookupResult> {
        // we need to fetch the symbol from the store
        let source = match std::fs::File::open(file.as_str()).and_then(std::io::read_to_string) {
            Ok(v) => v,
            Err(e) => {
                println!("failed to read file {}.ftd: {e:?}", file);
                return vec![];
            }
        };

        let d = fastn_unresolved::parse(&source);
        let mut definitions = d
            .desugar_imports(&self.auto_imports)
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<_>>();

        definitions.extend_from_slice(
            &d.definitions
                .into_iter()
                .map(|d| match d {
                    fastn_unresolved::UR::UnResolved(mut v) => {
                        v.symbol =
                            Some(symbol.with_name(v.name.unresolved().unwrap().str(), interner));
                        fastn_unresolved::UR::UnResolved(v)
                    }
                    _ => {
                        unreachable!(
                            "fastn_unresolved::parse() only returns unresolved definitions"
                        )
                    }
                })
                .collect::<Vec<_>>(),
        );

        definitions
    }
}

impl fastn_compiler::SymbolStore for Symbols {
    fn lookup(
        &mut self,
        interner: &mut string_interner::DefaultStringInterner,
        symbols: &std::collections::HashSet<fastn_unresolved::Symbol>,
    ) -> Vec<fastn_unresolved::LookupResult> {
        let unique_modules = symbols
            .iter()
            .map(|s| file_for_symbol(s, interner))
            .collect::<std::collections::HashSet<_>>();

        unique_modules
            .into_iter()
            .flat_map(|m| self.find_all_definitions_in_a_module(interner, m))
            .collect()
    }
}

fn file_for_symbol(
    symbol: &fastn_unresolved::Symbol,
    interner: &mut string_interner::DefaultStringInterner,
) -> (String, fastn_unresolved::Symbol) {
    (
        format!(
            "{}/{}.ftd",
            symbol.package(interner),
            symbol.module(interner)
        ),
        symbol.erase_name(interner),
    )
}
