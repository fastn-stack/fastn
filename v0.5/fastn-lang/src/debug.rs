pub trait JDebug {
    fn debug(&self, source: &str) -> serde_json::Value;
}

fn span(s: &fastn_lang::Span, key: &str, source: &str) -> serde_json::Value {
    serde_json::json!({ key: (source[s.start..s.end]).to_string()})
}

impl JDebug for fastn_lang::Span {
    fn debug(&self, source: &str) -> serde_json::Value {
        let t = &source[self.start..self.end];
        if t.is_empty() { "<empty>" } else { t }.into()
    }
}

impl<T: JDebug> JDebug for fastn_lang::Spanned<T> {
    fn debug(&self, source: &str) -> serde_json::Value {
        self.value.debug(source)
    }
}

impl JDebug for fastn_lang::Spanned<()> {
    fn debug(&self, source: &str) -> serde_json::Value {
        span(&self.span, "spanned", source)
    }
}

impl<T: JDebug> JDebug for Vec<T> {
    fn debug(&self, source: &str) -> serde_json::Value {
        serde_json::Value::Array(self.iter().map(|v| v.debug(source)).collect())
    }
}

impl<T: JDebug> JDebug for std::collections::HashMap<fastn_lang::token::Identifier, T> {
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

impl JDebug for fastn_lang::Visibility {
    fn debug(&self, _source: &str) -> serde_json::Value {
        format!("{self:?}").into()
    }
}

impl JDebug for fastn_lang::parse::Document {
    fn debug(&self, source: &str) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        if self.module_doc.is_some() {
            // TODO: can we create a map with `&'static str` keys to avoid this to_string()?
            o.insert("module-doc".to_string(), self.module_doc.debug(source));
        }
        if !self.content.is_empty() {
            o.insert("content".to_string(), self.content.debug(source));
        }
        if !self.errors.is_empty() {
            o.insert("errors".to_string(), self.errors.debug(source));
        }
        if !self.definitions.is_empty() {
            o.insert("definitions".to_string(), self.definitions.debug(source));
        }
        if !self.comments.is_empty() {
            o.insert("comments".to_string(), self.comments.debug(source));
        }
        if !self.imports.is_empty() {
            o.insert("imports".to_string(), self.imports.debug(source));
        }
        if o.is_empty() {
            return "<empty-document>".into();
        }
        serde_json::Value::Object(o)
    }
}

impl JDebug for fastn_lang::Section {
    fn debug(&self, source: &str) -> serde_json::Value {
        // todo: add headers etc (only if they are not null)
        serde_json::json! ({
            "init": self.init.debug(source),
        })
    }
}

impl JDebug for fastn_lang::token::SectionInit {
    fn debug(&self, source: &str) -> serde_json::Value {
        serde_json::json! ({
            "name": self.name.debug(source)
        })
    }
}

impl JDebug for fastn_lang::token::KindedName {
    fn debug(&self, source: &str) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        if let Some(kind) = &self.kind {
            o.insert("kind".into(), kind.debug(source));
        }
        o.insert("name".into(), self.name.debug(source));
        serde_json::Value::Object(o)
    }
}

impl JDebug for fastn_lang::token::Kind {
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

impl JDebug for fastn_lang::token::QualifiedIdentifier {
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

impl JDebug for fastn_lang::token::Tes {
    fn debug(&self, source: &str) -> serde_json::Value {
        match self {
            fastn_lang::token::Tes::Text(e) => e.debug(source),
            fastn_lang::token::Tes::Expression { content, .. } => content.debug(source),
            fastn_lang::token::Tes::Section(e) => e.debug(source),
        }
    }
}

impl JDebug for fastn_lang::token::Identifier {
    fn debug(&self, source: &str) -> serde_json::Value {
        self.name.debug(source)
    }
}

impl JDebug for fastn_lang::token::PackageName {
    fn debug(&self, source: &str) -> serde_json::Value {
        format!(
            "{} as {}",
            &source[self.name.start..self.name.end],
            &source[self.alias.start..self.alias.end],
        )
        .into()
    }
}

impl JDebug for fastn_lang::token::AliasableIdentifier {
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

impl JDebug for fastn_lang::token::ModuleName {
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

impl JDebug for fastn_lang::parse::Import {
    fn debug(&self, source: &str) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        o.insert("module".into(), self.module.debug(source));
        if self.exports.is_some() {
            o.insert("exports".into(), self.exports.debug(source));
        }
        if self.exposing.is_some() {
            o.insert("exposing".into(), self.exposing.debug(source));
        }
        serde_json::Value::Object(o)
    }
}

impl JDebug for fastn_lang::parse::Export {
    fn debug(&self, source: &str) -> serde_json::Value {
        match self {
            fastn_lang::parse::Export::All => "<all>".into(),
            fastn_lang::parse::Export::Things(t) => t.debug(source),
        }
    }
}

impl JDebug for fastn_lang::parse::Definition {
    fn debug(&self, source: &str) -> serde_json::Value {
        match self {
            fastn_lang::parse::Definition::Component(c) => {
                serde_json::json!({"component": c.debug(source)})
            }
            fastn_lang::parse::Definition::Variable(v) => {
                serde_json::json!({"variable": v.debug(source)})
            }
            fastn_lang::parse::Definition::Function(v) => {
                serde_json::json!({"function": v.debug(source)})
            }
            fastn_lang::parse::Definition::TypeAlias(v) => {
                serde_json::json!({"type-alias": v.debug(source)})
            }
            fastn_lang::parse::Definition::Record(v) => {
                serde_json::json!({"record": v.debug(source)})
            }
            fastn_lang::parse::Definition::OrType(v) => {
                serde_json::json!({"or-type": v.debug(source)})
            }
            fastn_lang::parse::Definition::Module(v) => {
                serde_json::json!({"module": v.debug(source)})
            }
        }
    }
}

impl JDebug for fastn_lang::Error {
    fn debug(&self, source: &str) -> serde_json::Value {
        error(self, &Default::default(), source)
    }
}

fn error(e: &fastn_lang::Error, _s: &fastn_lang::Span, _source: &str) -> serde_json::Value {
    serde_json::json!({ "error": match e {
        fastn_lang::Error::UnexpectedDocComment => "unexpected_doc_comment",
        fastn_lang::Error::UnwantedTextFound => "unwanted_text_found",
        fastn_lang::Error::EmptyAngleText => "empty_angle_text",
        fastn_lang::Error::ColonNotFound => "colon_not_found",
        fastn_lang::Error::DashDashNotFound => "dashdash_not_found",
        fastn_lang::Error::KindedNameNotFound => "kinded_name_not_found",
        fastn_lang::Error::SectionNameNotFoundForEnd => "section_name_not_found_for_end",
        fastn_lang::Error::EndContainsData => "end_contains_data",
        fastn_lang::Error::EndWithoutStart => "end_without_start",
        fastn_lang::Error::ImportCantHaveType => "import_cant_have_type",
    }})
}
