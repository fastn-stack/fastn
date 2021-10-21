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

pub fn string_and_ref(
    name: &str,
    properties: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    all_locals: &ftd_rt::Map,
) -> crate::p1::Result<(String, Option<String>)> {
    match properties.get(name) {
        Some((crate::Value::String { text, .. }, reference)) => {
            Ok((text.to_string(), complete_reference(reference, all_locals)))
        }
        Some(v) => crate::e2(
            format!("expected string, found: {:?}", v),
            "string_and_source",
        ),
        None => crate::e2(format!("'{}' not found", name), "string_and_source"),
    }
}

pub fn string_and_source_and_ref(
    name: &str,
    properties: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    all_locals: &ftd_rt::Map,
) -> crate::p1::Result<(String, crate::TextSource, Option<String>)> {
    match properties.get(name) {
        Some((crate::Value::String { text, source }, reference)) => Ok((
            text.to_string(),
            source.to_owned(),
            complete_reference(reference, all_locals),
        )),
        Some(v) => crate::e2(
            format!("expected string, found: {:?}", v),
            "string_and_source",
        ),
        None => crate::e2(format!("'{}' not found", name), "string_and_source"),
    }
}

pub fn complete_reference(reference: &Option<String>, all_locals: &ftd_rt::Map) -> Option<String> {
    let mut reference = reference.to_owned();
    if let Some(ref r) = reference {
        if let Some(name) = r.strip_prefix('@') {
            if let Some(string_container) = all_locals.get(name) {
                reference = Some(format!("@{}@{}", name, string_container));
            }
        }
    }
    reference
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

pub fn reorder(p1: &[ftd::p1::Section]) -> ftd::p1::Result<Vec<ftd::p1::Section>> {
    fn is_kernel_component(comp: String) -> bool {
        if ["ftd.row", "ftd.column"].contains(&comp.as_str()) {
            return true;
        }
        false
    }

    fn reorder_component(
        p1_map: &std::collections::BTreeMap<String, ftd::p1::Section>,
        new_p1: &mut Vec<ftd::p1::Section>,
        dependent_p1: Option<String>,
        inserted: &mut Vec<String>,
    ) {
        if let Some(p1) = dependent_p1 {
            if inserted.contains(&p1) {
                return;
            }
            if let Some(v) = p1_map.get(&p1) {
                for sub_section in v.sub_sections.0.iter() {
                    if inserted.contains(&sub_section.name) || p1 == sub_section.name {
                        continue;
                    }
                    reorder_component(p1_map, new_p1, Some(sub_section.name.to_string()), inserted);
                }
                if let Ok(root) = v.header.string("component") {
                    if !is_kernel_component(root.to_string()) && !inserted.contains(&root) {
                        reorder_component(p1_map, new_p1, Some(root), inserted);
                    }
                }
                new_p1.push(v.to_owned());
                inserted.push(p1.to_string());
            }
            return;
        }

        for (k, v) in p1_map {
            if inserted.contains(k) {
                continue;
            }
            for sub_section in v.sub_sections.0.iter() {
                if inserted.contains(&sub_section.name) || k == &sub_section.name {
                    continue;
                }
                reorder_component(p1_map, new_p1, Some(sub_section.name.to_string()), inserted);
            }
            if let Ok(root) = v.header.string("component") {
                if !is_kernel_component(root.to_string()) && !inserted.contains(&root) {
                    reorder_component(p1_map, new_p1, Some(root), inserted);
                }
            }

            new_p1.push(v.to_owned());
            inserted.push(k.to_string());
        }
    }

    let mut p1_map: std::collections::BTreeMap<String, ftd::p1::Section> = Default::default();
    let mut inserted_p1 = vec![];
    let mut new_p1 = vec![];
    for (idx, p1) in p1.iter().enumerate() {
        if p1.name == "import"
            || p1.name.starts_with("var ")
            || p1.name.starts_with("record ")
            || p1.name.starts_with("or-type ")
            || p1.name.starts_with("list ")
            || p1.name.starts_with("map ")
        {
            inserted_p1.push(idx);
            new_p1.push(p1.to_owned());
        }

        if p1.name.starts_with("component ") {
            p1_map.insert(
                ftd_rt::get_name("component", p1.name.as_str())?.to_string(),
                p1.to_owned(),
            );
            inserted_p1.push(idx);
        }
    }
    let mut new_p1_component = vec![];
    reorder_component(&p1_map, &mut new_p1_component, None, &mut vec![]);
    new_p1.extend(new_p1_component);

    for (idx, p1) in p1.iter().enumerate() {
        if inserted_p1.contains(&idx) {
            continue;
        }
        new_p1.push(p1.to_owned());
    }

    Ok(new_p1)
}

pub fn properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
) -> std::collections::BTreeMap<String, crate::Value> {
    let mut properties: std::collections::BTreeMap<String, crate::Value> = Default::default();
    for (k, (v, _)) in properties_with_ref {
        properties.insert(k.to_string(), v.to_owned());
    }
    properties
}
