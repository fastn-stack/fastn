use serde_json::Value;

impl fastn_jdebug::JDebug for fastn_unresolved::Import {
    fn debug(&self) -> serde_json::Value {
        let mut o = serde_json::Map::new();

        let name = if self.module.package.str().is_empty() {
            self.module.name.str().to_string()
        } else {
            format!("{}/{}", self.module.package.str(), self.module.name.str())
        };

        o.insert(
            "import".into(),
            match self.alias {
                Some(ref v) => format!("{name}=>{}", v.str()),
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
            Some(ref v) => format!("{}=>{}", self.name.str(), v.str()),
            None => self.name.str().to_string(),
        }
        .into()
    }
}

impl fastn_jdebug::JDebug for fastn_unresolved::ComponentInvocation {
    fn debug(&self) -> Value {
        serde_json::json!({
            "content": self.name.debug(),
            "caption": self.caption.debug(),
        })
    }
}

impl fastn_jdebug::JDebug for fastn_unresolved::Definition {
    fn debug(&self) -> Value {
        todo!()
    }
}

impl<U: fastn_jdebug::JDebug, R: fastn_jdebug::JDebug> fastn_jdebug::JDebug
    for fastn_unresolved::UR<U, R>
{
    fn debug(&self) -> Value {
        match self {
            crate::UR::Resolved(r) => r.debug(),
            crate::UR::UnResolved(u) => u.debug(),
            crate::UR::NotFound => unimplemented!(),
            crate::UR::Invalid(_) => unimplemented!(),
        }
    }
}
