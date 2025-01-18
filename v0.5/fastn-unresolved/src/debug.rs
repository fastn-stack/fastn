use fastn_section::JDebug;

// this leads to conflicting implementation issue
// impl<T: JDebug> fastn_unresolved::JIDebug for T {
//     fn idebug(&self, _arena: &fastn_section::Arena) -> serde_json::Value {
//         self.debug()
//     }
// }

impl<T: fastn_unresolved::JIDebug> fastn_unresolved::JIDebug for Option<T> {
    fn idebug(&self, arena: &fastn_section::Arena) -> serde_json::Value {
        self.as_ref()
            .map(|v| v.idebug(arena))
            .unwrap_or(serde_json::Value::Null)
    }
}

impl fastn_unresolved::JIDebug for fastn_section::Identifier {
    fn idebug(&self, _arena: &fastn_section::Arena) -> serde_json::Value {
        self.debug()
    }
}

impl fastn_unresolved::JIDebug for fastn_section::IdentifierReference {
    fn idebug(&self, _arena: &fastn_section::Arena) -> serde_json::Value {
        self.debug()
    }
}

impl fastn_unresolved::JIDebug for fastn_section::HeaderValue {
    fn idebug(&self, _arena: &fastn_section::Arena) -> serde_json::Value {
        self.debug()
    }
}

impl fastn_unresolved::JIDebug for () {
    fn idebug(&self, _arena: &fastn_section::Arena) -> serde_json::Value {
        self.debug()
    }
}

impl fastn_unresolved::JIDebug for fastn_section::Symbol {
    fn idebug(&self, arena: &fastn_section::Arena) -> serde_json::Value {
        self.string(arena).into()
    }
}

impl<T: fastn_unresolved::JIDebug> fastn_unresolved::JIDebug for Vec<T> {
    fn idebug(&self, arena: &fastn_section::Arena) -> serde_json::Value {
        serde_json::Value::Array(self.iter().map(|v| v.idebug(arena)).collect())
    }
}

impl<U: fastn_unresolved::JIDebug, R: fastn_unresolved::JIDebug> fastn_unresolved::JIDebug
    for fastn_section::UR<U, R>
{
    fn idebug(&self, arena: &fastn_section::Arena) -> serde_json::Value {
        match self {
            fastn_section::UR::Resolved(r) => r.idebug(arena),
            fastn_section::UR::UnResolved(u) => u.idebug(arena),
            fastn_section::UR::NotFound => unimplemented!(),
            fastn_section::UR::Invalid(_) => unimplemented!(),
            fastn_section::UR::InvalidN(_) => unimplemented!(),
        }
    }
}

impl fastn_unresolved::JIDebug for fastn_unresolved::ComponentInvocation {
    fn idebug(&self, arena: &fastn_section::Arena) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        o.insert("content".into(), self.name.idebug(arena));
        o.insert("caption".into(), self.caption.idebug(arena));
        if !self.properties.is_empty() {
            o.insert("properties".into(), self.properties.idebug(arena));
        }
        serde_json::Value::Object(o)
    }
}

impl fastn_unresolved::JIDebug for fastn_resolved::Property {
    fn idebug(&self, _arena: &fastn_section::Arena) -> serde_json::Value {
        todo!()
    }
}

impl fastn_unresolved::JIDebug for fastn_unresolved::Property {
    fn idebug(&self, arena: &fastn_section::Arena) -> serde_json::Value {
        serde_json::json!({
            "name": self.name.idebug(arena),
            "value": self.value.idebug(arena),
        })
    }
}

impl fastn_unresolved::JIDebug for fastn_unresolved::Definition {
    fn idebug(&self, arena: &fastn_section::Arena) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        o.insert("name".into(), self.name.idebug(arena));
        let inner = self.inner.idebug(arena);
        o.extend(inner.as_object().unwrap().clone());

        serde_json::Value::Object(o)
    }
}

impl fastn_unresolved::JIDebug for fastn_unresolved::InnerDefinition {
    fn idebug(&self, arena: &fastn_section::Arena) -> serde_json::Value {
        match self {
            crate::InnerDefinition::Function {
                arguments,
                return_type,
                ..
            } => {
                let args = arguments
                    .iter()
                    .map(|v| match v {
                        fastn_unresolved::UR::UnResolved(v) => v.idebug(arena),
                        fastn_unresolved::UR::Resolved(_v) => todo!(),
                        _ => unimplemented!(),
                    })
                    .collect::<Vec<_>>();

                let return_type = return_type
                    .clone()
                    .map(|r| match r {
                        fastn_unresolved::UR::UnResolved(v) => v.idebug(arena),
                        fastn_unresolved::UR::Resolved(v) => serde_json::to_value(v).unwrap(),
                        _ => unimplemented!(),
                    })
                    .unwrap_or_else(|| "void".into());

                serde_json::json!({
                    "args": args,
                    "return_type": return_type,
                    // "body": body.debug(),
                })
            }
            crate::InnerDefinition::Component { .. } => todo!(),
            crate::InnerDefinition::Variable { .. } => todo!(),
            crate::InnerDefinition::Record { .. } => todo!(),
        }
    }
}

impl fastn_unresolved::JIDebug for fastn_unresolved::Argument {
    fn idebug(&self, arena: &fastn_section::Arena) -> serde_json::Value {
        serde_json::json!({
            "name": self.name.debug(),
            "kind": self.kind.idebug(arena),
        })
    }
}

impl fastn_unresolved::JIDebug for fastn_unresolved::Kind {
    fn idebug(&self, arena: &fastn_section::Arena) -> serde_json::Value {
        match self {
            crate::Kind::Integer => "integer".into(),
            crate::Kind::Decimal => "decimal".into(),
            crate::Kind::String => "string".into(),
            crate::Kind::Boolean => "boolean".into(),
            crate::Kind::Option(k) => format!("Option<{}>", k.idebug(arena)).into(),
            crate::Kind::List(k) => format!("List<{}>", k.idebug(arena)).into(),
            crate::Kind::Caption(k) => format!("Caption<{}>", k.idebug(arena)).into(),
            crate::Kind::Body(k) => format!("Body<{}>", k.idebug(arena)).into(),
            crate::Kind::CaptionOrBody(k) => format!("CaptionOrBody<{}>", k.idebug(arena)).into(),
            crate::Kind::Custom(k) => format!("Custom<{}>", k.idebug(arena)).into(),
        }
    }
}
