impl fastn_section::JDebug for fastn_unresolved::Import {
    fn debug(&self, source: &str) -> serde_json::Value {
        let mut o = serde_json::Map::new();

        let name = if self.package.0.is_empty() {
            self.module.0.to_string()
        } else {
            format!("{}/{}", self.package.0, self.module.0)
        };

        o.insert(
            "import".into(),
            match self.alias {
                Some(ref v) => format!("{name}=>{}", v.0),
                None => name,
            }
            .into(),
        );

        if let Some(ref v) = self.export {
            o.insert("exports".into(), v.debug(source));
        }

        if let Some(ref v) = self.exposing {
            o.insert("exposing".into(), v.debug(source));
        }
        serde_json::Value::Object(o)
    }
}

impl fastn_section::JDebug for fastn_unresolved::Export {
    fn debug(&self, source: &str) -> serde_json::Value {
        match self {
            fastn_unresolved::Export::All => "all".into(),
            fastn_unresolved::Export::Things(v) => {
                serde_json::Value::Array(v.iter().map(|v| v.debug(source)).collect())
            }
        }
    }
}

impl fastn_section::JDebug for fastn_unresolved::AliasableIdentifier {
    fn debug(&self, _source: &str) -> serde_json::Value {
        match self.alias {
            Some(ref v) => format!("{}=>{}", self.name.0, v.0),
            None => self.name.0.to_string(),
        }
        .into()
    }
}
