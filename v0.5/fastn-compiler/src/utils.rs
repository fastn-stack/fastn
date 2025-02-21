pub(crate) fn resolved_content(
    content: Vec<fastn_unresolved::URCI>,
) -> Vec<fastn_resolved::ComponentInvocation> {
    // self.content should be all UR::R now
    // every symbol in self.symbol_used in the bag must be UR::R now
    content.into_iter().map(|ur| ur.into_resolved()).collect()
}

pub(crate) fn used_definitions(
    definitions: std::collections::HashMap<String, fastn_unresolved::URD>,
    definitions_used: std::collections::HashSet<fastn_section::Symbol>,
    arena: fastn_section::Arena,
) -> indexmap::IndexMap<String, fastn_resolved::Definition> {
    // go through self.symbols_used and get the resolved definitions
    let def_map = indexmap::IndexMap::new();
    for definition in definitions_used.iter() {
        match definitions.get(definition.str(&arena)) { Some(_definition) => {
            // definitions.insert(symbol.clone(), definition);
            todo!()
        } _ => { match fastn_builtins::builtins().get(definition.str(&arena)) { Some(_definition) => {
            // definitions.insert(symbol.clone(), definition);
            todo!()
        } _ => {}}}}
    }
    def_map
}
