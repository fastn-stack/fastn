pub trait JDebug {
    fn debug(&self, source: &str) -> serde_json::Value;
}

fn span(s: &fastn_p1::Span, key: &str, source: &str) -> serde_json::Value {
    serde_json::json!({ key: (source[s.start..s.end]).to_string()})
}

impl JDebug for fastn_p1::Span {
    fn debug(&self, source: &str) -> serde_json::Value {
        let t = &source[self.start..self.end];
        if t.is_empty() { "<empty>" } else { t }.into()
    }
}

impl<T: JDebug> JDebug for fastn_p1::Spanned<T> {
    fn debug(&self, source: &str) -> serde_json::Value {
        self.value.debug(source)
    }
}

impl JDebug for fastn_p1::Spanned<()> {
    fn debug(&self, source: &str) -> serde_json::Value {
        span(&self.span, "spanned", source)
    }
}

impl<T: JDebug> JDebug for Vec<T> {
    fn debug(&self, source: &str) -> serde_json::Value {
        serde_json::Value::Array(self.iter().map(|v| v.debug(source)).collect())
    }
}

impl<T: JDebug> JDebug for Option<T> {
    fn debug(&self, source: &str) -> serde_json::Value {
        self.as_ref()
            .map(|v| v.debug(source))
            .unwrap_or(serde_json::Value::Null)
    }
}

impl JDebug for fastn_p1::Visibility {
    fn debug(&self, _source: &str) -> serde_json::Value {
        format!("{self:?}").into()
    }
}

impl JDebug for fastn_p1::ParseOutput {
    fn debug(&self, source: &str) -> serde_json::Value {
        if self.items.is_empty() {
            return serde_json::json!({
                "module_doc": self.module_doc.debug(source),
            });
        }

        serde_json::json!({
            "module_doc": self.module_doc.debug(source),
            "items": self.items.debug(source),
            // ignoring line_starts for now
        })
    }
}

impl JDebug for fastn_p1::Section {
    fn debug(&self, source: &str) -> serde_json::Value {
        // todo: add headers etc (only if they are not null)
        serde_json::json! ({
            "init": self.init.debug(source),
        })
    }
}

impl JDebug for fastn_p1::SectionInit {
    fn debug(&self, source: &str) -> serde_json::Value {
        serde_json::json! ({
            "name": self.name.debug(source)
        })
    }
}

impl JDebug for fastn_p1::KindedName {
    fn debug(&self, source: &str) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        if let Some(kind) = &self.kind {
            o.insert("kind".into(), kind.debug(source));
        }
        o.insert("name".into(), self.name.debug(source));
        serde_json::Value::Object(o)
    }
}

impl JDebug for fastn_p1::Kind {
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

impl JDebug for fastn_p1::QualifiedIdentifier {
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

impl JDebug for fastn_p1::SES {
    fn debug(&self, source: &str) -> serde_json::Value {
        match self {
            fastn_p1::SES::String(e) => e.debug(source),
            fastn_p1::SES::Expression { content, .. } => content.debug(source),
            fastn_p1::SES::Section(e) => e.debug(source),
        }
    }
}

impl JDebug for fastn_p1::Identifier {
    fn debug(&self, source: &str) -> serde_json::Value {
        self.name.debug(source)
    }
}

impl JDebug for fastn_p1::PackageName {
    fn debug(&self, source: &str) -> serde_json::Value {
        serde_json::json! ({
            "alias": self.alias.debug(source),
            "name": self.name.debug(source),
        })
    }
}

impl JDebug for fastn_p1::ModuleName {
    fn debug(&self, source: &str) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        o.insert("package".into(), self.package.debug(source));
        o.insert("name".into(), self.name.debug(source));
        if !self.path.is_empty() {
            o.insert("path".into(), self.path.debug(source));
        }
        serde_json::Value::Object(o)
    }
}

impl JDebug for fastn_p1::Spanned<fastn_p1::Item> {
    fn debug(&self, source: &str) -> serde_json::Value {
        match &self.value {
            fastn_p1::Item::Section(section) => section.debug(source),
            fastn_p1::Item::Error(e) => error(e, &self.span, source),
            fastn_p1::Item::Comment => span(&self.span, "comment", source),
        }
    }
}

fn error(e: &fastn_p1::SingleError, _s: &fastn_p1::Span, _source: &str) -> serde_json::Value {
    serde_json::json!({ "error": match e {
        fastn_p1::SingleError::UnexpectedDocComment => "unexpected_doc_comment",
        fastn_p1::SingleError::UnwantedTextFound => "unwanted_text_found",
        fastn_p1::SingleError::EmptyAngleText => "empty_angle_text",
        fastn_p1::SingleError::ColonNotFound => "colon_not_found",
        fastn_p1::SingleError::DashDashNotFound => "dashdash_not_found",
        fastn_p1::SingleError::KindedNameNotFound => "kinded_name_not_found",
        fastn_p1::SingleError::SectionNameNotFoundForEnd => "section_name_not_found_for_end",
        fastn_p1::SingleError::EndContainsData => "end_contains_data",
        fastn_p1::SingleError::EndWithoutStart => "end_without_start",
    }})
}
