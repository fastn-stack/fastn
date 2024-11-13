impl fastn_unresolved::Document {
    pub(crate) fn new(
        document: fastn_section::Document,
    ) -> (fastn_unresolved::Document, Vec<fastn_section::Section>) {
        (
            fastn_unresolved::Document {
                module_doc: document.module_doc,
                imports: vec![],
                definitions: Default::default(),
                content: vec![],
                errors: document.errors,
                warnings: document.warnings,
                comments: document.comments,
                line_starts: document.line_starts,
            },
            document.sections,
        )
    }
}

pub(crate) fn assert_no_body(
    section: &fastn_section::Section,
    document: &mut fastn_unresolved::Document,
) -> bool {
    if section.body.is_some() {
        document.errors.push(
            section
                .init
                .name
                .name
                .name
                .wrap(fastn_section::Error::BodyNotAllowed),
        );
        return false;
    }

    true
}

pub(crate) fn assert_no_children(
    section: &fastn_section::Section,
    document: &mut fastn_unresolved::Document,
) -> bool {
    if !section.children.is_empty() {
        document.errors.push(
            section
                .init
                .name
                .name
                .name
                .wrap(fastn_section::Error::BodyNotAllowed),
        );
        return false;
    }

    true
}

pub(crate) fn assert_no_extra_headers(
    source: &str,
    section: &fastn_section::Section,
    document: &mut fastn_unresolved::Document,
    allowed: &[&str],
) -> bool {
    let mut found = false;
    for header in &section.headers {
        if !allowed.contains(&header.name(source)) {
            document
                .errors
                .push(header.span().wrap(fastn_section::Error::ExtraArgumentFound));
            found = true;
        }
    }

    !found
}

impl From<&str> for fastn_unresolved::Identifier {
    fn from(s: &str) -> fastn_unresolved::Identifier {
        fastn_unresolved::Identifier(s.to_string())
    }
}

impl std::str::FromStr for fastn_unresolved::Identifier {
    type Err = ();

    fn from_str(s: &str) -> Result<fastn_unresolved::Identifier, ()> {
        Ok(fastn_unresolved::Identifier(s.to_string()))
    }
}
