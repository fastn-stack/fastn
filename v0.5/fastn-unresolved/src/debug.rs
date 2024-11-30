use fastn_section::JDebug;

pub(crate) trait JIDebug {
    fn idebug(&self, arena: &fastn_unresolved::Arena) -> serde_json::Value;
}

impl<T: JDebug> JIDebug for T {
    fn idebug(&self, _arena: &fastn_unresolved::Arena) -> serde_json::Value {
        self.debug()
    }
}

impl fastn_unresolved::JIDebug for fastn_unresolved::ComponentInvocation {
    fn idebug(&self, arena: &fastn_unresolved::Arena) -> serde_json::Value {
        serde_json::json!({
            "content": self.name.idebug(arena),
            "caption": self.caption.idebug(arena),
        })
    }
}

impl fastn_unresolved::JIDebug for fastn_unresolved::Definition {
    fn idebug(&self, arena: &fastn_unresolved::Arena) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        o.insert("name".into(), self.name.idebug(arena));
        let inner = self.inner.idebug(arena);
        o.extend(inner.as_object().unwrap().clone());

        serde_json::Value::Object(o)
    }
}

impl fastn_unresolved::JIDebug for fastn_unresolved::InnerDefinition {
    fn idebug(&self, arena: &fastn_unresolved::Arena) -> serde_json::Value {
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
            crate::InnerDefinition::SymbolAlias { .. } => todo!(),
            crate::InnerDefinition::ModuleAlias { .. } => todo!(),
        }
    }
}

impl fastn_unresolved::JIDebug for fastn_unresolved::Argument {
    fn idebug(&self, arena: &fastn_unresolved::Arena) -> serde_json::Value {
        serde_json::json!({
            "name": self.name.debug(),
            "kind": self.kind.idebug(arena),
        })
    }
}

impl fastn_unresolved::JIDebug for fastn_unresolved::Kind {
    fn idebug(&self, arena: &fastn_unresolved::Arena) -> serde_json::Value {
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

impl fastn_unresolved::JIDebug for fastn_unresolved::Symbol {
    fn idebug(&self, _arena: &fastn_unresolved::Arena) -> serde_json::Value {
        todo!()
    }
}

impl<U: fastn_unresolved::JIDebug, R: fastn_unresolved::JIDebug> fastn_unresolved::JIDebug
    for fastn_unresolved::UR<U, R>
{
    fn idebug(&self, arena: &fastn_unresolved::Arena) -> serde_json::Value {
        match self {
            crate::UR::Resolved(r) => r.idebug(arena),
            crate::UR::UnResolved(u) => u.idebug(arena),
            crate::UR::NotFound => unimplemented!(),
            crate::UR::Invalid(_) => unimplemented!(),
        }
    }
}
