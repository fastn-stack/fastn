// fn span(s: &fastn_section::Span, key: &str) -> serde_json::Value {
//     serde_json::json!({ key: ([s.start..s.end]).to_string()})
// }

// impl fastn_section::JDebug for fastn_section::Spanned<()> {
//     fn debug(&self) -> serde_json::Value {
//         span(&self.span, "spanned", )
//     }
// }

impl fastn_section::JDebug for fastn_section::Visibility {
    fn debug(&self) -> serde_json::Value {
        format!("{self:?}").into()
    }
}

impl fastn_section::JDebug for fastn_section::Document {
    fn debug(&self) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        if self.module_doc.is_some() {
            // TODO: can we create a map with `&'static str` keys to avoid this to_string()?
            o.insert("module-doc".to_string(), self.module_doc.debug());
        }
        if !self.errors.is_empty() {
            o.insert("errors".to_string(), self.errors.debug());
        }
        if !self.comments.is_empty() {
            o.insert("comments".to_string(), self.comments.debug());
        }

        if !self.sections.is_empty() {
            o.insert("sections".to_string(), self.sections.debug());
        }
        if o.is_empty() {
            return "<empty-document>".into();
        }
        serde_json::Value::Object(o)
    }
}

impl fastn_section::JDebug for fastn_section::Section {
    fn debug(&self) -> serde_json::Value {
        // todo: add headers etc (only if they are not null)
        let mut o = serde_json::Map::new();
        o.insert("init".to_string(), self.init.debug());

        if let Some(c) = &self.caption {
            o.insert("caption".to_string(), c.0.debug());
        }

        serde_json::Value::Object(o)
    }
}

impl fastn_section::JDebug for fastn_section::SectionInit {
    fn debug(&self) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        if self.function_marker.is_some() {
            o.insert("function".into(), self.name.debug());
        } else {
            o.insert("name".into(), self.name.debug());
        }
        if let Some(v) = &self.visibility {
            o.insert("visibility".into(), v.debug());
        }
        if let Some(v) = &self.kind {
            o.insert("kind".into(), v.debug());
        }
        if let Some(v) = &self.doc {
            o.insert("doc".into(), v.debug());
        }
        serde_json::Value::Object(o)
    }
}

impl fastn_section::JDebug for fastn_section::Kind {
    fn debug(&self) -> serde_json::Value {
        if let Some(v) = self.to_identifier_reference() {
            return v.debug();
        }

        let mut o = serde_json::Map::new();
        o.insert("name".into(), self.name.debug());
        if let Some(args) = &self.args {
            o.insert("args".into(), args.debug());
        }
        serde_json::Value::Object(o)
    }
}

impl fastn_section::JDebug for fastn_section::HeaderValue {
    fn debug(&self) -> serde_json::Value {
        self.0.debug()
    }
}

impl fastn_section::JDebug for fastn_section::Tes {
    fn debug(&self) -> serde_json::Value {
        match self {
            fastn_section::Tes::Text(e) => e.debug(),
            fastn_section::Tes::Expression { content, .. } => content.debug(),
            fastn_section::Tes::Section(e) => e.debug(),
        }
    }
}

impl fastn_section::JDebug for fastn_section::Identifier {
    fn debug(&self) -> serde_json::Value {
        self.name.debug()
    }
}

impl fastn_section::JDebug for fastn_section::IdentifierReference {
    fn debug(&self) -> serde_json::Value {
        self.to_string().into()
    }
}

impl fastn_section::JDebug for fastn_section::Error {
    fn debug(&self) -> serde_json::Value {
        error(self, None)
    }
}

fn error(e: &fastn_section::Error, _s: Option<fastn_section::Span>) -> serde_json::Value {
    let v = match e {
        fastn_section::Error::UnexpectedDocComment => "unexpected_doc_comment",
        fastn_section::Error::UnwantedTextFound => "unwanted_text_found",
        fastn_section::Error::EmptyAngleText => "empty_angle_text",
        fastn_section::Error::ColonNotFound => "colon_not_found",
        fastn_section::Error::DashDashNotFound => "dashdash_not_found",
        fastn_section::Error::KindedNameNotFound => "kinded_name_not_found",
        fastn_section::Error::SectionNameNotFoundForEnd => "section_name_not_found_for_end",
        fastn_section::Error::EndContainsData => "end_contains_data",
        fastn_section::Error::EndWithoutStart => "end_without_start",
        fastn_section::Error::ImportCantHaveType => "import_cant_have_type",
        fastn_section::Error::ImportMustBeImport => "import_must_be_import",
        fastn_section::Error::ImportMustHaveCaption => "import_must_have_caption",
        fastn_section::Error::BodyNotAllowed => "body_not_allowed",
        fastn_section::Error::ExtraArgumentFound => "extra_argument_found",
        fastn_section::Error::ComponentIsNotAFunction => "component_is_not_a_function",
        fastn_section::Error::SymbolNotFound => "symbol_not_found",
        fastn_section::Error::InvalidIdentifier => "invalid_identifier",
        fastn_section::Error::UnexpectedCaption => "unexpected_caption",
        fastn_section::Error::InvalidPackageFile => "invalid_package_file",
        _ => todo!(),
    };

    serde_json::json!({ "error": v})
}

impl fastn_section::JDebug for fastn_section::Span {
    fn debug(&self) -> serde_json::Value {
        if self.inner.is_empty() {
            "<empty>"
        } else {
            self.inner.as_str()
        }
        .into()
    }
}

impl AsRef<arcstr::Substr> for fastn_section::Span {
    fn as_ref(&self) -> &arcstr::Substr {
        &self.inner
    }
}

impl<T: fastn_section::JDebug> fastn_section::JDebug for Vec<T> {
    fn debug(&self) -> serde_json::Value {
        serde_json::Value::Array(self.iter().map(|v| v.debug()).collect())
    }
}

impl<T: fastn_section::JDebug> fastn_section::JDebug for Option<T> {
    fn debug(&self) -> serde_json::Value {
        self.as_ref()
            .map(|v| v.debug())
            .unwrap_or(serde_json::Value::Null)
    }
}

impl<K: AsRef<fastn_section::Span> + std::fmt::Debug, V: fastn_section::JDebug>
    fastn_section::JDebug for std::collections::HashMap<K, V>
{
    fn debug(&self) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        for (k, v) in self {
            let r = k.as_ref();
            o.insert(r.inner.to_string(), v.debug());
        }
        serde_json::Value::Object(o)
    }
}

impl fastn_section::Span {
    pub fn inner_str(&self, s: &str) -> fastn_section::Span {
        fastn_section::Span {
            inner: self.inner.substr_from(s),
            module: self.module,
        }
    }

    pub fn wrap<T>(&self, value: T) -> fastn_section::Spanned<T> {
        fastn_section::Spanned {
            span: self.clone(),
            value,
        }
    }

    pub fn span(&self, start: usize, end: usize) -> fastn_section::Span {
        fastn_section::Span {
            inner: self.inner.substr(start..end),
            module: self.module,
        }
    }

    pub fn start(&self) -> usize {
        self.inner.range().start
    }

    pub fn end(&self) -> usize {
        self.inner.range().end
    }

    pub fn str(&self) -> &str {
        &self.inner
    }
}

impl<T> fastn_section::Spanned<T> {
    pub fn map<T2, F: FnOnce(T) -> T2>(self, f: F) -> fastn_section::Spanned<T2> {
        fastn_section::Spanned {
            span: self.span,
            value: f(self.value),
        }
    }
}

impl<T: fastn_section::JDebug> fastn_section::JDebug for fastn_section::Spanned<T> {
    fn debug(&self) -> serde_json::Value {
        self.value.debug()
    }
}

impl fastn_section::JDebug for () {
    fn debug(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
}
