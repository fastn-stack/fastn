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

    if v.contains('/') {
        let mut parts = v.rsplitn(2, '/');
        return match (parts.next(), parts.next()) {
            (Some(t), Some(_)) => Ok((v.to_string(), t.to_string())),
            _ => ftd::e2("doc id must contain /", doc_id, line_number),
        };
    }

    if let Some((t, _)) = v.split_once('.') {
        return Ok((v.to_string(), t.to_string()));
    }

    Ok((v.to_string(), v.to_string()))
}

pub fn boolean_and_ref(
    line_number: usize,
    name: &str,
    properties: &std::collections::BTreeMap<String, ftd::component::Property>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>, // todo: check the string_and_source_and_ref and use
) -> ftd::p1::Result<(bool, Option<String>)> {
    let properties = ftd::component::resolve_properties_with_ref(line_number, properties, doc)?;
    match properties.get(name) {
        Some((ftd::Value::Boolean { value }, reference)) => {
            Ok((value.to_owned(), complete_reference(reference)))
        }
        Some((ftd::Value::Optional { data, kind }, reference)) => {
            if !matches!(kind, ftd::p2::Kind::Boolean { .. }) {
                return ftd::e2(
                    format!("expected boolean, found: {:?}", kind),
                    doc.name,
                    line_number,
                );
            };
            match data.as_ref() {
                None => {
                    let reference = match reference {
                        Some(reference) => reference,
                        None => {
                            return ftd::e2(
                                format!("expected boolean, found: {:?}", kind),
                                doc.name,
                                line_number,
                            )
                        }
                    };

                    if let Some(ftd::p2::Boolean::IsNotNull { value }) = condition {
                        match value {
                            ftd::PropertyValue::Reference { name, .. }
                            | ftd::PropertyValue::Variable { name, .. } => {
                                if name.eq(reference) {
                                    return Ok((
                                        false,
                                        complete_reference(&Some(reference.to_owned())),
                                    ));
                                }
                            }
                            _ => {}
                        }
                    }

                    // In case when the optional string is null.
                    // Return the empty string

                    Ok((false, complete_reference(&Some(reference.to_owned()))))
                }
                Some(ftd::Value::Boolean { value }) => {
                    Ok((value.to_owned(), complete_reference(reference)))
                }
                _ => {
                    return ftd::e2(
                        format!("expected boolean, found: {:?}", kind),
                        doc.name,
                        line_number,
                    )
                }
            }
        }
        Some((ftd::Value::None { kind }, reference)) if condition.is_some() => {
            let kind = kind.inner();
            if !matches!(kind, ftd::p2::Kind::Boolean { .. }) {
                return ftd::e2(
                    format!("expected boolean, found: {:?}", kind),
                    doc.name,
                    line_number,
                );
            };

            let reference = match reference {
                Some(reference) => reference,
                None => {
                    return ftd::e2(
                        format!("expected integer, found: {:?}", kind),
                        doc.name,
                        line_number,
                    )
                }
            };
            if let Some(ftd::p2::Boolean::IsNotNull { value }) = condition {
                match value {
                    ftd::PropertyValue::Reference { name, .. }
                    | ftd::PropertyValue::Variable { name, .. } => {
                        if name.eq({
                            if let Some(reference) = reference.strip_prefix('@') {
                                reference
                            } else {
                                reference
                            }
                        }) {
                            return Ok((false, complete_reference(&Some(reference.to_owned()))));
                        }
                    }
                    _ => {}
                }
            }
            ftd::e2(
                format!("expected boolean, found: {:?}", kind),
                doc.name,
                line_number,
            )
        }
        Some(v) => ftd::e2(
            format!("expected boolean, found: {:?}", v),
            doc.name,
            line_number,
        ),
        None => ftd::e2(format!("'{}' not found", name), doc.name, line_number),
    }
}

pub fn integer_and_ref(
    line_number: usize,
    name: &str,
    properties: &std::collections::BTreeMap<String, ftd::component::Property>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>, // todo: check the string_and_source_and_ref and use
) -> ftd::p1::Result<(i64, Option<String>)> {
    let properties = ftd::component::resolve_properties_with_ref(line_number, properties, doc)?;
    match properties.get(name) {
        Some((ftd::Value::Integer { value }, reference)) => {
            Ok((value.to_owned(), complete_reference(reference)))
        }
        Some((ftd::Value::Optional { data, kind }, reference)) => {
            if !matches!(kind, ftd::p2::Kind::Integer { .. }) {
                return ftd::e2(
                    format!("expected integer, found: {:?}", kind),
                    doc.name,
                    line_number,
                );
            };
            match data.as_ref() {
                None => {
                    let reference = match reference {
                        Some(reference) => reference,
                        None => {
                            return ftd::e2(
                                format!("expected integer, found: {:?}", kind),
                                doc.name,
                                line_number,
                            )
                        }
                    };

                    if let Some(ftd::p2::Boolean::IsNotNull { value }) = condition {
                        match value {
                            ftd::PropertyValue::Reference { name, .. }
                            | ftd::PropertyValue::Variable { name, .. } => {
                                if name.eq(reference) {
                                    return Ok((
                                        0,
                                        complete_reference(&Some(reference.to_owned())),
                                    ));
                                }
                            }
                            _ => {}
                        }
                    }

                    // In case when the optional string is null.
                    // Return the empty string

                    Ok((0, complete_reference(&Some(reference.to_owned()))))
                }
                Some(ftd::Value::Integer { value }) => {
                    Ok((value.to_owned(), complete_reference(reference)))
                }
                _ => {
                    return ftd::e2(
                        format!("expected integer, found: {:?}", kind),
                        doc.name,
                        line_number,
                    )
                }
            }
        }
        Some((ftd::Value::None { kind }, reference)) if condition.is_some() => {
            let kind = kind.inner();
            if !matches!(kind, ftd::p2::Kind::Integer { .. }) {
                return ftd::e2(
                    format!("expected integer, found: {:?}", kind),
                    doc.name,
                    line_number,
                );
            };

            let reference = match reference {
                Some(reference) => reference,
                None => {
                    return ftd::e2(
                        format!("expected integer, found: {:?}", kind),
                        doc.name,
                        line_number,
                    )
                }
            };
            if let Some(ftd::p2::Boolean::IsNotNull { value }) = condition {
                match value {
                    ftd::PropertyValue::Reference { name, .. }
                    | ftd::PropertyValue::Variable { name, .. } => {
                        if name.eq({
                            if let Some(reference) = reference.strip_prefix('@') {
                                reference
                            } else {
                                reference
                            }
                        }) {
                            return Ok((0, complete_reference(&Some(reference.to_owned()))));
                        }
                    }
                    _ => {}
                }
            }
            ftd::e2(
                format!("expected integer, found: {:?}", kind),
                doc.name,
                line_number,
            )
        }
        Some(v) => ftd::e2(
            format!("expected integer, found: {:?}", v),
            doc.name,
            line_number,
        ),
        None => ftd::e2(format!("'{}' not found", name), doc.name, line_number),
    }
}

pub fn decimal_and_ref(
    line_number: usize,
    name: &str,
    properties: &std::collections::BTreeMap<String, ftd::component::Property>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>, // todo: check the string_and_source_and_ref and use
) -> ftd::p1::Result<(f64, Option<String>)> {
    let properties = ftd::component::resolve_properties_with_ref(line_number, properties, doc)?;
    match properties.get(name) {
        Some((ftd::Value::Decimal { value }, reference)) => {
            Ok((value.to_owned(), complete_reference(reference)))
        }
        Some((ftd::Value::Optional { data, kind }, reference)) => {
            if !matches!(kind, ftd::p2::Kind::Decimal { .. }) {
                return ftd::e2(
                    format!("expected decimal, found: {:?}", kind),
                    doc.name,
                    line_number,
                );
            };
            match data.as_ref() {
                None => {
                    let reference = match reference {
                        Some(reference) => reference,
                        None => {
                            return ftd::e2(
                                format!("expected decimal, found: {:?}", kind),
                                doc.name,
                                line_number,
                            )
                        }
                    };

                    if let Some(ftd::p2::Boolean::IsNotNull { value }) = condition {
                        match value {
                            ftd::PropertyValue::Reference { name, .. }
                            | ftd::PropertyValue::Variable { name, .. } => {
                                if name.eq(reference) {
                                    return Ok((
                                        0.0,
                                        complete_reference(&Some(reference.to_owned())),
                                    ));
                                }
                            }
                            _ => {}
                        }
                    }

                    // In case when the optional string is null.
                    // Return the empty string

                    Ok((0.0, complete_reference(&Some(reference.to_owned()))))
                }
                Some(ftd::Value::Decimal { value }) => {
                    Ok((value.to_owned(), complete_reference(reference)))
                }
                _ => {
                    return ftd::e2(
                        format!("expected decimal, found: {:?}", kind),
                        doc.name,
                        line_number,
                    )
                }
            }
        }
        Some((ftd::Value::None { kind }, reference)) if condition.is_some() => {
            let kind = kind.inner();
            if !matches!(kind, ftd::p2::Kind::Decimal { .. }) {
                return ftd::e2(
                    format!("expected integer, found: {:?}", kind),
                    doc.name,
                    line_number,
                );
            };

            let reference = match reference {
                Some(reference) => reference,
                None => {
                    return ftd::e2(
                        format!("expected integer, found: {:?}", kind),
                        doc.name,
                        line_number,
                    )
                }
            };
            if let Some(ftd::p2::Boolean::IsNotNull { value }) = condition {
                match value {
                    ftd::PropertyValue::Reference { name, .. }
                    | ftd::PropertyValue::Variable { name, .. } => {
                        if name.eq({
                            if let Some(reference) = reference.strip_prefix('@') {
                                reference
                            } else {
                                reference
                            }
                        }) {
                            return Ok((0.0, complete_reference(&Some(reference.to_owned()))));
                        }
                    }
                    _ => {}
                }
            }
            ftd::e2(
                format!("expected decimal, found: {:?}", kind),
                doc.name,
                line_number,
            )
        }
        Some(v) => ftd::e2(
            format!("expected decimal, found: {:?}", v),
            doc.name,
            line_number,
        ),
        None => ftd::e2(format!("'{}' not found", name), doc.name, line_number),
    }
}

pub fn string_and_source_and_ref(
    line_number: usize,
    name: &str,
    properties: &std::collections::BTreeMap<String, ftd::component::Property>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
) -> ftd::p1::Result<(String, ftd::TextSource, Option<String>)> {
    let properties = ftd::component::resolve_properties_with_ref(line_number, properties, doc)?;
    match properties.get(name) {
        Some((ftd::Value::String { text, source }, reference)) => {
            Ok((text.to_string(), source.to_owned(), reference.to_owned()))
        }
        Some((ftd::Value::Optional { data, kind }, reference)) => {
            let source = match kind {
                _ if matches!(kind, ftd::p2::Kind::String { .. }) => {
                    ftd::TextSource::from_kind(kind, doc.name, line_number)?
                }
                _ => {
                    return ftd::e2(
                        format!("expected string, found: {:?}", kind),
                        doc.name,
                        line_number,
                    )
                }
            };

            match data.as_ref() {
                None => {
                    let reference = match reference {
                        Some(reference) => reference,
                        None => {
                            return ftd::e2(
                                format!("expected string, found: {:?}", kind),
                                doc.name,
                                line_number,
                            )
                        }
                    };

                    if let Some(ftd::p2::Boolean::IsNotNull { value }) = condition {
                        match value {
                            ftd::PropertyValue::Reference { name, .. }
                            | ftd::PropertyValue::Variable { name, .. } => {
                                if name.eq(reference) {
                                    return Ok((
                                        "".to_string(),
                                        source,
                                        complete_reference(&Some(reference.to_owned())),
                                    ));
                                }
                            }
                            _ => {}
                        }
                    }

                    // In case when the optional string is null.
                    // Return the empty string

                    Ok((
                        "".to_string(),
                        source,
                        complete_reference(&Some(reference.to_owned())),
                    ))
                }
                Some(ftd::Value::String { text, source }) => Ok((
                    text.to_string(),
                    source.to_owned(),
                    complete_reference(reference),
                )),
                _ => {
                    return ftd::e2(
                        format!("expected string, found: {:?}", kind),
                        doc.name,
                        line_number,
                    )
                }
            }
        }
        Some((ftd::Value::None { kind }, reference)) if condition.is_some() => {
            let kind = kind.inner();
            let source = match kind {
                _ if matches!(kind, ftd::p2::Kind::String { .. }) => {
                    ftd::TextSource::from_kind(kind, doc.name, line_number)?
                }
                _ => {
                    return ftd::e2(
                        format!("expected string, found: {:?}", kind),
                        doc.name,
                        line_number,
                    )
                }
            };

            let reference = match reference {
                Some(reference) => reference,
                None => {
                    return ftd::e2(
                        format!("expected string, found: {:?}", kind),
                        doc.name,
                        line_number,
                    )
                }
            };
            if let Some(ftd::p2::Boolean::IsNotNull { value }) = condition {
                match value {
                    ftd::PropertyValue::Reference { name, .. }
                    | ftd::PropertyValue::Variable { name, .. } => {
                        if name.eq({
                            if let Some(reference) = reference.strip_prefix('@') {
                                reference
                            } else {
                                reference
                            }
                        }) {
                            return Ok((
                                "".to_string(),
                                source,
                                complete_reference(&Some(reference.to_owned())),
                            ));
                        }
                    }
                    _ => {}
                }
            }
            ftd::e2(
                format!("expected string, found: {:?}", kind),
                doc.name,
                line_number,
            )
        }
        Some(v) => ftd::e2(
            format!("expected string, found: {:?}", v),
            doc.name,
            line_number,
        ),
        None => ftd::e2(format!("'{}' not found", name), doc.name, line_number),
    }
}

// todo: remove this
pub fn complete_reference(reference: &Option<String>) -> Option<String> {
    let mut reference = reference.to_owned();
    if let Some(ref r) = reference {
        if let Some(name) = r.strip_prefix('@') {
            if name.eq("$loop$") {
                return None;
            } else if name.eq("MOUSE-IN") {
                reference = Some("$MOUSE-IN".to_string());
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
        Some(ftd::Value::Optional {
            data,
            kind: ftd::p2::Kind::String { .. },
        }) => match data.as_ref() {
            Some(ftd::Value::String { text: v, .. }) => Ok(Some(v.to_string())),
            None => Ok(None),
            v => ftd::e2(
                format!("expected string, for: `{}` found: {:?}", name, v),
                doc_id,
                line_number,
            ),
        },
        Some(v) => ftd::e2(
            format!("expected string, for: `{}` found: {:?}", name, v),
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
            format!("[{}] expected int, found1: {:?}", name, v),
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
        Some(ftd::Value::Optional {
            data,
            kind: ftd::p2::Kind::Integer { .. },
        }) => match data.as_ref() {
            Some(ftd::Value::Integer { value }) => Ok(Some(*value)),
            None => Ok(None),
            v => ftd::e2(
                format!("expected integer, for: `{}` found: {:?}", name, v),
                doc_id,
                line_number,
            ),
        },
        Some(v) => ftd::e2(
            format!("expected integer, found: {:?}", v),
            doc_id,
            line_number,
        ),
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
        Some(v) => ftd::e2(
            format!("expected int, found2: {:?}", v),
            doc_id,
            line_number,
        ),
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

pub fn bool_optional(
    name: &str,
    properties: &std::collections::BTreeMap<String, ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<Option<bool>> {
    match properties.get(name) {
        Some(ftd::Value::Boolean { value: v }) => Ok(Some(*v)),
        Some(ftd::Value::None {
            kind: ftd::p2::Kind::Boolean { .. },
        }) => Ok(None),
        Some(ftd::Value::Optional {
            data,
            kind: ftd::p2::Kind::Boolean { .. },
        }) => match data.as_ref() {
            Some(ftd::Value::Boolean { value: v }) => Ok(Some(*v)),
            None => Ok(None),
            v => ftd::e2(
                format!("expected string, for: `{}` found: {:?}", name, v),
                doc_id,
                line_number,
            ),
        },
        Some(v) => ftd::e2(
            format!("expected bool, found: {:?}", v),
            doc_id,
            line_number,
        ),
        None => Ok(None),
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
        Some(ftd::Value::Optional {
            data,
            kind: ftd::p2::Kind::Decimal { .. },
        }) => match data.as_ref() {
            Some(ftd::Value::Decimal { value: v }) => Ok(Some(*v)),
            None => Ok(None),
            v => ftd::e2(
                format!("expected string, for: `{}` found: {:?}", name, v),
                doc_id,
                line_number,
            ),
        },
        Some(v) => ftd::e2(
            format!("expected decimal, found: {:?}", v),
            doc_id,
            line_number,
        ),
        None => Ok(None),
    }
}

pub(crate) fn get_string_container(local_container: &[usize]) -> String {
    local_container
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join(",")
}

pub(crate) fn get_doc_name_and_remaining(s: &str) -> ftd::p1::Result<(String, Option<String>)> {
    let mut part1 = "".to_string();
    let mut pattern_to_split_at = s.to_string();
    if let Some((p1, p2)) = s.split_once('#') {
        part1 = format!("{}#", p1);
        pattern_to_split_at = p2.to_string();
    }
    Ok(if pattern_to_split_at.contains('.') {
        let (p1, p2) = ftd::p2::utils::split(pattern_to_split_at, ".")?;
        (format!("{}{}", part1, p1), Some(p2))
    } else {
        (s.to_string(), None)
    })
}

pub fn split(name: String, split_at: &str) -> ftd::p1::Result<(String, String)> {
    if !name.contains(split_at) {
        return ftd::e2(format!("{} is not found in {}", split_at, name), "", 0);
    }
    let mut part = name.splitn(2, split_at);
    let part_1 = part.next().unwrap().trim();
    let part_2 = part.next().unwrap().trim();
    Ok((part_1.to_string(), part_2.to_string()))
}

pub fn reorder(
    p1: &[ftd::p1::Section],
    doc: &mut ftd::p2::TDoc,
    types: &mut std::collections::BTreeMap<String, ftd::p2::Kind>,
) -> ftd::p1::Result<(Vec<ftd::p1::Section>, Vec<String>)> {
    fn is_kernel_component(comp: String) -> bool {
        if ["ftd.row", "ftd.column", "ftd.scene", "ftd.grid"].contains(&comp.as_str()) {
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
                for (_, _, v) in sub_section.header.0.iter() {
                    if v.contains(':') {
                        let (name, _) = ftd::p2::utils::split(v.to_string(), ":")?;
                        if inserted.contains(&name) || k == &name {
                            continue;
                        }
                        reorder_component(
                            p1_map,
                            new_p1,
                            Some(name.to_string()),
                            inserted,
                            doc,
                            var_types,
                        )?;
                    }
                }
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
            for (_, _, v) in v.header.0.iter() {
                if v.contains(':') {
                    let (name, _) = ftd::p2::utils::split(v.to_string(), ":")?;
                    if inserted.contains(&name) || k == &name {
                        continue;
                    }
                    reorder_component(
                        p1_map,
                        new_p1,
                        Some(name.to_string()),
                        inserted,
                        doc,
                        var_types,
                    )?;
                }
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

    let mut p1_map: std::collections::BTreeMap<String, ftd::p1::Section> = Default::default();
    let mut inserted_p1 = vec![];
    let mut new_p1 = vec![];
    let mut list_or_var = vec![];
    let mut var_types = vec![];
    for (idx, p1) in p1.iter().enumerate() {
        let var_data =
            ftd::variable::VariableData::get_name_kind(&p1.name, doc, p1.line_number, &var_types);
        if p1.name == "import"
            || p1.name.starts_with("record ")
            || p1.name.starts_with("or-type ")
            || p1.name.starts_with("map ")
        {
            inserted_p1.push(idx);
            new_p1.push(p1.to_owned());
        }
        if let Some(type_) = p1.name.strip_prefix("type ") {
            let alias = match p1.caption {
                Some(ref val) => val,
                None => {
                    return ftd::e2(
                        format!("expected value in caption, found: `{:?}`", p1),
                        doc.name,
                        p1.line_number,
                    );
                }
            };
            let kind = ftd::p2::TDoc {
                name: doc.name,
                aliases: doc.aliases,
                bag: doc.bag,
                local_variables: &mut Default::default(),
                types,
            }
            .get_kind(p1.line_number, alias, type_)?;
            types.insert(type_.to_string(), kind);
            inserted_p1.push(idx);
            continue;
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
            if p1_map.contains_key(name) {
                return ftd::e2(
                    format!("{} is already declared", name),
                    doc.name,
                    p1.line_number,
                );
            }
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
        doc,
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

pub(crate) fn get_root_component_name(
    doc: &ftd::p2::TDoc,
    name: &str,
    line_number: usize,
) -> ftd::p1::Result<String> {
    let mut name = name.to_string();
    let mut root_name = name.to_string();
    while name != "ftd.kernel" {
        let component = doc.get_component(line_number, name.as_str())?;
        name = component.root;
        root_name = component.full_name;
    }
    Ok(root_name)
}

pub(crate) fn get_markup_child(
    sub: &ftd::p1::SubSection,
    doc: &ftd::p2::TDoc,
    arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
) -> ftd::p1::Result<ftd::ChildComponent> {
    let (sub_name, ref_name) = match sub.name.split_once(" ") {
        Some((sub_name, ref_name)) => (sub_name.trim(), ref_name.trim()),
        _ => return ftd::e2("the component should have name", doc.name, sub.line_number),
    };
    let sub_caption = if sub.caption.is_none() && sub.body_without_comment().is_none() {
        Some(ref_name.to_string())
    } else {
        sub.caption.clone()
    };
    let mut child = ftd::ChildComponent::from_p1(
        sub.line_number,
        sub_name,
        &sub.header,
        &sub_caption,
        &sub.body_without_comment(),
        doc,
        arguments,
    )?;
    child.root = format!("{} {}", child.root, ref_name);
    Ok(child)
}

pub fn structure_header_to_properties(
    s: &str,
    arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
    doc: &ftd::p2::TDoc,
    line_number: usize,
    p1: &ftd::p1::Header,
) -> ftd::p1::Result<std::collections::BTreeMap<String, ftd::component::Property>> {
    let (name, caption) = ftd::p2::utils::split(s.to_string(), ":")?;
    match doc.get_thing(line_number, &name) {
        Ok(ftd::p2::Thing::Component(c)) => ftd::component::read_properties(
            line_number,
            p1,
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

pub fn arguments_on_condition(
    condition: &ftd::p2::Boolean,
    line_number: usize,
    doc: &ftd::p2::TDoc,
) -> ftd::p1::Result<(std::collections::BTreeMap<String, ftd::Value>, bool)> {
    let mut arguments: std::collections::BTreeMap<String, ftd::Value> = Default::default();
    let mut is_visible = true;
    if let ftd::p2::Boolean::IsNotNull { ref value } = condition {
        match value {
            ftd::PropertyValue::Value { .. } => {}
            ftd::PropertyValue::Reference { name, kind }
            | ftd::PropertyValue::Variable { name, kind } => {
                if let ftd::p2::Kind::Optional { kind } = kind {
                    if doc.get_value(line_number, name).is_err() {
                        is_visible = false;
                        arguments.insert(
                            name.to_string(),
                            kind_to_value(kind, line_number, doc.name)?,
                        );
                    }
                } else {
                    return ftd::e2(
                        format!("expected optional kind, found: {:?}", kind),
                        doc.name,
                        line_number,
                    );
                }
            }
        }
    }
    return Ok((arguments, is_visible));

    fn kind_to_value(
        kind: &ftd::p2::Kind,
        line_number: usize,
        doc_id: &str,
    ) -> ftd::p1::Result<ftd::Value> {
        if let Ok(value) = kind.to_value(line_number, doc_id) {
            return Ok(value);
        }
        // todo implement for all the kind
        Ok(match kind {
            ftd::p2::Kind::String { .. } => ftd::Value::String {
                text: "".to_string(),
                source: ftd::TextSource::Header,
            },
            ftd::p2::Kind::Integer { .. } => ftd::Value::Integer { value: 0 },
            ftd::p2::Kind::Decimal { .. } => ftd::Value::Decimal { value: 0.0 },
            ftd::p2::Kind::Boolean { .. } => ftd::Value::Boolean { value: false },
            _ => {
                return ftd::e2(
                    format!(
                        "implemented for string, integer, decimal and boolean, found: {:?}",
                        kind
                    ),
                    doc_id,
                    line_number,
                )
            }
        })
    }
}
