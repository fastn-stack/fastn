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
    _source: &str,
    _document: &mut fastn_unresolved::Document,
) -> Option<fastn_unresolved::Import> {
    // section.caption must be single text block, parsable as a module-name.
    //       module-name must be internally able to handle aliasing.
    todo!()
}

fn parse_export(
    _source: &str,
    _section: &fastn_section::Section,
    _document: &mut fastn_unresolved::Document,
    _import: &mut fastn_unresolved::Import,
) {
    todo!()
}

fn parse_exposing(
    _source: &str,
    _section: &fastn_section::Section,
    _document: &mut fastn_unresolved::Document,
    _import: &mut fastn_unresolved::Import,
) {
    todo!()
}
