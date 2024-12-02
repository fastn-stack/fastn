/// how to resolve local symbols, e.g., inside a function / component
///
/// locals is the stack of locals (excluding globals).
///
/// e.g., inside a function we can have block containing blocks, and each block may have defined
/// some variables, each such nested block is passed as locals,
/// with the innermost block as the last entry.
pub fn symbol(
    aid: fastn_unresolved::AliasesID,
    // foo.ftd (current_module = foo, package = foo, module = "")
    current_module: &fastn_unresolved::Module,
    // parent: Option<fastn_unresolved::Symbol>,
    // S1_name="bar.x"
    // S2_name="x"
    // S3_name="bar#x"
    name: &mut fastn_unresolved::URIS,
    definitions: &std::collections::HashMap<String, fastn_unresolved::URD>,
    modules: &std::collections::HashMap<fastn_unresolved::Module, bool>,
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
        } => {
            // this is S3, S3_name="bar#x"
            // we have to check if bar#x exists in definitions, and is resolved.
            // if it is in resolved-state we have resolved, and we store bar#x.
            // if it is not in unresolved-state, or if it is missing in definitions, we add "bar#x"
            // to output.stuck_on.
            // if it is in error state, or not found state, we resolve ourselves as them.
            fastn_unresolved::Symbol::new(package.str(), Some(module.str()), name.str(), arena)
        }
        fastn_section::IdentifierReference::Local(name) => {
            // we combine the name with current_module to create the target symbol.
            // but what if the name is an alias?
            // we resolve the alias first.
            match arena.aliases.get(aid).and_then(|v| v.get(name.str())) {
                Some(fastn_unresolved::SoM::Symbol(s)) => s.clone(),
                Some(fastn_unresolved::SoM::Module(_)) => {
                    // report an error, resolve always resolves
                    todo!()
                }
                None => current_module.symbol(name.str(), arena),
            }
        }
        fastn_section::IdentifierReference::Imported {
            module,
            name: dotted_name,
        } => {
            let target_symbol = current_module.symbol(module.str(), arena);
            // current module is foo.ftd
            // it has -- import: bar at the top
            // the value of module and name is "bar" and "x" respectively
            // current_module.symbol(module) should be a module alias, we should get the target
            // module from the module alias.
            // we combine the target module with the name, "x" to get the target symbol.
            match definitions.get(target_symbol.str(arena)) {
                // Some(fastn_unresolved::UR::Resolved(fastn_resolved::Definition::Module {
                //     package,
                //     module,
                //     ..
                // })) => {
                //     // what we stored in resolved place, no further action is required.
                //     *name = fastn_unresolved::UR::Resolved(fastn_unresolved::Symbol::new(
                //         package,
                //         module.as_deref(),
                //         dotted_name.str(),
                //         arena,
                //     ));
                //     return;
                // }
                // Some(fastn_unresolved::UR::UnResolved(_)) => {
                //     output.stuck_on.insert(target_symbol);
                //     return;
                // }
                // Some(fastn_unresolved::UR::NotFound) => {
                //     *name = fastn_unresolved::UR::Invalid(fastn_section::Error::InvalidIdentifier);
                //     return;
                // }
                _ => {
                    // we have
                    todo!()
                }
            }
        }
    };

    *name = fastn_unresolved::UR::Resolved(target_symbol);
}
