pub fn parse_import(c: &Option<String>) -> crate::p1::Result<(String, String)> {
    let v = match c {
        Some(v) => v.trim(),
        None => return crate::e("caption is missing in import statement".to_string()),
    };

    if v.contains(" as ") {
        let mut parts = v.splitn(2, " as ");
        return match (parts.next(), parts.next()) {
            (Some(n), Some(a)) => Ok((n.to_string(), a.to_string())),
            _ => crate::e("invalid use of keyword as in import statement".to_string()),
        };
    }

    if !v.contains('/') {
        return Ok((v.to_string(), v.to_string()));
    }

    let mut parts = v.rsplitn(2, '/');
    match (parts.next(), parts.next()) {
        (Some(t), Some(_)) => Ok((v.to_string(), t.to_string())),
        _ => crate::e("doc id must contain /".to_string()),
    }
}

pub fn string(
    name: &str,
    properties: &std::collections::BTreeMap<String, crate::Value>,
) -> crate::p1::Result<String> {
    match properties.get(name) {
        Some(crate::Value::String { text: v, .. }) => Ok(v.to_string()),
        Some(v) => crate::e2(
            format!("[{}] expected string, found: {:?}", name, v),
            "string",
        ),
        None => crate::e2(format!("'{}' not found", name), "string"),
    }
}

pub fn string_and_source(
    name: &str,
    properties: &std::collections::BTreeMap<String, crate::Value>,
) -> crate::p1::Result<(String, crate::TextSource)> {
    match properties.get(name) {
        Some(crate::Value::String { text, source }) => Ok((text.to_string(), source.to_owned())),
        Some(v) => crate::e2(
            format!("expected string, found: {:?}", v),
            "string_and_source",
        ),
        None => crate::e2(format!("'{}' not found", name), "string_and_source"),
    }
}

pub fn string_optional(
    name: &str,
    properties: &std::collections::BTreeMap<String, crate::Value>,
) -> crate::p1::Result<Option<String>> {
    match properties.get(name) {
        Some(crate::Value::String { text: v, .. }) => Ok(Some(v.to_string())),
        Some(crate::Value::None {
            kind: crate::p2::Kind::String { .. },
        }) => Ok(None),
        Some(ftd::Value::None { .. }) => Ok(None),
        Some(v) => crate::e2(
            format!("expected string, found: {:?}", v),
            "string_optional",
        ),
        None => Ok(None),
    }
}

pub fn string_with_default(
    name: &str,
    def: &str,
    properties: &std::collections::BTreeMap<String, crate::Value>,
) -> crate::p1::Result<String> {
    match properties.get(name) {
        Some(crate::Value::String { text: v, .. }) => Ok(v.to_string()),
        Some(crate::Value::None {
            kind: crate::p2::Kind::String { .. },
        }) => Ok(def.to_string()),
        Some(ftd::Value::None { .. }) => Ok(def.to_string()),
        Some(v) => crate::e2(
            format!("expected bool, found: {:?}", v),
            "string_with_default",
        ),
        None => Ok(def.to_string()),
    }
}

pub fn int(
    name: &str,
    properties: &std::collections::BTreeMap<String, crate::Value>,
) -> crate::p1::Result<i64> {
    match properties.get(name) {
        Some(crate::Value::Integer { value: v, .. }) => Ok(*v),
        Some(v) => crate::e2(format!("[{}] expected int, found: {:?}", name, v), "int"),
        None => crate::e2(format!("'{}' not found", name), "int"),
    }
}

pub fn int_optional(
    name: &str,
    properties: &std::collections::BTreeMap<String, crate::Value>,
) -> crate::p1::Result<Option<i64>> {
    match properties.get(name) {
        Some(crate::Value::Integer { value: v }) => Ok(Some(*v)),
        Some(crate::Value::None {
            kind: crate::p2::Kind::Integer { .. },
        }) => Ok(None),
        Some(ftd::Value::None { .. }) => Ok(None),
        Some(v) => crate::e2(format!("expected int, found: {:?}", v), "int_optional"),
        None => Ok(None),
    }
}

pub fn int_with_default(
    name: &str,
    def: i64,
    properties: &std::collections::BTreeMap<String, crate::Value>,
) -> crate::p1::Result<i64> {
    match properties.get(name) {
        Some(crate::Value::Integer { value: v }) => Ok(*v),
        Some(crate::Value::None {
            kind: crate::p2::Kind::Integer { .. },
        }) => Ok(def),
        Some(ftd::Value::None { .. }) => Ok(def),
        Some(v) => crate::e2(format!("expected int, found: {:?}", v), "int_with_default"),
        None => Ok(def),
    }
}

// pub fn elements(
//     name: &str,
//     properties: &std::collections::BTreeMap<String, crate::Value>,
// ) -> crate::p1::Result<Vec<ftd_rt::Element>> {
//     match properties.get(name) {
//         Some(crate::Value::Elements(v)) => Ok((*v).clone()),
//         Some(v) => crate::e(format!("expected elements, found: {:?}", v)),
//         None => crate::e(format!("'{}' not found", name)),
//     }
// }

pub fn bool_with_default(
    name: &str,
    def: bool,
    properties: &std::collections::BTreeMap<String, crate::Value>,
) -> crate::p1::Result<bool> {
    match properties.get(name) {
        Some(crate::Value::Boolean { value: v }) => Ok(*v),
        Some(crate::Value::None {
            kind: crate::p2::Kind::Boolean { .. },
        }) => Ok(def),
        Some(ftd::Value::None { .. }) => Ok(def),
        Some(v) => crate::e2(
            format!("expected bool, found: {:?}", v),
            "bool_with_default",
        ),
        None => Ok(def),
    }
}

pub fn bool(
    name: &str,
    properties: &std::collections::BTreeMap<String, crate::Value>,
) -> crate::p1::Result<bool> {
    match properties.get(name) {
        Some(crate::Value::Boolean { value: v, .. }) => Ok(*v),
        Some(v) => crate::e2(
            format!("[{}] expected bool, found: {:?}", name, v),
            "string",
        ),
        None => crate::e2(format!("'{}' not found", name), "bool"),
    }
}

pub fn string_bool_optional(
    name: &str,
    properties: &std::collections::BTreeMap<String, crate::Value>,
) -> crate::p1::Result<(Option<bool>, Option<String>)> {
    match properties.get(name) {
        Some(crate::Value::Boolean { value: v }) => Ok((Some(*v), None)),
        Some(crate::Value::None {
            kind: crate::p2::Kind::Boolean { .. },
        }) => Ok((None, None)),
        Some(ftd::Value::None { .. }) => Ok((None, None)),
        Some(crate::Value::String { text: v, .. }) => {
            if let Ok(b) = v.parse::<bool>() {
                Ok((Some(b), None))
            } else {
                Ok((None, Some(v.to_string())))
            }
        }
        Some(v) => crate::e2(format!("expected bool, found: {:?}", v), "bool_optional"),
        None => Ok((None, None)),
    }
}

#[cfg(test)]
mod test {
    macro_rules! p {
        ($s:expr, $id: expr, $alias: expr) => {
            assert_eq!(
                super::parse_import(&Some($s.to_string())).unwrap_or_else(|e| panic!("{}", e)),
                ($id.to_string(), $alias.to_string())
            )
        };
    }

    #[test]
    fn parse_import() {
        p!("a/b/c as foo", "a/b/c", "foo");
        p!("a/b as foo", "a/b", "foo");
        p!("a/b/c", "a/b/c", "c");
        p!("a/b", "a/b", "b");
        p!("a", "a", "a");
        p!("a as b", "a", "b");
    }
}

pub fn decimal(
    name: &str,
    properties: &std::collections::BTreeMap<String, crate::Value>,
) -> crate::p1::Result<f64> {
    match properties.get(name) {
        Some(crate::Value::Decimal { value: v, .. }) => Ok(*v),
        Some(v) => crate::e2(
            format!("[{}] expected Decimal, found: {:?}", name, v),
            "string",
        ),
        None => crate::e2(format!("'{}' not found", name), "decimal"),
    }
}

pub fn split(name: String, split_at: &str) -> crate::p1::Result<(String, String)> {
    let mut part = name.splitn(2, split_at);
    let part_1 = part.next().unwrap().trim();
    let part_2 = part.next().unwrap().trim();
    Ok((part_1.to_string(), part_2.to_string()))
}
