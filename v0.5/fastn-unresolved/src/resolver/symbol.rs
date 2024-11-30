/// how to resolve local symbols, e.g., inside a function / component
///
/// locals is the stack of locals (excluding globals).
///
/// e.g., inside a function we can have block containing blocks, and each block may have defined
/// some variables, each such nested block is passed as locals,
/// with the innermost block as the last entry.
pub fn resolve(
    current_module: &fastn_unresolved::Module,
    // parent: Option<fastn_unresolved::Symbol>,
    name: &mut fastn_unresolved::URIS,
    definitions: &std::collections::HashMap<String, fastn_unresolved::URD>,
    arena: &mut fastn_unresolved::Arena,
    _output: &mut fastn_unresolved::resolver::Output,
    _locals: &[Vec<fastn_unresolved::UR<fastn_unresolved::Argument, fastn_resolved::Argument>>],
) {
    let inner_name = if let fastn_unresolved::UR::UnResolved(name) = name {
        name
    } else {
        return;
    };

    let target_symbol = match inner_name {
        fastn_section::IdentifierReference::Absolute {
            package,
            module,
            name,
        } => fastn_unresolved::Symbol::new(package.str(), module.str(), name.str(), arena),
        fastn_section::IdentifierReference::Imported { module, name } => {
            // current module is foo.ftd
            // it has -- import: bar at the top
            // the value of module and name is "bar" and "x" respectively
            // current_module.symbol(module) should be a module alias, we should get the target
            // module from the module alias.
            // we combine the target module with the name, "x" to get the target symbol.

            match definitions.get(&current_module.symbol(module.str(), arena).string(arena)) {
                Some(fastn_unresolved::UR::UnResolved(fastn_unresolved::Definition {
                    inner: fastn_unresolved::InnerDefinition::ModuleAlias(target),
                    ..
                })) => target.symbol(name.str(), arena),
                _ => {
                    // we have
                    todo!()
                }
            }
        }
        fastn_section::IdentifierReference::Local(name) => {
            // we combine name with current_module to create target symbol.
            current_module.symbol(name.str(), arena)
        }
    };

    // we verify that target symbol exists in definitions and if so we resolve it.
    // if the target symbol resolves into a module alias, it is an error.
    // if the target symbol resolves into a symbol alias, we resolve that symbol, till we get a
    // non-alias symbol.
    // if the target-symbol does not exist in definitions, we add it to output.stuck_on symbol list
    if !validate_target(&target_symbol) {
        return;
    }

    *name = fastn_unresolved::UR::Resolved(target_symbol);
}

fn validate_target(_target: &fastn_unresolved::Symbol) -> bool {
    true
}
