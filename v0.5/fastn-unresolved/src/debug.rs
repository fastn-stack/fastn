use serde_json::Value;

impl fastn_jdebug::JDebug for fastn_unresolved::Import {
    fn debug(&self) -> serde_json::Value {
        let mut o = serde_json::Map::new();

        let name = if self.module.package.0.is_empty() {
            self.module.name.0.str().to_string()
        } else {
            format!("{}/{}", self.module.package.0, self.module.name.0.str())
        };

        o.insert(
            "import".into(),
            match self.alias {
                Some(ref v) => format!("{name}=>{}", v.0.str()),
                None => name,
            }
            .into(),
        );

        dbg!(&self);

        if let Some(ref v) = self.export {
            o.insert("export".into(), v.debug());
        }

        if let Some(ref v) = self.exposing {
            o.insert("exposing".into(), v.debug());
        }

        serde_json::Value::Object(o)
    }
}

impl fastn_jdebug::JDebug for fastn_unresolved::Export {
    fn debug(&self) -> serde_json::Value {
        match self {
            fastn_unresolved::Export::All => "all".into(),
            fastn_unresolved::Export::Things(v) => {
                serde_json::Value::Array(v.iter().map(|v| v.debug()).collect())
            }
        }
    }
}

impl fastn_jdebug::JDebug for fastn_unresolved::AliasableIdentifier {
    fn debug(&self) -> serde_json::Value {
        match self.alias {
            Some(ref v) => format!("{}=>{}", self.name.0.str(), v.0.str()),
            None => self.name.0.str().to_string(),
        }
        .into()
    }
}

impl fastn_jdebug::JDebug for fastn_unresolved::ComponentInvocation {
    fn debug(&self) -> Value {
        todo!()
    }
}

impl<U: fastn_jdebug::JDebug, R: fastn_jdebug::JDebug> fastn_jdebug::JDebug
    for fastn_unresolved::UR<U, R>
{
    fn debug(&self) -> Value {
        todo!()
    }
}
