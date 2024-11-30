#[derive(Debug)]
pub struct Symbols {}

impl Symbols {
    fn find_all_definitions_in_a_module(
        &mut self,
        arena: &mut fastn_unresolved::Arena,
        (file, module): (String, fastn_unresolved::Module),
    ) -> Vec<fastn_unresolved::URD> {
        // we need to fetch the symbol from the store
        let source = match std::fs::File::open(file.as_str()).and_then(std::io::read_to_string) {
            Ok(v) => v,
            Err(e) => {
                println!("failed to read file {}.ftd: {e:?}", file);
                return vec![];
            }
        };

        let d = fastn_unresolved::parse(module.clone(), &source, arena);

        d.definitions
            .into_iter()
            .map(|d| match d {
                fastn_unresolved::UR::UnResolved(mut v) => {
                    v.symbol = Some(module.symbol(v.name.unresolved().unwrap().str(), arena));
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
        arena: &mut fastn_unresolved::Arena,
        symbols: &std::collections::HashSet<fastn_unresolved::Symbol>,
        // auto_import_scope: fastn_unresolved::SFId,
    ) -> Vec<fastn_unresolved::URD> {
        let unique_modules = symbols
            .iter()
            .map(|s| file_for_symbol(s, arena))
            .collect::<std::collections::HashSet<_>>();

        unique_modules
            .into_iter()
            .flat_map(|m| self.find_all_definitions_in_a_module(arena, m))
            .collect()
    }
}

fn file_for_symbol(
    symbol: &fastn_unresolved::Symbol,
    arena: &mut fastn_unresolved::Arena,
) -> (String, fastn_unresolved::Module) {
    (
        format!("{}/{}.ftd", symbol.package(arena), symbol.module(arena)),
        symbol.parent(arena),
    )
}
