impl<T> fastn_section::Spanned<T> {
    pub fn map<T2, F: FnOnce(T) -> T2>(self, f: F) -> fastn_section::Spanned<T2> {
        fastn_section::Spanned {
            span: self.span,
            value: f(self.value),
        }
    }
}

impl fastn_section::Span {
    pub fn wrap<T>(&self, value: T) -> fastn_section::Spanned<T> {
        fastn_section::Spanned {
            span: self.clone(),
            value,
        }
    }
}

impl From<fastn_section::Span> for fastn_section::Identifier {
    fn from(value: fastn_section::Span) -> Self {
        fastn_section::Identifier { name: value }
    }
}

impl From<fastn_section::Span> for fastn_section::AliasableIdentifier {
    fn from(value: fastn_section::Span) -> Self {
        fastn_section::AliasableIdentifier {
            name: value,
            alias: None,
        }
    }
}

impl From<fastn_section::Identifier> for fastn_section::AliasableIdentifier {
    fn from(value: fastn_section::Identifier) -> Self {
        fastn_section::AliasableIdentifier {
            name: value.name,
            alias: None,
        }
    }
}

impl From<fastn_section::Identifier> for fastn_section::QualifiedIdentifier {
    fn from(value: fastn_section::Identifier) -> Self {
        fastn_section::QualifiedIdentifier {
            module: None,
            terms: vec![value],
        }
    }
}

impl From<fastn_section::Kind> for Option<fastn_section::KindedName> {
    fn from(value: fastn_section::Kind) -> Self {
        Some(fastn_section::KindedName {
            kind: None,
            name: value.to_identifier()?,
        })
    }
}

pub fn extend_span(span: &mut Option<fastn_section::Span>, other: fastn_section::Span) {
    if let Some(_s) = span {
        // s.extend(other);
        todo!()
    } else {
        *span = Some(other);
    }
}

#[allow(dead_code)]
pub fn extend_o_span(span: &mut Option<fastn_section::Span>, other: Option<fastn_section::Span>) {
    if let Some(other) = other {
        extend_span(span, other);
    }
}

#[allow(dead_code)]
pub fn extend_spanned<T>(
    span: &mut Option<fastn_section::Span>,
    other: &fastn_section::Spanned<T>,
) {
    extend_span(span, other.span.clone());
}

impl fastn_section::Kind {
    #[allow(dead_code)]
    pub fn span(&self) -> fastn_section::Span {
        todo!()
        // let mut span = self.doc.clone();
        // extend_spanned(&mut span, &self.visibility);
        //
        // span.unwrap()
    }
}

impl fastn_section::Section {
    pub fn span(&self) -> fastn_section::Span {
        todo!()
        // let mut span = self.init.name.name.name.clone();
        // extend_o_span(&mut span, self.function_marker.clone());
        //
        // span.unwrap()
    }
    pub fn full_name_with_kind<'input>(&self, _source: &'input str) -> &'input str {
        todo!()
    }

    pub fn kind_name<'input>(&self, _source: &'input str) -> &'input str {
        todo!()
    }

    pub fn name<'input>(&self, source: &'input str) -> &'input str {
        &source[self.init.name.name.name.start..self.init.name.name.name.end]
    }

    pub fn caption_as_plain_string<'input>(&self, _source: &'input str) -> Option<&'input str> {
        todo!()
    }
}

impl fastn_section::Header {
    pub fn name<'input>(&self, source: &'input str) -> &'input str {
        &source[self.name.name.name.start..self.name.name.name.end]
    }

    pub fn span(&self) -> fastn_section::Span {
        self.name.name.name.clone()
    }
}

impl fastn_section::Kind {
    pub fn attach_doc(&mut self, doc: fastn_section::Span) {
        if self.doc.is_some() {
            panic!("doc already attached");
        }
        self.doc = Some(doc);
    }

    pub fn attach_visibility(
        &mut self,
        visibility: fastn_section::Spanned<fastn_section::Visibility>,
    ) {
        if self.visibility.is_some() {
            panic!("visibility already attached");
        }
        self.visibility = Some(visibility);
    }

    pub fn to_identifier(&self) -> Option<fastn_section::Identifier> {
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

impl From<fastn_section::QualifiedIdentifier> for fastn_section::Kind {
    fn from(name: fastn_section::QualifiedIdentifier) -> Self {
        fastn_section::Kind {
            name,
            ..Default::default()
        }
    }
}

impl fastn_section::QualifiedIdentifier {
    pub fn new(
        module: Option<fastn_section::ModuleName>,
        terms: Vec<fastn_section::Identifier>,
    ) -> Self {
        assert!(module.is_some() || !terms.is_empty());
        fastn_section::QualifiedIdentifier { module, terms }
    }
}

impl fastn_section::Section {
    pub fn with_name(
        name: fastn_section::Span,
        function_marker: Option<fastn_section::Span>,
    ) -> Box<fastn_section::Section> {
        Box::new(fastn_section::Section {
            init: fastn_section::SectionInit {
                name: fastn_section::KindedName {
                    kind: None,
                    name: name.into(),
                },
                ..Default::default()
            },
            function_marker,
            ..Default::default()
        })
    }
}

impl fastn_section::Scannable for fastn_section::Document {
    fn add_error(&mut self, span: fastn_section::Span, error: fastn_section::Error) {
        self.errors
            .push(fastn_section::Spanned { span, value: error });
    }

    fn add_comment(&mut self, comment: fastn_section::Span) {
        self.comments.push(comment);
    }
}
