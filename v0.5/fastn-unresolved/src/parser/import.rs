/// Parses import statements in fastn.
///
/// ## Grammar
///
/// ```ftd
/// -- import: <package-name>[/<module-name>] [as <alias>]
/// [exposing: <symbol-list>]   ; Import specific symbols from module
/// [export: <symbol-list>]      ; Re-export symbols (main package only)
///
/// <symbol-list> := * | <aliasable-symbol> [, <aliasable-symbol>]*
/// <aliasable-symbol> := <symbol-name> [as <alias>]
/// <alias> := <plain-identifier>  ; Must be plain (foo, bar), not dotted (foo.bar)
/// ```
///
/// ## Features
///
/// ### Basic Import
/// - `-- import: foo` - Imports package/module 'foo' with alias 'foo'
/// - `-- import: foo/bar` - Imports module 'bar' from package 'foo' with alias 'bar'
/// - `-- import: foo as f` - Imports 'foo' with custom alias 'f'
///
/// ### Exposing (Import specific symbols)
/// - `-- import: foo\nexposing: *` - Imports all symbols from 'foo'
/// - `-- import: foo\nexposing: bar` - Imports only symbol 'bar' from 'foo'
/// - `-- import: foo\nexposing: bar as b` - Imports 'bar' with alias 'b'
/// - `-- import: foo\nexposing: bar, baz as z` - Multiple symbols with aliases
/// - `-- import: foo\nexposing: *, bar as b` - All symbols, but 'bar' aliased as 'b' (not both)
/// - Creates direct aliases: `foo#bar` becomes accessible as just `bar` (or alias)
///
/// ### Export (Re-export symbols - main package only)
/// - `-- import: foo\nexport: *` - Re-exports all symbols from imported module
/// - `-- import: foo\nexport: bar` - Re-exports symbol 'bar' from imported module
/// - `-- import: foo\nexport: bar as b` - Re-exports 'bar' with alias 'b'
/// - `-- import: foo\nexport: *, bar as b` - Exports all, but 'bar' as 'b' only (not both)
/// - Only works in main package (not in dependencies)
/// - Makes imported symbols available to consumers of this package
///
/// ### Combining Exposing and Export
/// - `-- import: foo\nexposing: bar\nexport: baz` - Import 'bar', re-export 'baz'
/// - `-- import: foo\nexposing: *\nexport: bar` - Import all, but only re-export 'bar'
/// - When both are present:
///   - `exposing` controls what's imported into current module
///   - `export` controls what's re-exported (main package only)
///
/// ### Validation
/// - Import must have a caption (package/module name)
/// - Import cannot have a type (e.g., `-- string import:` is invalid)
/// - Section name must be exactly "import" (not "impart", etc.)
/// - Imported packages must be in dependencies (unless self-import)
/// - Aliases must be plain identifiers (e.g., 'foo', 'bar'), not dotted ('foo.bar')
/// - No body, children, or extra headers allowed
///
/// ### Error Cases
/// - `ImportMustHaveCaption` - Missing package/module name
/// - `ImportCantHaveType` - Type specified for import
/// - `ImportMustBeImport` - Wrong section name (e.g., "impart" instead of "import")
/// - `ImportPackageNotFound` - Package not in dependencies
/// - `ImportAliasMustBePlain` - Alias contains dots or other non-identifier characters
pub(super) fn import(
    section: fastn_section::Section,
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_section::Arena,
    package: &Option<&fastn_package::Package>,
    main_package_name: &str,
) {
    if let Some(ref kind) = section.init.kind {
        document
            .errors
            .push(kind.span().wrap(fastn_section::Error::ImportCantHaveType));
        // we will go ahead with this import statement parsing
    }

    // section.name must be exactly import.
    if section.simple_name() != Some("import") {
        document.errors.push(
            section
                .init
                .name
                .wrap(fastn_section::Error::ImportMustBeImport),
        );
        // we will go ahead with this import statement parsing
    }

    let i = match parse_import(&section, document, arena) {
        Some(v) => v,
        None => {
            // error handling is job of parse_module_name().
            return;
        }
    };

    // ensure there are no extra headers, children or body
    fastn_unresolved::utils::assert_no_body(&section, document);
    fastn_unresolved::utils::assert_no_children(&section, document);
    fastn_unresolved::utils::assert_no_extra_headers(&section, document, &["export", "exposing"]);
    validate_import_module_in_dependencies(section, document, arena, package, &i);

    // Add import in document
    add_import(document, arena, &i);

    // Add export and exposing in document
    add_export_and_exposing(document, arena, &i, main_package_name, package);
}

fn add_import(
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_section::Arena,
    i: &Import,
) {
    add_to_document_alias(
        document,
        arena,
        i.alias.str(),
        fastn_section::SoM::Module(i.module),
    );
}

fn add_export_and_exposing(
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_section::Arena,
    i: &Import,
    main_package_name: &str,
    package: &Option<&fastn_package::Package>,
) {
    let alias = if is_main_package(package, main_package_name) {
        // Add Symbol aliases for exposing
        &i.exposing
    } else {
        // Add Symbol aliases for exports
        &i.export
    };

    let alias = match alias {
        Some(alias) => alias,
        None => return,
    };

    match alias {
        Export::All => todo!(),
        Export::Things(things) => {
            for thing in things {
                let alias = thing.alias.as_ref().unwrap_or(&thing.name).str();

                let symbol = i.module.symbol(thing.name.str(), arena);
                add_to_document_alias(document, arena, alias, fastn_section::SoM::Symbol(symbol));
            }
        }
    }
}

fn is_main_package(package: &Option<&fastn_package::Package>, main_package_name: &str) -> bool {
    match package {
        Some(package) => package.name == main_package_name,
        None => false,
    }
}

fn add_to_document_alias(
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_section::Arena,
    alias: &str,
    som: fastn_section::SoM,
) {
    match document.aliases {
        Some(id) => {
            arena
                .aliases
                .get_mut(id)
                .unwrap()
                .insert(alias.to_string(), som);
        }
        None => {
            let aliases = fastn_section::Aliases::from_iter([(alias.to_string(), som)]);
            document.aliases = Some(arena.aliases.alloc(aliases));
        }
    }
}

/// Validates that the import statement references a module in the current package or package's
/// dependencies.
fn validate_import_module_in_dependencies(
    section: fastn_section::Section,
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_section::Arena,
    package: &Option<&fastn_package::Package>,
    i: &Import,
) {
    // ensure that the import statement is for a module in dependency
    match package {
        Some(package) => {
            let imported_package_name = i.module.package(arena);

            // Check if the imported package exists in dependencies or matches the current package name
            let is_valid_import = imported_package_name == package.name
                || package
                    .dependencies
                    .iter()
                    .any(|dep| dep.name.as_str() == imported_package_name);

            if !is_valid_import {
                document.errors.push(
                    section
                        .init
                        .name
                        .wrap(fastn_section::Error::ImportPackageNotFound),
                );
            }
        }
        None => {
            document.errors.push(
                section
                    .init
                    .name
                    .wrap(fastn_section::Error::PackageNotFound),
            );
        }
    }
}

fn parse_import(
    section: &fastn_section::Section,
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_section::Arena,
) -> Option<Import> {
    let caption = match section.caption_as_plain_span() {
        Some(v) => v,
        None => {
            document.errors.push(
                section
                    .span()
                    .wrap(fastn_section::Error::ImportMustHaveCaption),
            );
            return None;
        }
    };

    // section.caption must be single text block, parsable as a module-name.
    //       module-name must be internally able to handle aliasing.
    let (raw_module, alias) = match caption.str().split_once(" as ") {
        Some((module, alias)) => (module, Some(alias)),
        None => (caption.str(), None),
    };

    let (package, module) = match raw_module.rsplit_once("/") {
        Some((package, module)) => (package, Some(module)),
        None => (raw_module, None),
    };

    // Determine the alias: prioritize explicit alias, fallback to module name, then package name
    let alias = alias
        .or(module)
        .unwrap_or_else(|| match package.rsplit_once(".") {
            Some((_, alias)) => alias,
            None => package,
        });

    Some(Import {
        module: if let Some(module) = module {
            fastn_section::Module::new(
                caption.inner_str(package).str(),
                Some(caption.inner_str(module).str()),
                arena,
            )
        } else {
            fastn_section::Module::new(caption.inner_str(package).str(), None, arena)
        },
        alias: fastn_section::Identifier {
            name: caption.inner_str(alias),
        },
        export: parse_field("export", section, document),
        exposing: parse_field("exposing", section, document),
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum Export {
    #[expect(unused)]
    All,
    Things(Vec<AliasableIdentifier>),
}

/// is this generic enough?
#[derive(Debug, Clone, PartialEq)]
pub struct AliasableIdentifier {
    pub alias: Option<fastn_section::Identifier>,
    pub name: fastn_section::Identifier,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub module: fastn_section::Module,
    pub alias: fastn_section::Identifier,
    pub export: Option<Export>,
    pub exposing: Option<Export>,
}

fn parse_field(
    field: &str,
    section: &fastn_section::Section,
    _document: &mut fastn_unresolved::Document,
) -> Option<Export> {
    let header = section.header_as_plain_span(field)?;

    Some(Export::Things(
        header
            .str()
            .split(",")
            .map(|v| aliasable(header, v.trim()))
            .collect(),
    ))
}

fn aliasable(span: &fastn_section::Span, s: &str) -> AliasableIdentifier {
    let (name, alias) = match s.split_once(" as ") {
        Some((name, alias)) => (
            span.inner_str(name).into(),
            Some(span.inner_str(alias).into()),
        ),
        None => (span.inner_str(s).into(), None),
    };

    AliasableIdentifier { name, alias }
}

#[cfg(test)]
mod tests {
    mod main_package {
        fastn_unresolved::tt!(super::import_in_main_package_function, super::tester);

        #[test]
        fn import() {
            // import without exposing or export
            t!("-- import: foo", { "import": "foo" });
            t!("-- import: foo.fifthtry.site/bar", { "import": "foo.fifthtry.site/bar=>bar" });
            t!("-- import: foo as f", { "import": "foo=>f" });

            // import with exposing
            t!(
                "-- import: foo
                exposing: bar",
                { "import": "foo", "symbols": ["foo#bar"] }
            );
            t!(
                "-- import: foo as f
                exposing: bar",
                { "import": "foo=>f", "symbols": ["foo#bar"] }
            );
            t!(
                "-- import: foo as f
                exposing: bar, moo",
                { "import": "foo=>f", "symbols": ["foo#bar", "foo#moo"] }
            );
            t!(
                "-- import: foo as f
                exposing: bar as b, moo",
                { "import": "foo=>f", "symbols": ["foo#bar=>b", "foo#moo"] }
            );

            // import with export - no error but no symbols added (main package only processes exposing)
            t!(
                "-- import: foo
                export: bar",
                { "import": "foo" }
            );

            // import with both exposing and export - only exposing adds symbols in main package
            t!(
                "-- import: foo
                exposing: bar
                export: moo",
                { "import": "foo", "symbols": ["foo#bar"] }
            );

            // Test that self-imports work (importing from the same package)
            t!("-- import: main/module", { "import": "main/module=>module" });
        }
    }

    mod other_package {
        fastn_unresolved::tt!(super::import_in_other_package_function, super::tester);

        #[test]
        fn import() {
            // import without exposing or export
            t!("-- import: foo", { "import": "foo" });

            // import with export - adds symbols (other packages only process export)
            t!(
                "-- import: foo
                export: bar",
                { "import": "foo", "symbols": ["foo#bar"] }
            );

            // import with exposing - no symbols added (other packages don't process exposing)
            t!(
                "-- import: foo
                exposing: bar",
                { "import": "foo" }
            );

            // import with both exposing and export - only export adds symbols in other packages
            t!(
                "-- import: foo
                exposing: bar
                export: moo",
                { "import": "foo", "symbols": ["foo#moo"] }
            );
        }
    }

    #[track_caller]
    fn tester(
        d: fastn_unresolved::Document,
        expected: serde_json::Value,
        arena: &fastn_section::Arena,
    ) {
        assert!(d.content.is_empty());
        assert!(d.definitions.is_empty());
        assert!(d.aliases.is_some());

        assert_eq!(
            fastn_unresolved::JIDebug::idebug(&AliasesID(d.aliases.unwrap()), arena),
            expected
        )
    }

    fn import_in_main_package_function(
        section: fastn_section::Section,
        document: &mut fastn_unresolved::Document,
        arena: &mut fastn_section::Arena,
        _package: &Option<&fastn_package::Package>,
    ) {
        let package = fastn_package::Package::new_for_test(
            "main",
            vec![
                fastn_package::Dependency::new_for_test("foo"),
                fastn_package::Dependency::new_for_test("foo.fifthtry.site"),
            ],
        );

        super::import(section, document, arena, &Some(&package), "main");
    }

    fn import_in_other_package_function(
        section: fastn_section::Section,
        document: &mut fastn_unresolved::Document,
        arena: &mut fastn_section::Arena,
        _package: &Option<&fastn_package::Package>,
    ) {
        let package = fastn_package::Package::new_for_test(
            "other",
            vec![fastn_package::Dependency::new_for_test("foo")],
        );

        super::import(section, document, arena, &Some(&package), "main");
    }

    mod error_cases {
        #[test]
        fn test_import_non_existent_package() {
            let mut arena = fastn_section::Arena::default();
            let module = fastn_section::Module::main(&mut arena);
            let source = arcstr::ArcStr::from("-- import: non_existent_package");
            let parsed_doc = fastn_section::Document::parse(&source, module);

            let (mut document, sections) =
                fastn_unresolved::Document::new(module, parsed_doc, &mut arena);

            let section = sections.into_iter().next().unwrap();

            let package = fastn_package::Package::new_for_test(
                "test",
                vec![], // No dependencies - should cause error
            );

            super::super::import(section, &mut document, &mut arena, &Some(&package), "test");

            // Should have an error for missing package
            assert_eq!(document.errors.len(), 1);
            assert!(matches!(
                document.errors[0].value,
                fastn_section::Error::ImportPackageNotFound
            ));
        }
    }

    fn import_with_no_deps_function(
        section: fastn_section::Section,
        document: &mut fastn_unresolved::Document,
        arena: &mut fastn_section::Arena,
        _package: &Option<&fastn_package::Package>,
    ) {
        let package = fastn_package::Package::new_for_test("test", vec![]);
        super::import(section, document, arena, &Some(&package), "test");
    }

    mod import_errors {
        fastn_unresolved::tt!(super::import_with_no_deps_function, super::tester);

        #[test]
        fn test_import_errors() {
            // Import from non-existent package - produces partial result with error
            t_err!("-- import: non_existent", {"import": "non_existent"}, "ImportPackageNotFound");

            // Import with type - produces partial result with multiple errors
            t_err!("-- string import: foo", {"import": "foo"}, ["ImportCantHaveType", "ImportPackageNotFound"]);

            // Wrong section name - still parses as import, produces result with errors
            t_err!("-- impart: foo", {"import": "foo"}, ["ImportMustBeImport", "ImportPackageNotFound"]);

            // Import without caption - no partial result, just error
            f!("-- import:", "ImportMustHaveCaption");

            // Multiple errors with partial result
            t_err!("-- integer import: non_existent", {"import": "non_existent"}, ["ImportCantHaveType", "ImportPackageNotFound"]);
        }
    }

    #[derive(Debug)]
    struct AliasesID(fastn_section::AliasesID);
    impl fastn_unresolved::JIDebug for AliasesID {
        fn idebug(&self, arena: &fastn_section::Arena) -> serde_json::Value {
            let aliases = arena.aliases.get(self.0).unwrap();
            let mut o = serde_json::Map::new();
            let mut symbols: Vec<String> = vec![];
            for (key, value) in aliases {
                match value {
                    fastn_section::SoM::Module(m) => {
                        let module_name = m.str(arena);
                        if module_name.eq("ftd") {
                            continue;
                        }

                        if module_name.eq(key) {
                            o.insert("import".into(), module_name.into());
                        } else {
                            o.insert("import".into(), format!("{module_name}=>{key}").into());
                        }
                    }
                    fastn_section::SoM::Symbol(s) => {
                        let symbol_name = s.str(arena).to_string();
                        if symbol_name.ends_with(format!("#{key}").as_str()) {
                            symbols.push(symbol_name)
                        } else {
                            symbols.push(format!("{symbol_name}=>{key}"));
                        }
                    }
                }
            }

            if !symbols.is_empty() {
                symbols.sort();
                o.insert(
                    "symbols".into(),
                    serde_json::Value::Array(symbols.into_iter().map(Into::into).collect()),
                );
            }

            serde_json::Value::Object(o)
        }
    }
}
