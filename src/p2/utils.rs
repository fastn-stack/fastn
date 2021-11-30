pub fn parse_import(
    c: &Option<String>,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<(String, String)> {
    let v = match c {
        Some(v) => v.trim(),
        None => {
            return ftd::e2(
                "caption is missing in import statement",
                doc_id,
                line_number,
            )
        }
    };

    if v.contains(" as ") {
        let mut parts = v.splitn(2, " as ");
        return match (parts.next(), parts.next()) {
            (Some(n), Some(a)) => Ok((n.to_string(), a.to_string())),
            _ => ftd::e2(
                "invalid use of keyword as in import statement",
                doc_id,
                line_number,
            ),
        };
    }

    if !v.contains('/') {
        return Ok((v.to_string(), v.to_string()));
    }

    let mut parts = v.rsplitn(2, '/');
    match (parts.next(), parts.next()) {
        (Some(t), Some(_)) => Ok((v.to_string(), t.to_string())),
        _ => ftd::e2("doc id must contain /", doc_id, line_number),
    }
}

pub fn string_and_ref(
    line_number: usize,
    name: &str,
    properties: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
    all_locals: &mut ftd::Map,
    doc_id: &str,
) -> ftd::p1::Result<(String, Option<String>)> {
    match properties.get(name) {
        Some((ftd::Value::String { text, .. }, reference)) => {
            Ok((text.to_string(), complete_reference(reference, all_locals)))
        }
        Some(v) => ftd::e2(
            format!("expected string, found: {:?}", v),
            doc_id,
            line_number,
        ),
        None => ftd::e2(format!("'{}' not found", name), doc_id, line_number),
    }
}

pub fn string_and_source_and_ref(
    line_number: usize,
    name: &str,
    properties: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
    all_locals: &mut ftd::Map,
    doc_id: &str,
) -> ftd::p1::Result<(String, ftd::TextSource, Option<String>)> {
    match properties.get(name) {
        Some((ftd::Value::String { text, source }, reference)) => Ok((
            text.to_string(),
            source.to_owned(),
            complete_reference(reference, all_locals),
        )),
        Some(v) => ftd::e2(
            format!("expected string, found: {:?}", v),
            doc_id,
            line_number,
        ),
        None => ftd::e2(format!("'{}' not found", name), doc_id, line_number),
    }
}

pub fn complete_reference(reference: &Option<String>, all_locals: &mut ftd::Map) -> Option<String> {
    let mut reference = reference.to_owned();
    if let Some(ref r) = reference {
        if let Some(name) = r.strip_prefix('@') {
            if name.eq("$loop$") {
                return None;
            }
            if let Some(string_container) = all_locals.get(name) {
                reference = Some(format!("@{}@{}", name, string_container));
            } else if name.eq("MOUSE-IN") {
                let string_container = all_locals.get("MOUSE-IN-TEMP").unwrap().clone();
                all_locals.insert("MOUSE-IN".to_string(), string_container.to_string());
                reference = Some(format!("@MOUSE-IN@{}", string_container));
            }
        }
    }
    reference
}

pub fn string_optional(
    name: &str,
    properties: &std::collections::BTreeMap<String, ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<Option<String>> {
    match properties.get(name) {
        Some(ftd::Value::String { text: v, .. }) => Ok(Some(v.to_string())),
        Some(ftd::Value::None {
            kind: ftd::p2::Kind::String { .. },
        }) => Ok(None),
        Some(ftd::Value::None { .. }) => Ok(None),
        Some(v) => ftd::e2(
            format!("expected string, found: {:?}", v),
            doc_id,
            line_number,
        ),
        None => Ok(None),
    }
}

pub fn string_with_default(
    name: &str,
    def: &str,
    properties: &std::collections::BTreeMap<String, ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<String> {
    match properties.get(name) {
        Some(ftd::Value::String { text: v, .. }) => Ok(v.to_string()),
        Some(ftd::Value::None {
            kind: ftd::p2::Kind::String { .. },
        }) => Ok(def.to_string()),
        Some(ftd::Value::None { .. }) => Ok(def.to_string()),
        Some(v) => ftd::e2(
            format!("expected bool, found: {:?}", v),
            doc_id,
            line_number,
        ),
        None => Ok(def.to_string()),
    }
}

pub fn int(
    name: &str,
    properties: &std::collections::BTreeMap<String, ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<i64> {
    match properties.get(name) {
        Some(ftd::Value::Integer { value: v, .. }) => Ok(*v),
        Some(v) => ftd::e2(
            format!("[{}] expected int, found: {:?}", name, v),
            doc_id,
            line_number,
        ),
        None => ftd::e2(format!("'{}' not found", name), doc_id, line_number),
    }
}

pub fn int_optional(
    name: &str,
    properties: &std::collections::BTreeMap<String, ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<Option<i64>> {
    match properties.get(name) {
        Some(ftd::Value::Integer { value: v }) => Ok(Some(*v)),
        Some(ftd::Value::None {
            kind: ftd::p2::Kind::Integer { .. },
        }) => Ok(None),
        Some(ftd::Value::None { .. }) => Ok(None),
        Some(v) => ftd::e2(format!("expected int, found: {:?}", v), doc_id, line_number),
        None => Ok(None),
    }
}

pub fn int_with_default(
    name: &str,
    def: i64,
    properties: &std::collections::BTreeMap<String, ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<i64> {
    match properties.get(name) {
        Some(ftd::Value::Integer { value: v }) => Ok(*v),
        Some(ftd::Value::None {
            kind: ftd::p2::Kind::Integer { .. },
        }) => Ok(def),
        Some(ftd::Value::None { .. }) => Ok(def),
        Some(v) => ftd::e2(format!("expected int, found: {:?}", v), doc_id, line_number),
        None => Ok(def),
    }
}

// pub fn elements(
//     name: &str,
//     properties: &std::collections::BTreeMap<String, ftd::Value>,
// ) -> ftd::p1::Result<Vec<ftd::Element>> {
//     match properties.get(name) {
//         Some(ftd::Value::Elements(v)) => Ok((*v).clone()),
//         Some(v) => ftd::e(format!("expected elements, found: {:?}", v)),
//         None => ftd::e(format!("'{}' not found", name)),
//     }
// }

pub fn bool_with_default(
    name: &str,
    def: bool,
    properties: &std::collections::BTreeMap<String, ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<bool> {
    match properties.get(name) {
        Some(ftd::Value::Boolean { value: v }) => Ok(*v),
        Some(ftd::Value::None {
            kind: ftd::p2::Kind::Boolean { .. },
        }) => Ok(def),
        Some(ftd::Value::None { .. }) => Ok(def),
        Some(v) => ftd::e2(
            format!("expected bool, found: {:?}", v),
            doc_id,
            line_number,
        ),
        None => Ok(def),
    }
}

pub fn bool(
    name: &str,
    properties: &std::collections::BTreeMap<String, ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<bool> {
    match properties.get(name) {
        Some(ftd::Value::Boolean { value: v, .. }) => Ok(*v),
        Some(v) => ftd::e2(
            format!("[{}] expected bool, found: {:?}", name, v),
            doc_id,
            line_number,
        ),
        None => ftd::e2(format!("'{}' not found", name), doc_id, line_number),
    }
}

pub fn string_bool_optional(
    name: &str,
    properties: &std::collections::BTreeMap<String, ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<(Option<bool>, Option<String>)> {
    match properties.get(name) {
        Some(ftd::Value::Boolean { value: v }) => Ok((Some(*v), None)),
        Some(ftd::Value::None {
            kind: ftd::p2::Kind::Boolean { .. },
        }) => Ok((None, None)),
        Some(ftd::Value::None { .. }) => Ok((None, None)),
        Some(ftd::Value::String { text: v, .. }) => {
            if let Ok(b) = v.parse::<bool>() {
                Ok((Some(b), None))
            } else {
                Ok((None, Some(v.to_string())))
            }
        }
        Some(v) => ftd::e2(
            format!("expected bool, found: {:?}", v),
            doc_id,
            line_number,
        ),
        None => Ok((None, None)),
    }
}

#[cfg(test)]
mod test {
    macro_rules! p {
        ($s:expr, $id: expr, $alias: expr) => {
            assert_eq!(
                super::parse_import(&Some($s.to_string()), $id, 0)
                    .unwrap_or_else(|e| panic!("{}", e)),
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
    properties: &std::collections::BTreeMap<String, ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<f64> {
    match properties.get(name) {
        Some(ftd::Value::Decimal { value: v, .. }) => Ok(*v),
        Some(v) => ftd::e2(
            format!("[{}] expected Decimal, found: {:?}", name, v),
            doc_id,
            line_number,
        ),
        None => ftd::e2(format!("'{}' not found", name), doc_id, line_number),
    }
}

pub fn decimal_optional(
    name: &str,
    properties: &std::collections::BTreeMap<String, ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<Option<f64>> {
    match properties.get(name) {
        Some(ftd::Value::Decimal { value: v }) => Ok(Some(*v)),
        Some(ftd::Value::None {
            kind: ftd::p2::Kind::Decimal { .. },
        }) => Ok(None),
        Some(ftd::Value::None { .. }) => Ok(None),
        Some(v) => ftd::e2(
            format!("expected decimal, found: {:?}", v),
            doc_id,
            line_number,
        ),
        None => Ok(None),
    }
}

pub fn split(name: String, split_at: &str) -> ftd::p1::Result<(String, String)> {
    let mut part = name.splitn(2, split_at);
    let part_1 = part.next().unwrap().trim();
    let part_2 = part.next().unwrap().trim();
    Ok((part_1.to_string(), part_2.to_string()))
}

pub fn reorder(
    p1: &[ftd::p1::Section],
    doc_id: &str,
) -> ftd::p1::Result<(Vec<ftd::p1::Section>, Vec<String>)> {
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
        doc: &ftd::p2::TDoc,
        var_types: &[String],
    ) -> ftd::p1::Result<()> {
        if let Some(p1) = dependent_p1 {
            if inserted.contains(&p1) {
                return Ok(());
            }
            if let Some(v) = p1_map.get(&p1) {
                for sub_section in v.sub_sections.0.iter() {
                    if inserted.contains(&sub_section.name) || p1 == sub_section.name {
                        continue;
                    }
                    reorder_component(
                        p1_map,
                        new_p1,
                        Some(sub_section.name.to_string()),
                        inserted,
                        doc,
                        var_types,
                    )?;
                }
                let var_data = ftd::variable::VariableData::get_name_kind(
                    &v.name,
                    doc,
                    v.line_number,
                    var_types,
                )?;
                if !is_kernel_component(var_data.kind.to_string())
                    && !inserted.contains(&var_data.kind)
                {
                    reorder_component(
                        p1_map,
                        new_p1,
                        Some(var_data.kind),
                        inserted,
                        doc,
                        var_types,
                    )?;
                }
                new_p1.push(v.to_owned());
                inserted.push(p1.to_string());
            }
            return Ok(());
        }

        for (k, v) in p1_map {
            if inserted.contains(k) {
                continue;
            }
            for sub_section in v.sub_sections.0.iter() {
                if inserted.contains(&sub_section.name) || k == &sub_section.name {
                    continue;
                }
                reorder_component(
                    p1_map,
                    new_p1,
                    Some(sub_section.name.to_string()),
                    inserted,
                    doc,
                    var_types,
                )?;
            }
            let var_data =
                ftd::variable::VariableData::get_name_kind(&v.name, doc, v.line_number, var_types)?;
            if !is_kernel_component(var_data.kind.to_string()) && !inserted.contains(&var_data.kind)
            {
                reorder_component(
                    p1_map,
                    new_p1,
                    Some(var_data.kind),
                    inserted,
                    doc,
                    var_types,
                )?;
            }

            new_p1.push(v.to_owned());
            inserted.push(k.to_string());
        }
        Ok(())
    }

    let doc = ftd::p2::TDoc {
        name: doc_id,
        aliases: &Default::default(),
        bag: &Default::default(),
    };

    let mut p1_map: std::collections::BTreeMap<String, ftd::p1::Section> = Default::default();
    let mut inserted_p1 = vec![];
    let mut new_p1 = vec![];
    let mut list_or_var = vec![];
    let mut var_types = vec![];
    for (idx, p1) in p1.iter().enumerate() {
        let var_data =
            ftd::variable::VariableData::get_name_kind(&p1.name, &doc, p1.line_number, &var_types);
        if p1.name == "import"
            || p1.name.starts_with("record ")
            || p1.name.starts_with("or-type ")
            || p1.name.starts_with("map ")
        {
            inserted_p1.push(idx);
            new_p1.push(p1.to_owned());
        }
        if let Ok(ftd::variable::VariableData {
            type_: ftd::variable::Type::Variable,
            ref name,
            ..
        }) = var_data
        {
            inserted_p1.push(idx);
            new_p1.push(p1.to_owned());
            list_or_var.push(name.to_string());
        }

        if p1.name.starts_with("record ") {
            let name = ftd::get_name("record", &p1.name, "")?;
            var_types.push(name.to_string());
        }

        if p1.name.starts_with("or-type ") {
            let name = ftd::get_name("or-type", &p1.name, "")?;
            var_types.push(name.to_string());
            for s in &p1.sub_sections.0 {
                var_types.push(format!("{}.{}", name, s.name));
            }
        }

        if list_or_var.contains(&p1.name) {
            inserted_p1.push(idx);
            new_p1.push(p1.to_owned());
        }

        if let Ok(ftd::variable::VariableData {
            type_: ftd::variable::Type::Component,
            ref name,
            ..
        }) = var_data
        {
            p1_map.insert(name.to_string(), p1.to_owned());
            inserted_p1.push(idx);
        }
    }
    let mut new_p1_component = vec![];
    reorder_component(
        &p1_map,
        &mut new_p1_component,
        None,
        &mut vec![],
        &doc,
        &var_types,
    )?;
    new_p1.extend(new_p1_component);

    for (idx, p1) in p1.iter().enumerate() {
        if inserted_p1.contains(&idx) {
            continue;
        }
        new_p1.push(p1.to_owned());
    }

    Ok((new_p1, var_types))
}

pub fn properties(
    properties_with_ref: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
) -> std::collections::BTreeMap<String, ftd::Value> {
    let mut properties: std::collections::BTreeMap<String, ftd::Value> = Default::default();
    for (k, (v, _)) in properties_with_ref {
        properties.insert(k.to_string(), v.to_owned());
    }
    properties
}

pub fn structure_header_to_properties(
    s: &str,
    arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
    doc: &ftd::p2::TDoc,
    line_number: usize,
) -> ftd::p1::Result<std::collections::BTreeMap<String, ftd::component::Property>> {
    let (name, caption) = ftd::p2::utils::split(s.to_string(), ":")?;
    match doc.get_thing(line_number, &name) {
        Ok(ftd::p2::Thing::Component(c)) => ftd::component::read_properties(
            line_number,
            &Default::default(),
            &if caption.is_empty() {
                None
            } else {
                Some(caption)
            },
            &None,
            "",
            "",
            &c.arguments,
            arguments,
            doc,
            &Default::default(),
            false,
        ),
        t => {
            return ftd::e2(
                format!("expected component, found: {:?}", t),
                doc.name,
                line_number,
            )
        }
    }
}
