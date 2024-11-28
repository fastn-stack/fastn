impl fastn_unresolved::ComponentInvocation {
    pub fn resolve(
        &mut self,
        definitions: &std::collections::HashMap<fastn_unresolved::Symbol, fastn_unresolved::URD>,
        interner: &mut string_interner::DefaultStringInterner,
        output: &mut fastn_unresolved::resolver::Output,
    ) {
        for c in self.children.iter_mut() {
            if let fastn_unresolved::UR::UnResolved(ref mut c) = c {
                c.resolve(definitions, interner, output);
            }
        }

        fastn_unresolved::resolver::symbol::resolve(
            &self.module,
            &mut self.name,
            definitions,
            interner,
            output,
            &[], // TODO
        );

        let component = match self.name {
            fastn_unresolved::UR::Resolved(ref name) => {
                get_component(definitions, interner, name).unwrap()
            }
            // in case of error or not found, nothing left to do
            _ => return,
        };

        dbg!(component);
    }
}

pub fn get_component<'a>(
    definitions: &'a std::collections::HashMap<fastn_unresolved::Symbol, fastn_unresolved::URD>,
    interner: &string_interner::DefaultStringInterner,
    symbol: &fastn_unresolved::Symbol,
) -> Option<&'a fastn_resolved::ComponentDefinition> {
    println!("looking for: {}", symbol.str(interner));
    if let Some(fastn_unresolved::UR::Resolved(fastn_resolved::Definition::Component(v))) =
        definitions.get(symbol)
    {
        println!("found in definitions");
        return Some(v);
    }
    if let Some(fastn_resolved::Definition::Component(v)) =
        fastn_builtins::builtins().get(symbol.str(interner))
    {
        println!("found in builtins");
        return Some(v);
    }
    println!("not found");
    None
}
