impl fastn_unresolved::Document {
    pub(crate) fn new(
        document: fastn_section::Document,
        desugared_auto_imports: &[fastn_unresolved::URD],
    ) -> (fastn_unresolved::Document, Vec<fastn_section::Section>) {
        (
            fastn_unresolved::Document {
                module_doc: document.module_doc,
                definitions: desugared_auto_imports.to_vec(),
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
}

impl fastn_unresolved::ComponentInvocation {
    pub fn resolved(self) -> Result<fastn_resolved::ComponentInvocation, Box<Self>> {
        // must be called only if `is_resolved()` has returned true
        todo!()
    }
}

impl fastn_unresolved::Definition {
    pub fn name(&self) -> &str {
        match self.name {
            fastn_unresolved::UR::UnResolved(ref u) => u.str(),
            fastn_unresolved::UR::Resolved(ref r) => r.str(),
            fastn_unresolved::UR::NotFound => unreachable!(),
            fastn_unresolved::UR::Invalid(_) => unreachable!(),
        }
    }

    pub fn resolved(self) -> Result<fastn_resolved::Definition, Self> {
        // must be called only if `is_resolved()` has returned true
        todo!()
    }
}

impl fastn_unresolved::PackageName {
    pub fn str(&self) -> &str {
        self.0.str()
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

    pub fn into_resolved(self) -> V {
        match self {
            fastn_unresolved::UR::Resolved(v) => v,
            _ => panic!(),
        }
    }
}

impl fastn_unresolved::Symbol {
    pub fn new(
        package: &str,
        module: &str,
        name: &str,
        interner: &mut string_interner::DefaultStringInterner,
    ) -> fastn_unresolved::Symbol {
        fastn_unresolved::Symbol {
            package_len: package.len() as u16,
            module_len: module.len() as u16,
            interned: interner.get_or_intern(format!("{package}/{module}#{name}")),
        }
    }

    pub fn erase_name(
        &self,
        interner: &mut string_interner::DefaultStringInterner,
    ) -> fastn_unresolved::Symbol {
        self.with_name("", interner)
    }

    pub fn with_name(
        &self,
        name: &str,
        interner: &mut string_interner::DefaultStringInterner,
    ) -> fastn_unresolved::Symbol {
        fastn_unresolved::Symbol {
            package_len: self.package_len,
            module_len: self.module_len,
            interned: interner.get_or_intern(format!(
                "{}/{}#{name}",
                self.package(interner),
                self.module(interner)
            )),
        }
    }

    pub fn symbol<'a>(&self, interner: &'a string_interner::DefaultStringInterner) -> &'a str {
        interner.resolve(self.interned).unwrap()
    }

    pub fn package<'a>(&self, interner: &'a string_interner::DefaultStringInterner) -> &'a str {
        &self.symbol(interner)[..self.package_len as usize]
    }

    pub fn module<'a>(&self, interner: &'a string_interner::DefaultStringInterner) -> &'a str {
        &self.symbol(interner)[self.package_len as usize + 1
            ..self.package_len as usize + 1 + self.module_len as usize]
    }

    pub fn name<'a>(&self, interner: &'a string_interner::DefaultStringInterner) -> &'a str {
        &self.symbol(interner)[self.package_len as usize + 1 + self.module_len as usize + 1..]
    }
}

pub fn desugar_auto_imports(
    _auto_imports: &[fastn_section::AutoImport],
) -> Vec<fastn_unresolved::URD> {
    // todo!()
    vec![]
}
