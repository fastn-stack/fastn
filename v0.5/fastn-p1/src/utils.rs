impl<'input, T> fastn_p1::Spanned<'input, T> {
    pub fn map<T2, F: FnOnce(T) -> T2>(self, f: F) -> fastn_p1::Spanned<'input, T2> {
        fastn_p1::Spanned {
            span: self.span,
            value: f(self.value),
        }
    }
}

impl From<fastn_p1::Span<'_>> for fastn_p1::Identifier<'_> {
    fn from(value: fastn_p1::Span) -> Self {
        fastn_p1::Identifier { name: value }
    }
}

impl From<fastn_p1::Span<'_>> for fastn_p1::PackageName<'_> {
    fn from(value: fastn_p1::Span) -> Self {
        fastn_p1::PackageName { name: value }
    }
}

impl From<fastn_p1::Identifier<'_>> for fastn_p1::QualifiedIdentifier<'_> {
    fn from(value: fastn_p1::Identifier) -> Self {
        fastn_p1::QualifiedIdentifier {
            module: None,
            terms: vec![value],
        }
    }
}

impl<'input> From<fastn_p1::Kind<'input>> for Option<fastn_p1::KindedName<'input>> {
    fn from(value: fastn_p1::Kind) -> Self {
        Some(fastn_p1::KindedName {
            kind: None,
            name: value.to_identifier()?,
        })
    }
}

impl fastn_p1::ParserEngine {
    pub fn new(doc_name: String) -> Self {
        Self {
            doc_name,
            edits: vec![],
        }
    }

    pub fn add_edit(&mut self, from: usize, to: usize, text: String) -> &fastn_p1::Edit {
        self.edits.push(fastn_p1::Edit {
            from,
            to,
            text: text.chars().collect(),
        });
        self.edits.last().unwrap()
    }
}

impl fastn_p1::Kind<'_> {
    pub fn attach_doc(&mut self, doc: fastn_p1::Span) {
        if self.doc.is_some() {
            panic!("doc already attached");
        }
        self.doc = Some(doc);
    }

    pub fn attach_visibility(&mut self, visibility: fastn_p1::Spanned<fastn_p1::Visibility>) {
        if self.visibility.is_some() {
            panic!("visibility already attached");
        }
        self.visibility = Some(visibility);
    }

    pub fn to_identifier(&self) -> Option<fastn_p1::Identifier> {
        if self.args.is_some()
            || self.doc.is_some()
            || self.visibility.is_some()
            || !self.name.terms.is_empty()
            || self.name.module.is_none()
            || !self.name.terms.is_empty()
        {
            return None;
        }

        let m = self.name.module.as_ref()?;
        if !m.path.is_empty() {
            return None;
        }

        Some(m.package.name.clone().into())
    }
}

impl<'input> From<fastn_p1::QualifiedIdentifier<'input>> for fastn_p1::Kind<'input> {
    fn from(name: fastn_p1::QualifiedIdentifier) -> Self {
        fastn_p1::Kind {
            name,
            ..Default::default()
        }
    }
}
