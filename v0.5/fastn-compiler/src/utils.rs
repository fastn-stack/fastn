pub(crate) fn resolved_content(
    content: Vec<fastn_unresolved::URCI>,
) -> Vec<fastn_resolved::ComponentInvocation> {
    // self.content should be all UR::R now
    // every symbol in self.symbol_used in the bag must be UR::R now
    content.into_iter().map(|ur| ur.into_resolved()).collect()
}

pub(crate) fn used_definitions(
    definitions: std::collections::HashMap<fastn_unresolved::Symbol, fastn_unresolved::URD>,
    definitions_used: std::collections::HashSet<fastn_unresolved::Symbol>,
    interner: string_interner::DefaultStringInterner,
) -> indexmap::IndexMap<String, fastn_resolved::Definition> {
    // go through self.symbols_used and get the resolved definitions
    let def_map = indexmap::IndexMap::new();
    for definition in definitions_used.iter() {
        if let Some(_definition) = definitions.get(definition) {
            // definitions.insert(symbol.clone(), definition);
            todo!()
        } else if let Some(_definition) = fastn_builtins::builtins().get(definition.str(&interner))
        {
            // definitions.insert(symbol.clone(), definition);
            todo!()
        }
    }
    def_map
}
