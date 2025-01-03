impl From<fastn_section::Span> for fastn_section::Identifier {
    fn from(value: fastn_section::Span) -> Self {
        fastn_section::Identifier { name: value }
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

    pub fn simple_section_kind_name(&self) -> Option<&str> {
        let kind = match self.init.kind {
            Some(ref k) => k,
            None => return None,
        };

        // the reason doc must be none as this is for section, and section doc is not stored in
        // kind.doc.
        if kind.args.is_some()
        // || kind.name.module.is_some()
        // || kind.name.terms.len() != 1
        {
            return None;
        }

        match kind.name {
            fastn_section::IdentifierReference::Local(ref kind) => Some(kind.str()),
            _ => None,
        }
    }

    pub fn simple_name(&self) -> Option<&str> {
        match self.init.name {
            fastn_section::IdentifierReference::Local(ref name) => Some(name.str()),
            _ => None,
        }
    }

    pub fn simple_name_span(&self) -> &fastn_section::Span {
        match self.init.name {
            fastn_section::IdentifierReference::Local(ref name) => name,
            _ => panic!("not a local name"),
        }
    }

    pub fn caption_as_plain_span(&self) -> Option<&fastn_section::Span> {
        self.caption.as_ref().and_then(|c| c.as_plain_span())
    }

    pub fn simple_caption(&self) -> Option<&str> {
        self.caption.as_ref().and_then(|c| c.as_plain_string())
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

    pub fn name(&self) -> &str {
        self.name.name.str()
    }

    pub fn simple_value(&self) -> Option<&str> {
        todo!()
    }

    pub fn name_span(&self) -> &fastn_section::Span {
        &self.name.name
    }
}

impl fastn_section::Kind {
    pub fn to_identifier_reference(&self) -> Option<fastn_section::IdentifierReference> {
        if self.args.is_some() {
            return None;
        }

        Some(self.name.clone())
    }
}

impl From<fastn_section::IdentifierReference> for fastn_section::Kind {
    fn from(name: fastn_section::IdentifierReference) -> Self {
        fastn_section::Kind { name, args: None }
    }
}

impl fastn_section::Identifier {
    pub fn str(&self) -> &str {
        self.name.str()
    }

    pub fn spanned(&self, e: fastn_section::Error) -> fastn_section::Spanned<fastn_section::Error> {
        fastn_section::Spanned {
            span: self.name.clone(),
            value: e,
        }
    }
}

impl fastn_section::IdentifierReference {
    pub fn wrap<T>(&self, value: T) -> fastn_section::Spanned<T> {
        let span = match self {
            fastn_section::IdentifierReference::Local(ref name) => name.clone(),
            // TODO: this is wrong, we should coalesce the spans.
            fastn_section::IdentifierReference::Absolute { package, .. } => package.clone(),
            // TODO: this is wrong, we should coalesce the spans.
            fastn_section::IdentifierReference::Imported { module, .. } => module.clone(),
        };

        fastn_section::Spanned { span, value }
    }
}

impl From<fastn_section::Span> for fastn_section::IdentifierReference {
    fn from(name: fastn_section::Span) -> Self {
        fastn_section::IdentifierReference::Local(name)
    }
}

impl std::fmt::Display for fastn_section::IdentifierReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            fastn_section::IdentifierReference::Local(ref name) => name.str().to_string(),
            fastn_section::IdentifierReference::Absolute {
                package,
                module,
                name,
            } => match module {
                Some(module) => format!("{}/{}#{}", package.str(), module.str(), name.str()),
                None => format!("{}#{}", package.str(), name.str()),
            },
            fastn_section::IdentifierReference::Imported { module, name } => {
                format!("{}.{}", module.str(), name.str())
            }
        };
        write!(f, "{}", str)
    }
}

// impl fastn_section::QualifiedIdentifier {
//     pub fn new(
//         module: Option<fastn_section::ModuleName>,
//         terms: Vec<fastn_section::Identifier>,
//     ) -> Self {
//         assert!(module.is_some() || !terms.is_empty());
//         fastn_section::QualifiedIdentifier { module, terms }
//     }
// }

impl fastn_section::Section {
    pub fn with_name(
        name: fastn_section::Span,
        function_marker: Option<fastn_section::Span>,
    ) -> Box<fastn_section::Section> {
        Box::new(fastn_section::Section {
            init: fastn_section::SectionInit {
                dashdash: Default::default(),
                kind: None,
                doc: None,
                name: name.into(),
                colon: Default::default(),
                function_marker,
                visibility: None,
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

impl fastn_section::ECey for fastn_section::Document {
    fn add_error(&mut self, span: fastn_section::Span, error: fastn_section::Error) {
        self.errors
            .push(fastn_section::Spanned { span, value: error });
    }

    fn add_comment(&mut self, comment: fastn_section::Span) {
        self.comments.push(comment);
    }
}

impl fastn_section::Document {
    pub fn diagnostics(self) -> Vec<fastn_section::Spanned<fastn_section::Diagnostic>> {
        let mut o: Vec<_> = self
            .errors
            .into_iter()
            .map(|v| v.map(fastn_section::Diagnostic::Error))
            .collect();

        o.extend(
            self.warnings
                .into_iter()
                .map(|v| v.map(fastn_section::Diagnostic::Warning)),
        );

        o
    }
}
