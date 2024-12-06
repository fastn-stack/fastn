impl fastn_ds::DocumentStore {
    async fn find_all_definitions_in_a_module(
        &self,
        arena: &mut fastn_unresolved::Arena,
        global_aliases: &fastn_unresolved::AliasesSimple,
        (file, module): (String, fastn_unresolved::Module),
    ) -> Vec<fastn_unresolved::URD> {
        // we need to fetch the symbol from the store
        let source = match self
            .read_to_string(&fastn_ds::Path::new(file.as_str()), &None)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                println!("failed to read file {}.ftd: {e:?}", file);
                return vec![];
            }
        };

        let d = fastn_unresolved::parse(module.clone(), &source, arena, global_aliases);

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
impl fastn_compiler::SymbolStore for &fastn_ds::DocumentStore {
    async fn lookup(
        &self,
        arena: &mut fastn_unresolved::Arena,
        global_aliases: &fastn_unresolved::AliasesSimple,
        symbols: &std::collections::HashSet<fastn_unresolved::Symbol>,
    ) -> Vec<fastn_unresolved::URD> {
        let unique_modules = symbols
            .iter()
            .map(|s| file_for_symbol(s, arena))
            .collect::<std::collections::HashSet<_>>();

        let mut urds = vec![];

        for m in unique_modules {
            urds.extend(
                self.find_all_definitions_in_a_module(arena, global_aliases, m)
                    .await,
            )
        }

        urds
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
