pub fn kinded_name(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<KindedName> {
    let kind = fastn_section::parser::kind(scanner);
    scanner.skip_spaces();

    let name = match fastn_section::parser::identifier_reference(scanner) {
        Some(v) => v,
        None => {
            return kind.and_then(Into::into);
        }
    };

    Some(KindedName { kind, name })
}

#[derive(Debug)]
pub struct KindedName {
    pub kind: Option<fastn_section::Kind>,
    pub name: fastn_section::IdentifierReference,
}

impl fastn_section::JDebug for KindedName {
    fn debug(&self) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        if let Some(kind) = &self.kind {
            o.insert("kind".into(), kind.debug());
        }
        o.insert("name".into(), self.name.debug());
        serde_json::Value::Object(o)
    }
}

impl From<fastn_section::Kind> for Option<KindedName> {
    fn from(value: fastn_section::Kind) -> Self {
        Some(KindedName {
            kind: None,
            name: value.to_identifier_reference()?,
        })
    }
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::kinded_name);

    #[test]
    fn kinded_name() {
        t!("string", {"name": "string"});
        t!("string foo", {"name": "foo", "kind": "string"});
    }
}
