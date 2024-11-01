use crate::{Error, Span};

impl<T> fastn_parser::Spanned<T> {
    pub fn map<T2, F: FnOnce(T) -> T2>(self, f: F) -> fastn_parser::Spanned<T2> {
        fastn_parser::Spanned {
            span: self.span,
            value: f(self.value),
        }
    }
}

impl From<fastn_parser::Span> for fastn_parser::Identifier {
    fn from(value: fastn_parser::Span) -> Self {
        fastn_parser::Identifier { name: value }
    }
}

impl From<fastn_parser::Span> for fastn_parser::AliasableIdentifier {
    fn from(value: fastn_parser::Span) -> Self {
        fastn_parser::AliasableIdentifier {
            name: value,
            alias: None,
        }
    }
}

impl From<fastn_parser::Identifier> for fastn_parser::AliasableIdentifier {
    fn from(value: fastn_parser::Identifier) -> Self {
        fastn_parser::AliasableIdentifier {
            name: value.name,
            alias: None,
        }
    }
}

impl From<fastn_parser::Identifier> for fastn_parser::QualifiedIdentifier {
    fn from(value: fastn_parser::Identifier) -> Self {
        fastn_parser::QualifiedIdentifier {
            module: None,
            terms: vec![value],
        }
    }
}

impl From<fastn_parser::Kind> for Option<fastn_parser::KindedName> {
    fn from(value: fastn_parser::Kind) -> Self {
        Some(fastn_parser::KindedName {
            kind: None,
            name: value.to_identifier()?,
        })
    }
}

impl fastn_parser::ParserEngine {
    pub fn new(doc_name: String) -> Self {
        Self {
            doc_name,
            edits: vec![],
        }
    }

    pub fn add_edit(&mut self, from: usize, to: usize, text: String) -> &fastn_parser::Edit {
        self.edits.push(fastn_parser::Edit {
            from,
            to,
            text: text.chars().collect(),
        });
        self.edits.last().unwrap()
    }
}

impl fastn_parser::Kind {
    pub fn attach_doc(&mut self, doc: fastn_parser::Span) {
        if self.doc.is_some() {
            panic!("doc already attached");
        }
        self.doc = Some(doc);
    }

    pub fn attach_visibility(
        &mut self,
        visibility: fastn_parser::Spanned<fastn_parser::Visibility>,
    ) {
        if self.visibility.is_some() {
            panic!("visibility already attached");
        }
        self.visibility = Some(visibility);
    }

    pub fn to_identifier(&self) -> Option<fastn_parser::Identifier> {
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

impl From<fastn_parser::QualifiedIdentifier> for fastn_parser::Kind {
    fn from(name: fastn_parser::QualifiedIdentifier) -> Self {
        fastn_parser::Kind {
            name,
            ..Default::default()
        }
    }
}

impl fastn_parser::QualifiedIdentifier {
    pub fn new(
        module: Option<fastn_parser::ModuleName>,
        terms: Vec<fastn_parser::Identifier>,
    ) -> Self {
        assert!(module.is_some() || !terms.is_empty());
        fastn_parser::QualifiedIdentifier { module, terms }
    }
}

impl fastn_parser::Section {
    pub fn with_name(
        name: fastn_parser::Span,
        function_marker: Option<fastn_parser::Span>,
    ) -> Box<fastn_parser::Section> {
        Box::new(fastn_parser::Section {
            init: fastn_parser::SectionInit {
                name: fastn_parser::KindedName {
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

impl fastn_parser::section::EC for fastn_parser::section::Document {
    fn add_error(&mut self, _span: Span, _message: Error) {
        todo!()
    }

    fn add_comment(&mut self, _span: Span) {
        todo!()
    }
}
