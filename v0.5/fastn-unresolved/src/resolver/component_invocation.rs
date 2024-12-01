impl fastn_unresolved::ComponentInvocation {
    pub fn resolve(
        &mut self,
        definitions: &std::collections::HashMap<String, fastn_unresolved::URD>,
        arena: &mut fastn_unresolved::Arena,
        output: &mut fastn_unresolved::resolver::Output,
    ) {
        for c in self.children.iter_mut() {
            if let fastn_unresolved::UR::UnResolved(ref mut c) = c {
                c.resolve(definitions, arena, output);
            }
        }

        fastn_unresolved::resolver::symbol::resolve(
            &self.module,
            &mut self.name,
            definitions,
            arena,
            output,
            &[], // TODO
        );

        let _component = match self.name {
            fastn_unresolved::UR::Resolved(ref name) => {
                get_component(definitions, arena, name).unwrap()
            }
            // in case of error or not found, nothing left to do
            _ => return,
        };

        // dbg!(component);
    }
}

pub fn get_component<'a>(
    definitions: &'a std::collections::HashMap<String, fastn_unresolved::URD>,
    arena: &fastn_unresolved::Arena,
    symbol: &fastn_unresolved::Symbol,
) -> Option<&'a fastn_resolved::ComponentDefinition> {
    println!("looking for: {}", symbol.str(arena));
    if let Some(fastn_unresolved::UR::Resolved(fastn_resolved::Definition::Component(v))) =
        definitions.get(symbol.str(arena))
    {
        println!("found in definitions");
        return Some(v);
    }
    if let Some(fastn_resolved::Definition::Component(v)) =
        fastn_builtins::builtins().get(symbol.str(arena))
    {
        println!("found in builtins");
        return Some(v);
    }
    println!("not found");
    None
}
