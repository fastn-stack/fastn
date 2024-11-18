// fn span(s: &fastn_section::Span, key: &str, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
//     serde_json::json!({ key: (interner[s.start..s.end]).to_string()})
// }

impl<T: fastn_jdebug::JDebug> fastn_jdebug::JDebug for fastn_section::Spanned<T> {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        self.value.debug(interner)
    }
}

// impl fastn_jdebug::JDebug for fastn_section::Spanned<()> {
//     fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
//         span(&self.span, "spanned", interner)
//     }
// }

impl fastn_jdebug::JDebug for fastn_section::Visibility {
    fn debug(&self, _interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        format!("{self:?}").into()
    }
}

impl fastn_jdebug::JDebug for fastn_section::Document {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        if self.module_doc.is_some() {
            // TODO: can we create a map with `&'static str` keys to avoid this to_string()?
            o.insert("module-doc".to_string(), self.module_doc.debug(interner));
        }
        if !self.errors.is_empty() {
            o.insert("errors".to_string(), self.errors.debug(interner));
        }
        if !self.comments.is_empty() {
            o.insert("comments".to_string(), self.comments.debug(interner));
        }

        if !self.sections.is_empty() {
            o.insert("sections".to_string(), self.sections.debug(interner));
        }
        if o.is_empty() {
            return "<empty-document>".into();
        }
        serde_json::Value::Object(o)
    }
}

impl fastn_jdebug::JDebug for fastn_section::Section {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        // todo: add headers etc (only if they are not null)
        let mut o = serde_json::Map::new();
        o.insert("init".to_string(), self.init.debug(interner));

        if let Some(c) = &self.caption {
            o.insert("caption".to_string(), c.0.debug(interner));
        }

        serde_json::Value::Object(o)
    }
}

impl fastn_jdebug::JDebug for fastn_section::SectionInit {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        self.name.debug(interner)
    }
}

impl fastn_jdebug::JDebug for fastn_section::KindedName {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        if let Some(kind) = &self.kind {
            o.insert("kind".into(), kind.debug(interner));
        }
        o.insert("name".into(), self.name.debug(interner));
        serde_json::Value::Object(o)
    }
}

impl fastn_jdebug::JDebug for fastn_section::Kind {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        if let Some(v) = self.to_identifier() {
            return v.debug(interner);
        }

        let mut o = serde_json::Map::new();
        if let Some(doc) = &self.doc {
            o.insert("doc".into(), doc.debug(interner));
        }
        if let Some(visibility) = &self.visibility {
            o.insert("visibility".into(), visibility.debug(interner));
        }
        o.insert("name".into(), self.name.debug(interner));
        if let Some(args) = &self.args {
            o.insert("args".into(), args.debug(interner));
        }
        serde_json::Value::Object(o)
    }
}

impl fastn_jdebug::JDebug for fastn_section::QualifiedIdentifier {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        if self.terms.is_empty() {
            return self.module.debug(interner);
        }

        serde_json::json! ({
            "module": self.module.debug(interner),
            "terms": self.terms.debug(interner),
        })
    }
}

impl fastn_jdebug::JDebug for fastn_section::HeaderValue {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        self.0.debug(interner)
    }
}

impl fastn_jdebug::JDebug for fastn_section::Tes {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        match self {
            fastn_section::Tes::Text(e) => e.debug(interner),
            fastn_section::Tes::Expression { content, .. } => content.debug(interner),
            fastn_section::Tes::Section(e) => e.debug(interner),
        }
    }
}

impl fastn_jdebug::JDebug for fastn_section::Identifier {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        self.name.debug(interner)
    }
}

impl fastn_jdebug::JDebug for fastn_section::PackageName {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        format!(
            "{} as {}",
            &interner[self.name.start..self.name.end],
            &interner[self.alias.start..self.alias.end],
        )
        .into()
    }
}

impl fastn_jdebug::JDebug for fastn_section::AliasableIdentifier {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        if self.alias.is_none() {
            return self.name.debug(interner);
        }

        serde_json::json! ({
            "name": self.name.debug(interner),
            "alias": self.alias.debug(interner),
        })
    }
}

impl fastn_jdebug::JDebug for fastn_section::ModuleName {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        if self.path.is_empty()
            && self.name.alias.is_none()
            && self.name.name == self.package.name
            && self.name.name == self.package.alias
        {
            return self.name.name.debug(interner);
        }

        if self.path.is_empty()
            && self.name.name != self.package.name
            && self.name.alias.is_none()
            && self.package.name != self.package.alias
            && self.name.name == self.package.alias
        {
            return self.package.debug(interner);
        }

        let mut o = serde_json::Map::new();
        o.insert("package".into(), self.package.debug(interner));
        o.insert("name".into(), self.name.debug(interner));
        if !self.path.is_empty() {
            o.insert("path".into(), self.path.debug(interner));
        }
        serde_json::Value::Object(o)
    }
}

impl fastn_jdebug::JDebug for fastn_section::Error {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        error(self, None, interner)
    }
}

fn error(
    e: &fastn_section::Error,
    _s: Option<fastn_section::Span>,
    _interner: &string_interner::DefaultStringInterner,
) -> serde_json::Value {
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
    };

    serde_json::json!({ "error": v})
}
