impl fastn_unresolved::ComponentInvocation {
    pub fn resolve(
        &mut self,
        definitions: &std::collections::HashMap<String, fastn_unresolved::URD>,
        modules: &std::collections::HashMap<fastn_unresolved::Module, bool>,
        arena: &mut fastn_unresolved::Arena,
        output: &mut fastn_unresolved::resolver::Output,
    ) {
        // -- foo: (foo has children)
        //    -- bar:
        // -- end: foo

        // we resolve children first (so we can do early returns after this for loop)
        for c in self.children.iter_mut() {
            if let fastn_unresolved::UR::UnResolved(ref mut c) = c {
                c.resolve(definitions, modules, arena, output);
            }
        }

        println!("{:?}", self.name);
        fastn_unresolved::resolver::symbol(
            self.aliases,
            &self.module,
            &mut self.name,
            definitions,
            modules,
            arena,
            output,
            &[], // TODO
        );
        println!("{:?}", self.name);

        let name = match self.name {
            fastn_unresolved::UR::Resolved(ref name) => name,
            // in case of error or not found, nothing left to do
            fastn_unresolved::UR::UnResolved(_) => {
                return;
            }
            // TODO: handle errors
            ref t => {
                println!("{t:?}");
                todo!()
            }
        };

        let component = match get_component(definitions, arena, name) {
            Some(fastn_unresolved::UR::Resolved(component)) => component,
            Some(fastn_unresolved::UR::UnResolved(_)) => {
                output.stuck_on.insert(name.clone());
                return;
            }
            Some(_) | None => {
                // handle error
                todo!()
            }
        };

        fastn_unresolved::resolver::arguments(
            &component.arguments,
            &mut self.caption,
            &mut self.properties,
            &mut self.body,
            &self.children,
            definitions,
            modules,
            arena,
            output,
        )
    }
}

pub fn get_component<'a>(
    definitions: &'a std::collections::HashMap<String, fastn_unresolved::URD>,
    arena: &fastn_unresolved::Arena,
    symbol: &fastn_unresolved::Symbol,
) -> Option<
    fastn_unresolved::UR<&'a fastn_unresolved::Definition, &'a fastn_resolved::ComponentDefinition>,
> {
    println!("looking for: {}", symbol.str(arena));
    match definitions.get(symbol.str(arena)) {
        Some(fastn_unresolved::UR::Resolved(fastn_resolved::Definition::Component(v))) => {
            return Some(fastn_unresolved::UR::Resolved(v))
        }
        Some(fastn_unresolved::UR::UnResolved(v)) => {
            return Some(fastn_unresolved::UR::UnResolved(v))
        }
        Some(_) | None => {}
    }

    if let Some(fastn_resolved::Definition::Component(v)) =
        fastn_builtins::builtins().get(symbol.str(arena))
    {
        println!("found in builtins");
        return Some(fastn_unresolved::UR::Resolved(v));
    }
    println!("not found");
    None
}
