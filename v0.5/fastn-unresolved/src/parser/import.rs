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
    fastn_unresolved::utils::assert_no_extra_headers(&section, document, &["exports", "exposing"]);
    validate_import_module_in_dependencies(section, document, arena, package, &i);

    // Add import in document
    add_import_in_document(document, arena, &i);

    // Add Symbol aliases
    add_symbol_aliases(document, arena, &i, main_package_name, package);
}

fn add_import_in_document(
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_section::Arena,
    i: &Import,
) {
    let alias = i
        .alias
        .as_ref()
        .map(|alias| alias.str().to_string())
        .unwrap_or_else(|| i.module.package(arena).to_string());

    match document.aliases {
        Some(id) => {
            arena
                .aliases
                .get_mut(id)
                .unwrap()
                .insert(alias, fastn_section::SoM::Module(i.module));
        }
        None => {
            let aliases =
                fastn_section::Aliases::from_iter([(alias, fastn_section::SoM::Module(i.module))]);
            document.aliases = Some(arena.aliases.alloc(aliases));
        }
    }
}

fn is_main_package(package: &Option<&fastn_package::Package>, main_package_name: &str) -> bool {
    match package {
        Some(package) => package.name == main_package_name,
        None => false,
    }
}

fn add_symbol_aliases(
    _document: &mut fastn_unresolved::Document,
    _arena: &mut fastn_section::Arena,
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

    let _alias = match alias {
        Some(alias) => alias,
        None => return,
    };

    todo!()
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
            let is_valid_import = package.dependencies.iter().any(|dep| {
                dep.name.as_str() == imported_package_name || dep.name.as_str() == package.name
            });

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
    let (module, alias) = match caption.str().split_once(" as ") {
        Some((module, alias)) => (module, Some(alias)),
        None => (caption.str(), None),
    };

    let (package, module) = match module.rsplit_once("/") {
        Some((package, module)) => (package, Some(module)),
        None => (module, None),
    };

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
        alias: alias.map(|v| fastn_section::Identifier {
            name: caption.inner_str(v),
        }),
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
    pub alias: Option<fastn_section::Identifier>,
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
            .map(|v| aliasable(header, v))
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
    fastn_unresolved::tt!(import_function, tester);

    #[test]
    fn import() {
        t!("-- import: foo", { "import": "foo" });
        // t!("-- import: foo.fifthtry.site/bar", { "import": "foo.fifthtry.site/bar" });
        // t!("-- import: foo as f", { "import": "foo=>f" });
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
            fastn_section::JIDebug::idebug(&AliasesID(d.aliases.unwrap()), arena),
            expected
        )
    }

    fn import_function(
        section: fastn_section::Section,
        document: &mut fastn_unresolved::Document,
        arena: &mut fastn_section::Arena,
        package: &Option<&fastn_package::Package>,
    ) {
        super::import(section, document, arena, package, "foo");
    }

    #[test]
    #[should_panic]
    fn failing_tests() {
        t!("-- import: foo as f\nexposing: x", { "import": "foo=>f", "exposing": ["x"] });
        t!("-- import: foo\nexposing: x", { "import": "foo", "exposing": ["x"] });
        t!("-- import: foo\nexposing: x, y, z", { "import": "foo", "exposing": ["x", "y", "z"] });
        t!("-- import: foo as f\nexposing: x as y", { "import": "foo as f", "exposing": ["x=>y"] });
        t!("-- import: foo as f\nexposing: x as y, z", { "import": "foo as f", "exposing": ["x=>y", "z"] });
        t!("-- import: foo as f\nexport: x", { "import": "foo=>f", "export": ["x"] });
        t!("-- import: foo\nexport: x", { "import": "foo", "export": ["x"] });
        t!("-- import: foo\nexport: x, y, z", { "import": "foo", "export": ["x", "y", "z"] });
        t!("-- import: foo as f\nexport: x as y", { "import": "foo as f", "export": ["x=>y"] });
        t!("-- import: foo as f\nexport: x as y, z", { "import": "foo as f", "export": ["x=>y", "z"] });
        t!("-- import: foo as f\nexport: x\nexposing: y", { "import": "foo=>f", "export": ["x"], "exposing": ["y"] });
        t!("-- import: foo\nexport: x\nexposing: y", { "import": "foo", "export": ["x"], "exposing": ["y"] });
        t!("-- import: foo\nexport: x, y, z\nexposing: y", { "import": "foo", "export": ["x", "y", "z"], "exposing": ["y"] });
        t!("-- import: foo as f\nexport: x as y\nexposing: y", { "import": "foo as f", "export": ["x=>y"], "exposing": ["y"] });
        t!("-- import: foo as f\nexport: x as y, z\nexposing: y", { "import": "foo as f", "export": ["x=>y", "z"], "exposing": ["y"] });
    }

    #[derive(Debug)]
    struct AliasesID(fastn_section::AliasesID);
    impl fastn_section::JIDebug for AliasesID {
        fn idebug(&self, arena: &fastn_section::Arena) -> serde_json::Value {
            let aliases = arena.aliases.get(self.0).unwrap();
            let mut o = serde_json::Map::new();
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
                    fastn_section::SoM::Symbol(_) => todo!(),
                }
            }

            serde_json::Value::Object(o)
        }
    }

    impl fastn_section::JIDebug for super::Import {
        fn idebug(&self, arena: &fastn_section::Arena) -> serde_json::Value {
            let mut o = serde_json::Map::new();

            let name = if self.module.package(arena).is_empty() {
                self.module.module(arena).to_string()
            } else {
                format!(
                    "{}/{}",
                    self.module.package(arena),
                    self.module.module(arena)
                )
            };

            o.insert(
                "import".into(),
                match self.alias {
                    Some(ref v) => format!("{name}=>{}", v.str()),
                    None => name,
                }
                .into(),
            );

            if let Some(ref v) = self.export {
                o.insert("export".into(), v.idebug(arena));
            }

            if let Some(ref v) = self.exposing {
                o.insert("exposing".into(), v.idebug(arena));
            }

            serde_json::Value::Object(o)
        }
    }

    impl fastn_section::JIDebug for super::Export {
        fn idebug(&self, arena: &fastn_section::Arena) -> serde_json::Value {
            match self {
                super::Export::All => "all".into(),
                super::Export::Things(v) => {
                    serde_json::Value::Array(v.iter().map(|v| v.idebug(arena)).collect())
                }
            }
        }
    }

    impl fastn_section::JIDebug for super::AliasableIdentifier {
        fn idebug(&self, _arena: &fastn_section::Arena) -> serde_json::Value {
            match self.alias {
                Some(ref v) => format!("{}=>{}", self.name.str(), v.str()),
                None => self.name.str().to_string(),
            }
            .into()
        }
    }
}
