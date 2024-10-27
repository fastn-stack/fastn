impl<T> fastn_p1::Spanned<T> {
    pub fn map<T2, F: FnOnce(T) -> T2>(self, f: F) -> fastn_p1::Spanned<T2> {
        fastn_p1::Spanned {
            span: self.span,
            value: f(self.value),
        }
    }
}

impl From<fastn_p1::Span> for fastn_p1::Identifier {
    fn from(value: fastn_p1::Span) -> Self {
        fastn_p1::Identifier { name: value }
    }
}

impl From<fastn_p1::Span> for fastn_p1::PackageName {
    fn from(value: fastn_p1::Span) -> Self {
        fastn_p1::PackageName { name: value }
    }
}

impl From<fastn_p1::Identifier> for fastn_p1::QualifiedIdentifier {
    fn from(value: fastn_p1::Identifier) -> Self {
        fastn_p1::QualifiedIdentifier {
            module: None,
            terms: vec![value],
        }
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

impl fastn_p1::Kind {
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
}
