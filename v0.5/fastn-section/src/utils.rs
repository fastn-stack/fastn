impl From<fastn_jdebug::Span> for fastn_section::Identifier {
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
    pub fn full_name_with_kind(&self) -> &fastn_section::Span {
        todo!()
    }

    pub fn kind_name(&self) -> Option<&str> {
        todo!()
    }

    pub fn name(&self) -> &str {
        self.init.name.name.name.str()
    }

    pub fn name_span(&self) -> &fastn_section::Span {
        &self.init.name.name.name
    }

    pub fn caption_as_plain_span(&self) -> Option<&fastn_section::Span> {
        self.caption.as_ref().and_then(|c| c.as_plain_span())
    }

    pub fn header_as_plain_span(&self, name: &str) -> Option<&fastn_section::Span> {
        self.headers
            .iter()
            .find(|h| h.name() == name)
            .and_then(|h| h.value.as_plain_span())
    }
}

impl fastn_section::HeaderValue {
    pub fn as_plain_string(&self) -> Option<&str> {
        self.as_plain_span().map(fastn_section::Span::str)
    }

    pub fn as_plain_span(&self) -> Option<&fastn_section::Span> {
        if self.0.len() != 1 {
            return None;
        }

        match self.0.get(0) {
            Some(fastn_section::Tes::Text(s)) => Some(s),
            _ => None,
        }
    }
}

impl fastn_section::Header {
    pub fn name(&self) -> &str {
        self.name.name.name.str()
    }

    pub fn name_span(&self) -> &fastn_section::Span {
        &self.name.name.name
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

impl fastn_section::Identifier {
    pub fn str(&self) -> &str {
        self.name.str()
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
                dashdash: Default::default(),
                name: fastn_section::KindedName {
                    kind: None,
                    name: name.into(),
                },
                colon: Default::default(),
                function_marker,
            },
            caption: None,
            headers: vec![],
            body: None,
            children: vec![],
            is_commented: false,
            has_end: false,
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
