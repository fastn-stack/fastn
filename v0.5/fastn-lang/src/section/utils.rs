use crate::{Error, Span};

impl<T> fastn_lang::Spanned<T> {
    pub fn map<T2, F: FnOnce(T) -> T2>(self, f: F) -> fastn_lang::Spanned<T2> {
        fastn_lang::Spanned {
            span: self.span,
            value: f(self.value),
        }
    }
}

impl From<fastn_lang::Span> for fastn_lang::Identifier {
    fn from(value: fastn_lang::Span) -> Self {
        fastn_lang::Identifier { name: value }
    }
}

impl From<fastn_lang::Span> for fastn_lang::AliasableIdentifier {
    fn from(value: fastn_lang::Span) -> Self {
        fastn_lang::AliasableIdentifier {
            name: value,
            alias: None,
        }
    }
}

impl From<fastn_lang::Identifier> for fastn_lang::AliasableIdentifier {
    fn from(value: fastn_lang::Identifier) -> Self {
        fastn_lang::AliasableIdentifier {
            name: value.name,
            alias: None,
        }
    }
}

impl From<fastn_lang::Identifier> for fastn_lang::QualifiedIdentifier {
    fn from(value: fastn_lang::Identifier) -> Self {
        fastn_lang::QualifiedIdentifier {
            module: None,
            terms: vec![value],
        }
    }
}

impl From<fastn_lang::Kind> for Option<fastn_lang::KindedName> {
    fn from(value: fastn_lang::Kind) -> Self {
        Some(fastn_lang::KindedName {
            kind: None,
            name: value.to_identifier()?,
        })
    }
}

impl fastn_lang::ParserEngine {
    pub fn new(doc_name: String) -> Self {
        Self {
            doc_name,
            edits: vec![],
        }
    }

    pub fn add_edit(&mut self, from: usize, to: usize, text: String) -> &fastn_lang::Edit {
        self.edits.push(fastn_lang::Edit {
            from,
            to,
            text: text.chars().collect(),
        });
        self.edits.last().unwrap()
    }
}

impl fastn_lang::Kind {
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

    pub fn to_identifier(&self) -> Option<fastn_lang::Identifier> {
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

impl From<fastn_lang::QualifiedIdentifier> for fastn_lang::Kind {
    fn from(name: fastn_lang::QualifiedIdentifier) -> Self {
        fastn_lang::Kind {
            name,
            ..Default::default()
        }
    }
}

impl fastn_lang::QualifiedIdentifier {
    pub fn new(module: Option<fastn_lang::ModuleName>, terms: Vec<fastn_lang::Identifier>) -> Self {
        assert!(module.is_some() || !terms.is_empty());
        fastn_lang::QualifiedIdentifier { module, terms }
    }
}

impl fastn_lang::Section {
    pub fn with_name(
        name: fastn_lang::Span,
        function_marker: Option<fastn_lang::Span>,
    ) -> Box<fastn_lang::Section> {
        Box::new(fastn_lang::Section {
            init: fastn_lang::SectionInit {
                name: fastn_lang::KindedName {
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
