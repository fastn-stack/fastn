impl fastn_unresolved::Document {
    pub(crate) fn new(
        module: fastn_section::Module,
        document: fastn_section::Document,
        arena: &mut fastn_section::Arena,
    ) -> (fastn_unresolved::Document, Vec<fastn_section::Section>) {
        (
            fastn_unresolved::Document {
                module,
                aliases: Some(arena.new_aliases()),
                module_doc: document.module_doc,
                definitions: vec![],
                content: vec![],
                errors: document.errors,
                warnings: document.warnings,
                comments: document.comments,
                line_starts: document.line_starts,
            },
            document.sections,
        )
    }

    pub fn merge(
        &mut self,
        errors: Vec<fastn_section::Spanned<fastn_section::Error>>,
        warnings: Vec<fastn_section::Spanned<fastn_section::Warning>>,
        comments: Vec<fastn_section::Span>,
    ) {
        self.errors.extend(errors);
        self.warnings.extend(warnings);
        self.comments.extend(comments);
    }

    #[expect(unused)]
    pub(crate) fn add_definitions_to_scope(
        &mut self,
        _arena: &mut fastn_section::Arena,
        _global_aliases: &fastn_section::AliasesSimple,
    ) {
        // this takes id auto imports in self.aliases, and creates a new Aliases with imports
        // merged into it, and updates the self.aliases to point to that
    }
}

impl fastn_unresolved::ComponentInvocation {
    pub fn resolve_it(&mut self) -> bool {
        // must be called only if `is_resolved()` has returned true
        todo!()
    }
}

impl fastn_unresolved::Definition {
    pub fn name(&self) -> &str {
        match self.name {
            fastn_unresolved::UR::UnResolved(ref u) => u.str(),
            fastn_unresolved::UR::Resolved(Some(ref r)) => r.str(),
            fastn_unresolved::UR::Resolved(None) => unreachable!(),
            fastn_unresolved::UR::NotFound => unreachable!(),
            fastn_unresolved::UR::Invalid(_) => unreachable!(),
            fastn_unresolved::UR::InvalidN(_) => unreachable!(),
        }
    }

    pub fn resolved(self) -> Result<fastn_resolved::Definition, Self> {
        // must be called only if `is_resolved()` has returned true
        todo!()
    }
}

pub(crate) fn assert_no_body(
    section: &fastn_section::Section,
    document: &mut fastn_unresolved::Document,
) -> bool {
    if section.body.is_some() {
        document
            .errors
            .push(section.init.name.wrap(fastn_section::Error::BodyNotAllowed));
        return false;
    }

    true
}

pub(crate) fn assert_no_children(
    section: &fastn_section::Section,
    document: &mut fastn_unresolved::Document,
) -> bool {
    if !section.children.is_empty() {
        document
            .errors
            .push(section.init.name.wrap(fastn_section::Error::BodyNotAllowed));
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

impl fastn_continuation::FromWith<fastn_unresolved::ComponentInvocation, &fastn_section::Arena>
    for fastn_resolved::ComponentInvocation
{
    fn from(u: fastn_unresolved::ComponentInvocation, arena: &fastn_section::Arena) -> Self {
        fastn_resolved::ComponentInvocation {
            id: None,
            name: u.name.resolved().unwrap().string(arena),
            properties: u
                .properties
                .into_iter()
                .map(|u| u.into_resolved())
                .collect(),
            iteration: Box::new(None),
            condition: Box::new(None),
            events: vec![],
            children: vec![],
            source: Default::default(),
            line_number: 0,
        }
    }
}
