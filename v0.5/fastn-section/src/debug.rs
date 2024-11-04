pub trait JDebug {
    fn debug(&self, source: &str) -> serde_json::Value;
}

fn span(s: &fastn_section::Span, key: &str, source: &str) -> serde_json::Value {
    serde_json::json!({ key: (source[s.start..s.end]).to_string()})
}

impl JDebug for fastn_section::Span {
    fn debug(&self, source: &str) -> serde_json::Value {
        let t = &source[self.start..self.end];
        if t.is_empty() { "<empty>" } else { t }.into()
    }
}

impl<T: JDebug> JDebug for fastn_section::Spanned<T> {
    fn debug(&self, source: &str) -> serde_json::Value {
        self.value.debug(source)
    }
}

impl JDebug for fastn_section::Spanned<()> {
    fn debug(&self, source: &str) -> serde_json::Value {
        span(&self.span, "spanned", source)
    }
}

impl<T: JDebug> JDebug for Vec<T> {
    fn debug(&self, source: &str) -> serde_json::Value {
        serde_json::Value::Array(self.iter().map(|v| v.debug(source)).collect())
    }
}

impl<T: JDebug> JDebug for std::collections::HashMap<fastn_section::Identifier, T> {
    fn debug(&self, source: &str) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        for (k, v) in self {
            o.insert(
                source[k.name.start..k.name.end].to_string(),
                v.debug(source),
            );
        }
        serde_json::Value::Object(o)
    }
}

impl<T: JDebug> JDebug for Option<T> {
    fn debug(&self, source: &str) -> serde_json::Value {
        self.as_ref()
            .map(|v| v.debug(source))
            .unwrap_or(serde_json::Value::Null)
    }
}

impl JDebug for fastn_section::Visibility {
    fn debug(&self, _source: &str) -> serde_json::Value {
        format!("{self:?}").into()
    }
}

impl JDebug for fastn_section::Document {
    fn debug(&self, source: &str) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        if self.module_doc.is_some() {
            // TODO: can we create a map with `&'static str` keys to avoid this to_string()?
            o.insert("module-doc".to_string(), self.module_doc.debug(source));
        }
        if !self.errors.is_empty() {
            o.insert("errors".to_string(), self.errors.debug(source));
        }
        if !self.comments.is_empty() {
            o.insert("comments".to_string(), self.comments.debug(source));
        }

        if !self.sections.is_empty() {
            o.insert("sections".to_string(), self.sections.debug(source));
        }
        if o.is_empty() {
            return "<empty-document>".into();
        }
        serde_json::Value::Object(o)
    }
}

impl JDebug for fastn_section::Section {
    fn debug(&self, source: &str) -> serde_json::Value {
        // todo: add headers etc (only if they are not null)
        let mut o = serde_json::Map::new();
        o.insert("init".to_string(), self.init.debug(source));

        if let Some(c) = &self.caption {
            o.insert("caption".to_string(), c.debug(source));
        }

        serde_json::Value::Object(o)
    }
}

impl JDebug for fastn_section::SectionInit {
    fn debug(&self, source: &str) -> serde_json::Value {
        self.name.debug(source)
    }
}

impl JDebug for fastn_section::KindedName {
    fn debug(&self, source: &str) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        if let Some(kind) = &self.kind {
            o.insert("kind".into(), kind.debug(source));
        }
        o.insert("name".into(), self.name.debug(source));
        serde_json::Value::Object(o)
    }
}

impl JDebug for fastn_section::Kind {
    fn debug(&self, source: &str) -> serde_json::Value {
        if let Some(v) = self.to_identifier() {
            return v.debug(source);
        }

        let mut o = serde_json::Map::new();
        if let Some(doc) = &self.doc {
            o.insert("doc".into(), doc.debug(source));
        }
        if let Some(visibility) = &self.visibility {
            o.insert("visibility".into(), visibility.debug(source));
        }
        o.insert("name".into(), self.name.debug(source));
        if let Some(args) = &self.args {
            o.insert("args".into(), args.debug(source));
        }
        serde_json::Value::Object(o)
    }
}

impl JDebug for fastn_section::QualifiedIdentifier {
    fn debug(&self, source: &str) -> serde_json::Value {
        if self.terms.is_empty() {
            return self.module.debug(source);
        }

        serde_json::json! ({
            "module": self.module.debug(source),
            "terms": self.terms.debug(source),
        })
    }
}

impl JDebug for fastn_section::Tes {
    fn debug(&self, source: &str) -> serde_json::Value {
        match self {
            fastn_section::Tes::Text(e) => e.debug(source),
            fastn_section::Tes::Expression { content, .. } => content.debug(source),
            fastn_section::Tes::Section(e) => e.debug(source),
        }
    }
}

impl JDebug for fastn_section::Identifier {
    fn debug(&self, source: &str) -> serde_json::Value {
        self.name.debug(source)
    }
}

impl JDebug for fastn_section::PackageName {
    fn debug(&self, source: &str) -> serde_json::Value {
        format!(
            "{} as {}",
            &source[self.name.start..self.name.end],
            &source[self.alias.start..self.alias.end],
        )
        .into()
    }
}

impl JDebug for fastn_section::AliasableIdentifier {
    fn debug(&self, source: &str) -> serde_json::Value {
        if self.alias.is_none() {
            return self.name.debug(source);
        }

        serde_json::json! ({
            "name": self.name.debug(source),
            "alias": self.alias.debug(source),
        })
    }
}

impl JDebug for fastn_section::ModuleName {
    fn debug(&self, source: &str) -> serde_json::Value {
        if self.path.is_empty()
            && self.name.alias.is_none()
            && self.name.name == self.package.name
            && self.name.name == self.package.alias
        {
            return self.name.name.debug(source);
        }

        if self.path.is_empty()
            && self.name.name != self.package.name
            && self.name.alias.is_none()
            && self.package.name != self.package.alias
            && self.name.name == self.package.alias
        {
            return self.package.debug(source);
        }

        let mut o = serde_json::Map::new();
        o.insert("package".into(), self.package.debug(source));
        o.insert("name".into(), self.name.debug(source));
        if !self.path.is_empty() {
            o.insert("path".into(), self.path.debug(source));
        }
        serde_json::Value::Object(o)
    }
}

impl JDebug for fastn_section::Error {
    fn debug(&self, source: &str) -> serde_json::Value {
        error(self, &Default::default(), source)
    }
}

fn error(e: &fastn_section::Error, _s: &fastn_section::Span, _source: &str) -> serde_json::Value {
    serde_json::json!({ "error": match e {
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
    }})
}
