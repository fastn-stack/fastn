/// Resolve symbols, e.g., inside a function / component
#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(definitions, arena))]
pub fn symbol(
    aid: fastn_section::AliasesID,
    // foo.ftd (current_module = foo, package = foo, module = "")

    // [amitu.com/bar] (amitu.com/bar/FASTN: dependency amitu.com/foo)
    //
    // -- import: amitu.com/foo (Alias: foo -> SoM::M<amitu.com/foo>)
    // exposing: bar            (bar -> SoM::S(<amitu.com/foo#bar>)
    // -- amitu.com/foo#bar:
    current_module: fastn_section::Module,
    // parent: Option<fastn_section::Symbol>,
    // S1_name="bar.x"
    // S2_name="x"
    // S3_name="bar#x"
    name: &mut fastn_unresolved::URIS,
    definitions: &std::collections::HashMap<String, fastn_unresolved::URD>,
    arena: &mut fastn_section::Arena,
    output: &mut fastn_unresolved::resolver::Output,
    // locals is the stack of locals (excluding globals).
    //
    // e.g., inside a function we can have block containing blocks, and each block may have defined
    // some variables, each such nested block is passed as locals,
    // with the innermost block as the last entry.
    locals: &[Vec<fastn_unresolved::UR<fastn_unresolved::Argument, fastn_resolved::Argument>>],
    main_package: &fastn_package::MainPackage,
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
            tracing::info!("Absolute: {} {:?} {}", package.str(), module, name.str());
            fastn_section::Symbol::new(
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
                Some(fastn_section::SoM::Symbol(s)) => s.clone(),
                Some(fastn_section::SoM::Module(_)) => {
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
                Some(fastn_section::SoM::Module(m)) => m.symbol(dotted_name.str(), arena),
                Some(fastn_section::SoM::Symbol(_s)) => {
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
            tracing::info!("{} is unresolved, adding to stuck_on", target_symbol_key);
            output.stuck_on.insert(target_symbol);
            false
        }
        Some(fastn_unresolved::UR::NotFound) => {
            tracing::info!("{} not found", target_symbol_key);
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
            tracing::info!("Found a resolved definition for {}", target_symbol_key);
            *name = fastn_unresolved::UR::Resolved(Some(target_symbol));
            true
        }
        None => {
            tracing::info!(
                "No definition exist for {}, checking builtins",
                target_symbol_key
            );
            if fastn_builtins::builtins().contains_key(target_symbol_key) {
                tracing::info!("Found {} in builtins", target_symbol_key);
                *name = fastn_unresolved::UR::Resolved(Some(target_symbol));
            } else {
                *name = fastn_unresolved::UR::Invalid(fastn_section::Error::InvalidIdentifier);
            }
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn t_(
        name_to_resolve: &str,
        current_module: &str,
        _sources: std::collections::HashMap<String, String>,
        _dependencies: std::collections::HashMap<String, Vec<String>>,
        expected: &str,
    ) {
        let package_name = "main";
        let main_package = fastn_package::MainPackage {
            name: package_name.to_string(),
            systems: vec![],
            apps: vec![],
            packages: Default::default(),
        };

        let mut arena = fastn_section::Arena::default();

        let document = {
            let source = format!("--{name_to_resolve}: basic");
            let module = if current_module.is_empty() {
                fastn_section::Module::main(&mut arena)
            } else {
                fastn_section::Module::new(package_name, Some(current_module), &mut arena)
            };
            fastn_unresolved::parse(&main_package, module, &source, &mut arena)
        };

        let mut name = document
            .content
            .first()
            .unwrap()
            .unresolved()
            .unwrap()
            .name
            .clone();

        let resolved = symbol(
            document.aliases.unwrap(),
            document.module,
            &mut name,
            &Default::default(),
            &mut arena,
            &mut fastn_unresolved::resolver::Output::default(),
            &[],
            &main_package,
        );

        assert!(resolved);

        let output_str = name.resolved().unwrap().str(&arena);

        assert_eq!(expected, output_str);
    }

    macro_rules! t {
        ($name:expr, $expected:expr) => {
            t_(
                $name,
                "",
                std::collections::HashMap::new(),
                std::collections::HashMap::new(),
                $expected,
            );
        };
    }

    #[test]
    fn basic() {
        t!("ftd.text", "ftd#text"); // Resolve builtin
        t!("ftd#text", "ftd#text"); // Resolve absolute symbol usage
        // t!("foo", "-- integer foo: 10", "main.foo");
        // t!("ftd.txt", "-- integer foo: 10", "main.foo");
        // t!("ftd#txt", "-- integer foo: 10", "main.foo");
        // t!("bar", "-- import: current-package/foo",  {"current-package/foo": "-- integer bar: 10"}, "foo.bar");
        // t!(
        //     "foo.bar",
        //     "-- import: other-package/foo",
        //     {"other-package/foo": "-- integer bar: 10"},
        //     {"current-package": ["other-package"]},
        //     "foo.bar"
        // );
        // t!(
        //     "foo.bar",
        //     "-- import: other-package/foo",
        //     {"other-package/foo": "-- public integer bar: 10"},
        //     {"current-package": ["other-package"]},
        //     "foo.bar"
        // );
    }
}
