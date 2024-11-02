use crate::{Error, Span};

impl<T> fastn_lang::Spanned<T> {
    pub fn map<T2, F: FnOnce(T) -> T2>(self, f: F) -> fastn_lang::Spanned<T2> {
        fastn_lang::Spanned {
            span: self.span,
            value: f(self.value),
        }
    }
}

impl From<fastn_lang::Span> for fastn_lang::section::Identifier {
    fn from(value: fastn_lang::Span) -> Self {
        fastn_lang::section::Identifier { name: value }
    }
}

impl From<fastn_lang::Span> for fastn_lang::section::AliasableIdentifier {
    fn from(value: fastn_lang::Span) -> Self {
        fastn_lang::section::AliasableIdentifier {
            name: value,
            alias: None,
        }
    }
}

impl From<fastn_lang::section::Identifier> for fastn_lang::section::AliasableIdentifier {
    fn from(value: fastn_lang::section::Identifier) -> Self {
        fastn_lang::section::AliasableIdentifier {
            name: value.name,
            alias: None,
        }
    }
}

impl From<fastn_lang::section::Identifier> for fastn_lang::section::QualifiedIdentifier {
    fn from(value: fastn_lang::section::Identifier) -> Self {
        fastn_lang::section::QualifiedIdentifier {
            module: None,
            terms: vec![value],
        }
    }
}

impl From<fastn_lang::section::Kind> for Option<fastn_lang::section::KindedName> {
    fn from(value: fastn_lang::section::Kind) -> Self {
        Some(fastn_lang::section::KindedName {
            kind: None,
            name: value.to_identifier()?,
        })
    }
}

pub fn extend_span(span: &mut Option<fastn_lang::Span>, other: fastn_lang::Span) {
    if let Some(s) = span {
        s.extend(other);
    } else {
        *span = Some(other);
    }
}

pub fn extend_o_span(span: &mut Option<fastn_lang::Span>, other: Option<fastn_lang::Span>) {
    if let Some(other) = other {
        extend_span(span, other);
    }
}

pub fn extend_spanned<T>(span: &mut Option<fastn_lang::Span>, other: &fastn_lang::Spanned<T>) {
    extend_span(span, other.span.clone());
}

impl fastn_lang::section::Kind {
    fn span(&self) -> fastn_lang::Span {
        let mut span = self.doc.clone();
        extend_spanned(&mut span, &self.visibility);

        span.unwrap()
    }
}

impl fastn_lang::Section {}

impl fastn_lang::section::Kind {
    pub fn attach_doc(&mut self, doc: fastn_lang::Span) {
        if self.doc.is_some() {
            panic!("doc already attached");
        }
        self.doc = Some(doc);
    }

    pub fn attach_visibility(&mut self, visibility: fastn_lang::Spanned<fastn_lang::Visibility>) {
        if self.visibility.is_some() {
            panic!("visibility already attached");
        }
        self.visibility = Some(visibility);
    }

    pub fn to_identifier(&self) -> Option<fastn_lang::section::Identifier> {
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

impl From<fastn_lang::section::QualifiedIdentifier> for fastn_lang::section::Kind {
    fn from(name: fastn_lang::section::QualifiedIdentifier) -> Self {
        fastn_lang::section::Kind {
            name,
            ..Default::default()
        }
    }
}

impl fastn_lang::section::QualifiedIdentifier {
    pub fn new(
        module: Option<fastn_lang::section::ModuleName>,
        terms: Vec<fastn_lang::section::Identifier>,
    ) -> Self {
        assert!(module.is_some() || !terms.is_empty());
        fastn_lang::section::QualifiedIdentifier { module, terms }
    }
}

impl fastn_lang::Section {
    pub fn with_name(
        name: fastn_lang::Span,
        function_marker: Option<fastn_lang::Span>,
    ) -> Box<fastn_lang::Section> {
        Box::new(fastn_lang::Section {
            init: fastn_lang::section::SectionInit {
                name: fastn_lang::section::KindedName {
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

impl fastn_lang::Scannable for fastn_lang::section::Document {
    fn add_error(&mut self, _span: Span, _message: Error) {
        todo!()
    }

    fn add_comment(&mut self, _span: Span) {
        todo!()
    }
}
