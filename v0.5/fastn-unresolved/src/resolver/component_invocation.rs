impl fastn_unresolved::ComponentInvocation {
    pub fn resolve(
        &mut self,
        definitions: &std::collections::HashMap<String, fastn_unresolved::URD>,
        arena: &mut fastn_section::Arena,
        output: &mut fastn_unresolved::resolver::Output,
        main_package: &fastn_package::MainPackage,
    ) -> bool {
        let mut resolved = true;
        // -- foo: (foo has children)
        //    -- bar:
        // -- end: foo

        // we resolve children first (so we can do early returns after this for loop)
        for c in self.children.iter_mut() {
            if let fastn_unresolved::UR::UnResolved(ref mut c) = c {
                resolved &= c.resolve(definitions, arena, output, main_package);
            }
        }

        println!("{:?}", self.name);
        resolved &= fastn_unresolved::resolver::symbol(
            self.aliases,
            self.module,
            &mut self.name,
            definitions,
            arena,
            output,
            &[], // TODO
            main_package,
        );
        println!("{:?}", self.name);

        let name = match self.name {
            fastn_unresolved::UR::Resolved(ref name) => name,
            // in case of error or not found, nothing left to do
            fastn_unresolved::UR::UnResolved(_) => {
                return false;
            }
            // TODO: handle errors
            ref t => {
                println!("{t:?}");
                todo!()
            }
        };

        let component = match get_component(definitions, arena, name.as_ref().unwrap()) {
            Some(fastn_unresolved::UR::Resolved(component)) => component,
            Some(fastn_unresolved::UR::UnResolved(_)) => {
                output.stuck_on.insert(name.clone().unwrap());
                return false;
            }
            Some(_) | None => {
                // handle error
                todo!()
            }
        };

        resolved &= fastn_unresolved::resolver::arguments(
            &component.unwrap().arguments,
            &mut self.caption,
            &mut self.properties,
            &mut self.body,
            &self.children,
            self.module,
            definitions,
            arena,
            output,
            main_package,
        );

        resolved
    }
}

pub fn get_component<'a>(
    definitions: &'a std::collections::HashMap<String, fastn_unresolved::URD>,
    arena: &fastn_section::Arena,
    symbol: &fastn_section::Symbol,
) -> Option<
    fastn_unresolved::UR<&'a fastn_unresolved::Definition, &'a fastn_resolved::ComponentDefinition>,
> {
    println!("looking for: {}", symbol.str(arena));
    match definitions.get(symbol.str(arena)) {
        Some(fastn_unresolved::UR::Resolved(Some(fastn_resolved::Definition::Component(v)))) => {
            return Some(fastn_unresolved::UR::Resolved(Some(v)))
        }
        Some(fastn_unresolved::UR::Resolved(None)) => unreachable!(),
        Some(fastn_unresolved::UR::UnResolved(v)) => {
            return Some(fastn_unresolved::UR::UnResolved(v))
        }
        Some(_) | None => {}
    }

    if let Some(fastn_resolved::Definition::Component(v)) =
        fastn_builtins::builtins().get(symbol.str(arena))
    {
        println!("found in builtins");
        return Some(fastn_unresolved::UR::Resolved(Some(v)));
    }
    println!("not found");
    None
}
