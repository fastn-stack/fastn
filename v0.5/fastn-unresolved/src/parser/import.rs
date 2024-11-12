pub(super) fn import(
    source: &str,
    section: fastn_section::Section,
    document: &mut fastn_unresolved::Document,
) {
    if let Some(ref kind) = section.init.name.kind {
        document
            .errors
            .push(kind.span().wrap(fastn_section::Error::ImportCantHaveType));
        // we will go ahead with this import statement parsing
    }

    // section.name must be exactly import.
    if section.name(source) != "import" {
        document.errors.push(
            section
                .init
                .name
                .name
                .name
                .wrap(fastn_section::Error::ImportMustBeImport),
        );
        // we will go ahead with this import statement parsing
    }

    let caption = match section.caption_as_plain_string(source) {
        Some(v) => v,
        None => {
            document.errors.push(
                section
                    .span()
                    .wrap(fastn_section::Error::ImportMustHaveCaption),
            );
            return;
        }
    };

    let mut i = match parse_module_name(caption, document) {
        Some(v) => v,
        None => {
            // error handling is job of parse_module_name().
            return;
        }
    };

    // only two headers allowed: exports and exposing, unresolved them.
    parse_export(source, &section, document, &mut i);
    parse_exposing(source, &section, document, &mut i);

    // ensure there are no extra headers, children or body
    fastn_unresolved::utils::assert_no_body(&section, document);
    fastn_unresolved::utils::assert_no_sub_sections(&section, document);
    fastn_unresolved::utils::assert_no_extra_headers(
        source,
        &section,
        document,
        &["exports", "exposing"],
    );
    document.imports.push(i);
}

fn parse_module_name(
    caption: &str,
    _document: &mut fastn_unresolved::Document,
) -> Option<fastn_unresolved::Import> {
    // section.caption must be single text block, parsable as a module-name.
    //       module-name must be internally able to handle aliasing.
    let (module, alias) = match caption.split_once(" as ") {
        Some((module, alias)) => (module, Some(alias)),
        None => (caption, None),
    };

    let (package, module) = match module.split_once("/") {
        Some((package, module)) => (package, module),
        None => ("", module),
    };

    Some(fastn_unresolved::Import {
        package: fastn_unresolved::PackageName(package.to_string()),
        module: fastn_unresolved::ModuleName(module.to_string()),
        alias: alias.map(|v| fastn_unresolved::Identifier(v.to_string())),
        exports: None,
        exposing: None,
    })
}

fn parse_export(
    _source: &str,
    _section: &fastn_section::Section,
    _document: &mut fastn_unresolved::Document,
    _import: &mut fastn_unresolved::Import,
) {
}

fn parse_exposing(
    _source: &str,
    _section: &fastn_section::Section,
    _document: &mut fastn_unresolved::Document,
    _import: &mut fastn_unresolved::Import,
) {
}

#[cfg(test)]
mod tests {
    #[track_caller]
    fn t1(source: &str, expected: serde_json::Value) {
        use fastn_section::JDebug;

        let (mut document, sections) =
            fastn_unresolved::Document::new(fastn_section::Document::parse(source));

        let section = {
            assert_eq!(sections.len(), 1);
            sections.into_iter().next().unwrap()
        };

        super::import(source, section, &mut document);
        assert_eq!(document.imports.get(0).unwrap().debug(source), expected);
    }

    macro_rules! t {
        ($source:expr, $debug:tt) => {
            t1($source, serde_json::json!($debug));
        };
    }

    #[test]
    fn test_import() {
        t!("-- import: foo", { "import": "foo" });
        t!("-- import: foo.fifthtry.site/bar", { "import": "foo.fifthtry.site/bar" });
        t!("-- import: foo as f", { "import": "foo=>f" });
        t!("-- import: foo as f\nexposing: x", { "import": "foo=>f", "exposing": ["x"] });
        t!("-- import: foo\nexposing: x", { "import": "foo", "exposing": ["x"] });
        t!("-- import: foo as f\nexposing: x as y", { "import": "foo as f", "exposing": ["x=>y"] });
    }
}
