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
        let mut o = serde_json::Map::new();
        o.insert("name".into(), self.name.debug());
        let inner = self.inner.debug();
        o.extend(inner.as_object().unwrap().clone());

        serde_json::Value::Object(o)
    }
}

impl fastn_jdebug::JDebug for fastn_unresolved::InnerDefinition {
    fn debug(&self) -> serde_json::Value {
        match self {
            crate::InnerDefinition::Function { arguments, return_type, .. } => {
                let args = arguments.iter().map(|v| match v { 
                    fastn_unresolved::UR::UnResolved(v) => v.debug(),
                    fastn_unresolved::UR::Resolved(_v) => todo!(),
                    _ => unimplemented!(),
                }).collect::<Vec<_>>();


                let return_type = return_type.clone().map(|r| match r {
                    fastn_unresolved::UR::UnResolved(v) => v.debug(),
                    fastn_unresolved::UR::Resolved(v) => serde_json::to_value(v).unwrap(),
                    _ => unimplemented!(),
                }).unwrap_or_else(|| "void".into());

                serde_json::json!({
                    "args": args,
                    "return_type": return_type,
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
