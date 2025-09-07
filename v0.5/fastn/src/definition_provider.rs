fn find_all_definitions_in_a_module(
    compiler: &mut fastn_compiler::Compiler,
    (file, module): (String, fastn_section::Module),
) -> Vec<fastn_unresolved::Urd> {
    // we need to fetch the symbol from the store
    let source = match std::fs::File::open(file.as_str()).and_then(std::io::read_to_string) {
        Ok(v) => v,
        Err(e) => {
            println!("failed to read file {file}.ftd: {e:?}");
            return vec![];
        }
    };

    let d = fastn_unresolved::parse(&compiler.main_package, module, &source, &mut compiler.arena);

    d.definitions
        .into_iter()
        .map(|d| match d {
            fastn_unresolved::UR::UnResolved(mut v) => {
                v.symbol =
                    Some(module.symbol(v.name.unresolved().unwrap().str(), &mut compiler.arena));
                fastn_unresolved::UR::UnResolved(v)
            }
            _ => {
                unreachable!("fastn_unresolved::parse() only returns unresolved definitions")
            }
        })
        .collect::<Vec<_>>()
}

pub fn lookup(
    compiler: &mut fastn_compiler::Compiler,
    symbols: std::collections::HashSet<fastn_section::Symbol>,
) -> Vec<fastn_unresolved::Urd> {
    let unique_modules = symbols
        .iter()
        .map(|s| file_for_symbol(s, &mut compiler.arena))
        .collect::<std::collections::HashSet<_>>();

    unique_modules
        .into_iter()
        .flat_map(|m| find_all_definitions_in_a_module(compiler, m))
        .collect()
}

fn file_for_symbol(
    symbol: &fastn_section::Symbol,
    arena: &mut fastn_section::Arena,
) -> (String, fastn_section::Module) {
    (
        // this code is nonsense right now
        match symbol.module(arena) {
            Some(module) => format!("{}/{}.ftd", symbol.package(arena), module),
            None => format!("{}/index.ftd", symbol.package(arena)),
        },
        symbol.parent(arena),
    )
}
