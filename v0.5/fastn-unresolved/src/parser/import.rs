pub(super) fn import(
    section: fastn_section::Section,
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_unresolved::Arena,
    _package: &Option<&fastn_package::Package>,
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

    let _i = match parse_import(&section, document, arena) {
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
    // TODO: assert that the import statement is for a module in dependency
    todo!()
    // document.imports.push(i);
}

fn parse_import(
    section: &fastn_section::Section,
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_unresolved::Arena,
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

    let (package, module) = match module.split_once("/") {
        Some((package, module)) => (package, Some(module)),
        None => (module, None),
    };

    Some(Import {
        module: if let Some(module) = module {
            fastn_unresolved::Module::new(
                caption.inner_str(package).str(),
                Some(caption.inner_str(module).str()),
                arena,
            )
        } else {
            fastn_unresolved::Module::new(caption.inner_str(package).str(), None, arena)
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
    pub module: fastn_unresolved::Module,
    pub alias: Option<fastn_section::Identifier>,
    pub export: Option<Export>,
    pub exposing: Option<Export>,
}

fn parse_field(
    field: &str,
    section: &fastn_section::Section,
    _document: &mut fastn_unresolved::Document,
) -> Option<Export> {
    let header = match section.header_as_plain_span(field) {
        Some(v) => v,
        None => return None,
    };

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
    #[track_caller]
    fn tester(
        d: fastn_unresolved::Document,
        _expected: serde_json::Value,
        _arena: &fastn_unresolved::Arena,
    ) {
        assert!(d.content.is_empty());
        assert!(d.definitions.is_empty());

        // assert_eq!(
        //     fastn_unresolved::JDebug::debug(&d.imports.pop().unwrap()),
        //     expected
        // )
    }

    fastn_unresolved::tt!(super::import, tester);

    #[test]
    fn import() {
        t!("-- import: foo", { "import": "foo" });
        // t!("-- import: foo.fifthtry.site/bar", { "import": "foo.fifthtry.site/bar" });
        // t!("-- import: foo as f", { "import": "foo=>f" });
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

    impl fastn_unresolved::JIDebug for super::Import {
        fn idebug(&self, arena: &fastn_unresolved::Arena) -> serde_json::Value {
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

    impl fastn_unresolved::JIDebug for super::Export {
        fn idebug(&self, arena: &fastn_unresolved::Arena) -> serde_json::Value {
            match self {
                super::Export::All => "all".into(),
                super::Export::Things(v) => {
                    serde_json::Value::Array(v.iter().map(|v| v.idebug(arena)).collect())
                }
            }
        }
    }

    impl fastn_unresolved::JIDebug for super::AliasableIdentifier {
        fn idebug(&self, _arena: &fastn_unresolved::Arena) -> serde_json::Value {
            match self.alias {
                Some(ref v) => format!("{}=>{}", self.name.str(), v.str()),
                None => self.name.str().to_string(),
            }
            .into()
        }
    }
}
