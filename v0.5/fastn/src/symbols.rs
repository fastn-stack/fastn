#[derive(Debug)]
pub struct Symbols {}

impl Symbols {
    fn find_all_definitions_in_a_module(
        &mut self,
        arena: &mut fastn_unresolved::Arena,
        global_arena: &fastn_unresolved::Arena,
        (file, module): (String, fastn_unresolved::Module),
        auto_imports: fastn_unresolved::AliasesID,
    ) -> Vec<fastn_unresolved::URD> {
        // we need to fetch the symbol from the store
        let source = match std::fs::File::open(file.as_str()).and_then(std::io::read_to_string) {
            Ok(v) => v,
            Err(e) => {
                println!("failed to read file {}.ftd: {e:?}", file);
                return vec![];
            }
        };

        let d = fastn_unresolved::parse(module.clone(), &source, arena, auto_imports, global_arena);

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
        global_arena: &fastn_unresolved::Arena,
        symbols: &std::collections::HashSet<fastn_unresolved::Symbol>,
        auto_imports: fastn_unresolved::AliasesID,
    ) -> Vec<fastn_unresolved::URD> {
        let unique_modules = symbols
            .iter()
            .map(|s| file_for_symbol(s, arena))
            .collect::<std::collections::HashSet<_>>();

        unique_modules
            .into_iter()
            .flat_map(|m| {
                self.find_all_definitions_in_a_module(arena, global_arena, m, auto_imports)
            })
            .collect()
    }
}

fn file_for_symbol(
    symbol: &fastn_unresolved::Symbol,
    arena: &mut fastn_unresolved::Arena,
) -> (String, fastn_unresolved::Module) {
    (
        // this code is nonsense right now
        match symbol.module(arena) {
            Some(module) => format!("{}/{}.ftd", symbol.package(arena), module),
            None => format!("{}/index.ftd", symbol.package(arena)),
        },
        symbol.parent(arena),
    )
}
