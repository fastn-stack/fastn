#[derive(Debug)]
pub struct Symbols {}

impl Symbols {
    fn find_all_definitions_in_a_module(
        &mut self,
        interner: &mut string_interner::DefaultStringInterner,
        (file, module): (String, fastn_unresolved::Module),
        desugared_auto_imports: &[fastn_unresolved::URD],
    ) -> Vec<fastn_unresolved::URD> {
        // we need to fetch the symbol from the store
        let source = match std::fs::File::open(file.as_str()).and_then(std::io::read_to_string) {
            Ok(v) => v,
            Err(e) => {
                println!("failed to read file {}.ftd: {e:?}", file);
                return vec![];
            }
        };

        let d = fastn_unresolved::parse(module.clone(), &source, desugared_auto_imports, interner);

        d.definitions
            .into_iter()
            .map(|d| match d {
                fastn_unresolved::UR::UnResolved(mut v) => {
                    v.symbol = Some(module.symbol(v.name.unresolved().unwrap().str(), interner));
                    fastn_unresolved::UR::UnResolved(v)
                }
                _ => {
                    unreachable!("fastn_unresolved::parse() only returns unresolved definitions")
                }
            })
            .collect::<Vec<_>>()
    }
}

#[async_trait::async_trait]
impl fastn_compiler::SymbolStore for Symbols {
    async fn lookup(
        &mut self,
        interner: &mut string_interner::DefaultStringInterner,
        symbols: &std::collections::HashSet<fastn_unresolved::Symbol>,
        desugared_auto_imports: &[fastn_unresolved::URD],
    ) -> Vec<fastn_unresolved::URD> {
        let unique_modules = symbols
            .iter()
            .map(|s| file_for_symbol(s, interner))
            .collect::<std::collections::HashSet<_>>();

        unique_modules
            .into_iter()
            .flat_map(|m| {
                self.find_all_definitions_in_a_module(interner, m, desugared_auto_imports)
            })
            .collect()
    }
}

fn file_for_symbol(
    symbol: &fastn_unresolved::Symbol,
    interner: &mut string_interner::DefaultStringInterner,
) -> (String, fastn_unresolved::Module) {
    (
        format!(
            "{}/{}.ftd",
            symbol.package(interner),
            symbol.module(interner)
        ),
        symbol.parent(interner),
    )
}
