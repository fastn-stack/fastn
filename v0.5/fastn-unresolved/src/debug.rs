use fastn_section::JDebug;

pub(crate) trait JIDebug {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value;
}

impl fastn_unresolved::JIDebug for fastn_unresolved::ComponentInvocation {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        serde_json::json!({
            "content": self.name.debug(interner),
            "caption": self.caption.debug(interner),
        })
    }
}

impl fastn_unresolved::JIDebug for fastn_unresolved::Definition {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        o.insert("name".into(), self.name.debug(interner));
        let inner = self.inner.debug(interner);
        o.extend(inner.as_object().unwrap().clone());

        serde_json::Value::Object(o)
    }
}

impl fastn_unresolved::JIDebug for fastn_unresolved::InnerDefinition {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        match self {
            crate::InnerDefinition::Function {
                arguments,
                return_type,
                ..
            } => {
                let args = arguments
                    .iter()
                    .map(|v| match v {
                        fastn_unresolved::UR::UnResolved(v) => v.debug(interner),
                        fastn_unresolved::UR::Resolved(_v) => todo!(),
                        _ => unimplemented!(),
                    })
                    .collect::<Vec<_>>();

                let return_type = return_type
                    .clone()
                    .map(|r| match r {
                        fastn_unresolved::UR::UnResolved(v) => v.debug(interner),
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
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        serde_json::json!({
            "name": self.name.debug(),
            "kind": self.kind.debug(interner),
        })
    }
}

impl fastn_unresolved::JIDebug for fastn_unresolved::Kind {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        match self {
            crate::Kind::Integer => "integer".into(),
            crate::Kind::Decimal => "decimal".into(),
            crate::Kind::String => "string".into(),
            crate::Kind::Boolean => "boolean".into(),
            crate::Kind::Option(k) => format!("Option<{}>", k.debug(interner)).into(),
            crate::Kind::List(k) => format!("List<{}>", k.debug(interner)).into(),
            crate::Kind::Caption(k) => format!("Caption<{}>", k.debug(interner)).into(),
            crate::Kind::Body(k) => format!("Body<{}>", k.debug(interner)).into(),
            crate::Kind::CaptionOrBody(k) => format!("CaptionOrBody<{}>", k.debug(interner)).into(),
            crate::Kind::Custom(k) => format!("Custom<{}>", k.debug(interner)).into(),
        }
    }
}

impl fastn_unresolved::JIDebug for fastn_unresolved::Symbol {
    fn debug(&self, _interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        todo!()
    }
}

impl<U: fastn_unresolved::JIDebug, R: fastn_unresolved::JIDebug> fastn_unresolved::JIDebug
    for fastn_unresolved::UR<U, R>
{
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        match self {
            crate::UR::Resolved(r) => r.debug(interner),
            crate::UR::UnResolved(u) => u.debug(interner),
            crate::UR::NotFound => unimplemented!(),
            crate::UR::Invalid(_) => unimplemented!(),
        }
    }
}
