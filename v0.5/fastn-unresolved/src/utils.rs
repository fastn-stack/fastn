impl fastn_unresolved::Document {
    pub(crate) fn new(
        module: fastn_unresolved::Module,
        document: fastn_section::Document,
        auto_imports: fastn_unresolved::AliasesID,
    ) -> (fastn_unresolved::Document, Vec<fastn_section::Section>) {
        (
            fastn_unresolved::Document {
                module,
                aliases: auto_imports,
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

    pub(crate) fn add_definitions_to_scope(&mut self, _arena: &mut fastn_unresolved::Arena) {
        // this takes id auto imports in self.aliases, and creates a new Aliases with imports
        // merged into it, and updates the self.aliases to point to that
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
        module: Option<&str>,
        name: &str,
        arena: &mut fastn_unresolved::Arena,
    ) -> fastn_unresolved::Symbol {
        let v = match module {
            Some(module) => format!("{package}/{module}#{name}"),
            None => format!("{package}#{name}"),
        };
        fastn_unresolved::Symbol {
            package_len: std::num::NonZeroU16::new(package.len() as u16).unwrap(),
            module_len: module.map(|v| std::num::NonZeroU16::new(v.len() as u16).unwrap()),
            interned: arena.interner.get_or_intern(v),
        }
    }

    pub fn parent(&self, arena: &mut fastn_unresolved::Arena) -> fastn_unresolved::Module {
        let v = match self.module_len {
            None => format!("{}/{}", self.package(arena), self.module(arena).unwrap()),
            Some(_) => self.package(arena).to_string(),
        };
        fastn_unresolved::Module {
            package_len: self.package_len,
            interned: arena.interner.get_or_intern(v),
        }
    }

    pub fn str<'a>(&self, arena: &'a fastn_unresolved::Arena) -> &'a str {
        arena.interner.resolve(self.interned).unwrap()
    }

    pub fn base<'a>(&self, arena: &'a fastn_unresolved::Arena) -> &'a str {
        &self.str(arena)[..self.package_len.get() as usize
            + self.module_len.map(|v| v.get() + 1).unwrap_or(0) as usize]
    }

    pub fn string(&self, arena: &fastn_unresolved::Arena) -> String {
        self.str(arena).to_string()
    }

    pub fn package<'a>(&self, arena: &'a fastn_unresolved::Arena) -> &'a str {
        &self.str(arena)[..self.package_len.get() as usize]
    }

    pub fn module<'a>(&self, arena: &'a fastn_unresolved::Arena) -> Option<&'a str> {
        self.module_len.map(|module_len| {
            &self.str(arena)[self.package_len.get() as usize + 1
                ..self.package_len.get() as usize + 1 + module_len.get() as usize]
        })
    }

    pub fn name<'a>(&self, arena: &'a fastn_unresolved::Arena) -> &'a str {
        &self.str(arena)[self.package_len.get() as usize
            + 1
            + self.module_len.map(|v| v.get()).unwrap_or_default() as usize
            + 1..]
    }
}

impl fastn_unresolved::Module {
    pub fn new(
        package: &str,
        module: Option<&str>,
        arena: &mut fastn_unresolved::Arena,
    ) -> fastn_unresolved::Module {
        let v = match module {
            None => package.to_string(),
            Some(module) => format!("{package}/{module}"),
        };
        fastn_unresolved::Module {
            package_len: std::num::NonZeroU16::new(package.len() as u16).unwrap(),
            interned: arena.interner.get_or_intern(v),
        }
    }

    pub fn str<'a>(&self, arena: &'a fastn_unresolved::Arena) -> &'a str {
        arena.interner.resolve(self.interned).unwrap()
    }

    pub fn package<'a>(&self, arena: &'a fastn_unresolved::Arena) -> &'a str {
        &self.str(arena)[..self.package_len.get() as usize]
    }

    pub fn module<'a>(&self, arena: &'a fastn_unresolved::Arena) -> &'a str {
        &self.str(arena)[self.package_len.get() as usize + 1..]
    }

    pub fn symbol(
        &self,
        name: &str,
        arena: &mut fastn_unresolved::Arena,
    ) -> fastn_unresolved::Symbol {
        let module_len = {
            let len = arena.interner.resolve(self.interned).unwrap().len() as u16
                - self.package_len.get();
            if len > 0 {
                Some(std::num::NonZeroU16::new(len).unwrap())
            } else {
                None
            }
        };
        let v = if module_len.is_none() {
            format!("{}#{name}", self.package(arena))
        } else {
            format!("{}/{}#{name}", self.package(arena), self.module(arena))
        };
        fastn_unresolved::Symbol {
            package_len: self.package_len,
            module_len,
            interned: arena.interner.get_or_intern(v),
        }
    }
}

impl fastn_unresolved::Arena {
    pub fn new_aliases(&mut self) -> fastn_unresolved::AliasesID {
        self.aliases.alloc(fastn_unresolved::Aliases::default())
    }
}
