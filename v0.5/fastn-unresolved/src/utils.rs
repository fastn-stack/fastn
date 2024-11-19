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

impl fastn_unresolved::Definition {
    pub fn name(&self) -> &str {
        match self.name {
            fastn_unresolved::UR::UnResolved(ref u) => u.str(),
            fastn_unresolved::UR::Resolved(ref r) => r.str(),
        }
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
    section: &fastn_section::Section,
    document: &mut fastn_unresolved::Document,
    allowed: &[&str],
) -> bool {
    let mut found = false;
    for header in &section.headers {
        if !allowed.contains(&header.name()) {
            document.errors.push(
                header
                    .name_span()
                    .wrap(fastn_section::Error::ExtraArgumentFound),
            );
            found = true;
        }
    }

    !found
}

impl<U, R> From<U> for fastn_unresolved::UR<U, R> {
    fn from(u: U) -> fastn_unresolved::UR<U, R> {
        fastn_unresolved::UR::UnResolved(u)
    }
}

impl<U, V> fastn_unresolved::UR<U, V> {
    pub fn unresolved(&self) -> Option<&U> {
        match self {
            fastn_unresolved::UR::UnResolved(u) => Some(u),
            _ => None,
        }
    }

    pub fn resolved(&self) -> Option<&V> {
        match self {
            fastn_unresolved::UR::Resolved(v) => Some(v),
            _ => None,
        }
    }
}
