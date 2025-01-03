/// how to resolve local symbols, e.g., inside a function / component
///
/// locals is the stack of locals (excluding globals).
///
/// e.g., inside a function we can have block containing blocks, and each block may have defined
/// some variables, each such nested block is passed as locals,
/// with the innermost block as the last entry.
#[allow(clippy::too_many_arguments)]
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
    _modules: &std::collections::HashMap<fastn_unresolved::Module, bool>,
    arena: &mut fastn_unresolved::Arena,
    output: &mut fastn_unresolved::resolver::Output,
    _locals: &[Vec<fastn_unresolved::UR<fastn_unresolved::Argument, fastn_resolved::Argument>>],
    _main_package: &fastn_package::MainPackage,
) -> bool {
    let inner_name = if let fastn_unresolved::UR::UnResolved(name) = name {
        name
    } else {
        return true;
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
            fastn_unresolved::Symbol::new(
                package.str(),
                module.as_ref().map(|v| v.str()),
                name.str(),
                arena,
            )
        }
        fastn_section::IdentifierReference::Local(name) => {
            // we combine the name with current_module to create the target symbol.
            // but what if the name is an alias?
            // we resolve the alias first.
            match arena.aliases.get(aid).and_then(|v| v.get(name.str())) {
                Some(fastn_unresolved::SoM::Symbol(s)) => s.clone(),
                Some(fastn_unresolved::SoM::Module(_)) => {
                    // report an error, this function always resolves a symbol
                    todo!()
                }
                None => current_module.symbol(name.str(), arena),
            }
        }
        fastn_section::IdentifierReference::Imported {
            module,
            name: dotted_name,
        } => {
            let o = arena.module_alias(aid, module.str());
            match o {
                Some(fastn_unresolved::SoM::Module(m)) => m.symbol(dotted_name.str(), arena),
                Some(fastn_unresolved::SoM::Symbol(_s)) => {
                    // report an error, this function always resolves a symbol
                    todo!()
                }
                None => {
                    // there are two cases where this is valid.
                    // a, if the module was defined in the current module using the future
                    // `-- module foo: ` syntax.
                    // b, this is a foo.bar case, where foo is the name of component, and we have
                    // to look for bar in the arguments.
                    //
                    // note that in case of b, if the argument type was a module, we may have more
                    // than one dots present in the dotted_name, and we will have to parse it
                    // appropriately
                    todo!()
                }
            }
        }
    };

    // check if target symbol is part of a direct dependency.
    let target_symbol_key = target_symbol.str(arena);

    match definitions.get(target_symbol_key) {
        Some(fastn_unresolved::UR::UnResolved(_)) => {
            output.stuck_on.insert(target_symbol);
            false
        }
        Some(fastn_unresolved::UR::NotFound) => {
            *name = fastn_unresolved::UR::Invalid(fastn_section::Error::InvalidIdentifier);
            true
        }
        Some(fastn_unresolved::UR::Invalid(_)) => {
            todo!()
        }
        Some(fastn_unresolved::UR::InvalidN(_)) => {
            todo!()
        }
        Some(fastn_unresolved::UR::Resolved(_)) => {
            *name = fastn_unresolved::UR::Resolved(Some(target_symbol));
            true
        }
        None => {
            if fastn_builtins::builtins().contains_key(target_symbol_key) {
                *name = fastn_unresolved::UR::Resolved(Some(target_symbol));
            } else {
                *name = fastn_unresolved::UR::Invalid(fastn_section::Error::InvalidIdentifier);
            }
            true
        }
    }
}
