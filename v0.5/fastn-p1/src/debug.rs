use crate::SingleError;

fn span(s: &fastn_p1::Span, key: &str, source: &str) -> serde_json::Value {
    serde_json::json!({ key: (source[s.start..s.end]).to_string()})
}

impl JDebug for fastn_p1::Span {
    fn debug(&self, source: &str) -> serde_json::Value {
        (&source[self.start..self.end]).into()
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

pub trait JDebug {
    fn debug(&self, source: &str) -> serde_json::Value;
}

impl JDebug for fastn_p1::ParseOutput {
    fn debug(&self, source: &str) -> serde_json::Value {
        serde_json::json!({
            "module_doc": self.module_doc.debug(source),
            "items": self.items.debug(source),
            // ignoring line_starts for now
        })
    }
}

impl JDebug for fastn_p1::Section {
    fn debug(&self, source: &str) -> serde_json::Value {
        serde_json::json! ({
            "name": self.name.debug(source)
        })
    }
}

impl JDebug for fastn_p1::KindedName {
    fn debug(&self, source: &str) -> serde_json::Value {
        serde_json::json! ({
            "kind": self.kind.debug(source),
            "name": self.name.debug(source),
        })
    }
}

impl JDebug for fastn_p1::Kind {
    fn debug(&self, source: &str) -> serde_json::Value {
        serde_json::json! ({
            "doc": self.doc.debug(source),
            "visibility": self.visibility.debug(source),
            "kind": self.name.debug(source),
        })
    }
}
impl JDebug for fastn_p1::QualifiedIdentifier {
    fn debug(&self, source: &str) -> serde_json::Value {
        serde_json::json! ({
            "module": self.module.debug(source),
            "terms": self.terms.debug(source),
        })
    }
}

impl JDebug for fastn_p1::ModuleName {
    fn debug(&self, source: &str) -> serde_json::Value {
        serde_json::json! ({
            "package": self.package.debug(source),
            "path": self.path.debug(source),
        })
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
        SingleError::UnwantedTextFound => "unwanted_text_found",
        SingleError::EmptyAngleText => "empty_angle_text",
        SingleError::ColonNotFound => "colon_not_found",
        SingleError::DashDashNotFound => "dashdash_not_found",
        SingleError::KindedNameNotFound => "kinded_name_not_found",
    }})
}
