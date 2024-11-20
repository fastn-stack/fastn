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
        serde_json::json!({
            "name": self.name.debug(),
            "visibility": self.visibility.debug(),
            "inner": self.inner.debug(),
        })
    }
}

impl fastn_jdebug::JDebug for fastn_unresolved::InnerDefinition {
    fn debug(&self) -> serde_json::Value {
        match self {
            crate::InnerDefinition::Function { arguments, .. } => {
                let args = arguments.iter().map(|v| match v { 
                    fastn_unresolved::UR::UnResolved(v) => v.debug(),
                    fastn_unresolved::UR::Resolved(v) => serde_json::to_value(v).unwrap(),
                    _ => unimplemented!(),
                }).collect::<Vec<_>>();

                serde_json::json!({
                    "args": args,
                    // "return_type": return_type.debug(),
                    // "body": body.debug(),
                })
            }
            crate::InnerDefinition::Component { .. } => todo!(),
            crate::InnerDefinition::Variable { .. } => todo!(),
            crate::InnerDefinition::TypeAlias { .. } => todo!(),
            crate::InnerDefinition::Record { .. } => todo!(),
        }
    }
}

impl fastn_jdebug::JDebug for fastn_unresolved::Argument {
    fn debug(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name.debug(),
            "kind": self.kind.debug(),
            "visibility": self.visibility.debug(),
            "default": self.default.debug(),
        })
    }
}

impl fastn_jdebug::JDebug for fastn_unresolved::Kind {
    fn debug(&self) -> serde_json::Value {
        match self {
            crate::Kind::Integer => "integer".into(),
            crate::Kind::Decimal => "decimal".into(),
            crate::Kind::String => "string".into(),
            crate::Kind::Boolean => "boolean".into(),
            crate::Kind::Option(k) => format!("Option<{}>", k.debug()).into(),
            crate::Kind::List(k) => format!("List<{}>", k.debug()).into(),
            crate::Kind::Caption(k) => format!("Caption<{}>", k.debug()).into(),
            crate::Kind::Body(k) => format!("Body<{}>", k.debug()).into(),
            crate::Kind::CaptionOrBody(k) => format!("CaptionOrBody<{}>", k.debug()).into(),
            crate::Kind::Custom(k) => format!("Custom<{}>", k.debug()).into(),
        }
    }
}

impl fastn_jdebug::JDebug for fastn_unresolved::SymbolName {
    fn debug(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name.debug(),
        })
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
