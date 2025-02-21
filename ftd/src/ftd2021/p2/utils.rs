/// returns key/value pair separated by the KV_SEPERATOR
pub fn split_once(
    line: &str,
    doc_id: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<(String, Option<String>)> {
    // Trim any section/subsection identifier from the beginning of the line
    let line = ftd::identifier::trim_section_subsection_identifier(line);

    let (before_kv_delimiter, after_kv_delimiter) = line
        .split_once(ftd::identifier::KV_SEPERATOR)
        .ok_or_else(|| ftd::ftd2021::p1::Error::NotFound {
            doc_id: doc_id.to_string(),
            line_number,
            key: format!(
                "\'{}\' not found while segregating kv in {}",
                ftd::identifier::KV_SEPERATOR,
                line
            ),
        })?;

    match (before_kv_delimiter, after_kv_delimiter) {
        (before, after) if after.trim().is_empty() => Ok((before.trim().to_string(), None)),
        (before, after) => Ok((before.trim().to_string(), Some(after.trim().to_string()))),
    }
}

pub fn parse_import(
    c: &Option<String>,
    doc_id: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<(String, String)> {
    let v = match c {
        Some(v) => v.trim(),
        None => {
            return ftd::ftd2021::p2::utils::e2(
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
            _ => ftd::ftd2021::p2::utils::e2(
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
            _ => ftd::ftd2021::p2::utils::e2("doc id must contain /", doc_id, line_number),
        };
    }

    if let Some((t, _)) = v.split_once('.') {
        return Ok((v.to_string(), t.to_string()));
    }

    Ok((v.to_string(), v.to_string()))
}

pub fn get_name<'b>(prefix: &str, s: &'b str, doc_id: &str) -> ftd::ftd2021::p1::Result<&'b str> {
    match s.split_once(' ') {
        Some((p1, p2)) => {
            if p1 != prefix {
                return ftd::ftd2021::p2::utils::e2(
                    format!("must start with {}", prefix),
                    doc_id,
                    0,
                );
                // TODO:
            }
            Ok(p2)
        }
        None => ftd::ftd2021::p2::utils::e2(
            format!("{} does not contain space (prefix={})", s, prefix),
            doc_id,
            0, // TODO
        ),
    }
}

pub fn boolean_and_ref(
    line_number: usize,
    name: &str,
    properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>, // todo: check the string_and_source_and_ref and use
) -> ftd::ftd2021::p1::Result<(bool, Option<String>)> {
    let properties =
        ftd::ftd2021::component::resolve_properties_with_ref(line_number, properties, doc)?;
    match properties.get(name) {
        Some((ftd::Value::Boolean { value }, reference)) => {
            Ok((value.to_owned(), complete_reference(reference)))
        }
        Some((ftd::Value::Optional { data, kind }, reference)) => {
            if !matches!(kind, ftd::ftd2021::p2::Kind::Boolean { .. })
                && !matches!(kind, ftd::ftd2021::p2::Kind::Element)
            {
                return ftd::ftd2021::p2::utils::e2(
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
                            return ftd::ftd2021::p2::utils::e2(
                                format!("expected boolean, found: {:?}", kind),
                                doc.name,
                                line_number,
                            )
                        }
                    };

                    if let Some(ftd::ftd2021::p2::Boolean::IsNotNull {
                        value:
                            ftd::PropertyValue::Reference { name, .. }
                            | ftd::PropertyValue::Variable { name, .. },
                    }) = condition
                    {
                        if name.eq(reference) {
                            return Ok((false, complete_reference(&Some(reference.to_owned()))));
                        }
                    }

                    // In case when the optional string is null.
                    // Return the empty string

                    Ok((false, complete_reference(&Some(reference.to_owned()))))
                }
                Some(ftd::Value::Boolean { value }) => {
                    Ok((value.to_owned(), complete_reference(reference)))
                }
                _ => ftd::ftd2021::p2::utils::e2(
                    format!("expected boolean, found: {:?}", kind),
                    doc.name,
                    line_number,
                ),
            }
        }
        Some((ftd::Value::None { kind }, reference)) if condition.is_some() => {
            let kind = kind.inner();
            if !matches!(kind, ftd::ftd2021::p2::Kind::Boolean { .. })
                && !matches!(kind, ftd::ftd2021::p2::Kind::Element)
            {
                return ftd::ftd2021::p2::utils::e2(
                    format!("expected boolean, found: {:?}", kind),
                    doc.name,
                    line_number,
                );
            };

            let reference = match reference {
                Some(reference) => reference,
                None => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected integer, found 7: {:?}", kind),
                        doc.name,
                        line_number,
                    )
                }
            };
            if let Some(ftd::ftd2021::p2::Boolean::IsNotNull {
                value:
                    ftd::PropertyValue::Reference { name, .. }
                    | ftd::PropertyValue::Variable { name, .. },
            }) = condition
            {
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
            ftd::ftd2021::p2::utils::e2(
                format!("expected boolean, found: {:?}", kind),
                doc.name,
                line_number,
            )
        }
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("expected boolean, found: {:?}", v),
            doc.name,
            line_number,
        ),
        None => ftd::ftd2021::p2::utils::e2(format!("'{}' not found", name), doc.name, line_number),
    }
}

pub fn integer_and_ref(
    line_number: usize,
    name: &str,
    properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>, // todo: check the string_and_source_and_ref and use
) -> ftd::ftd2021::p1::Result<(i64, Option<String>)> {
    let properties =
        ftd::ftd2021::component::resolve_properties_with_ref(line_number, properties, doc)?;
    match properties.get(name) {
        Some((ftd::Value::Integer { value }, reference)) => {
            Ok((value.to_owned(), complete_reference(reference)))
        }
        Some((ftd::Value::Optional { data, kind }, reference)) => {
            if !matches!(kind, ftd::ftd2021::p2::Kind::Integer { .. })
                && !matches!(kind, ftd::ftd2021::p2::Kind::Element)
            {
                return ftd::ftd2021::p2::utils::e2(
                    format!("expected integer, found 8: {:?}", kind),
                    doc.name,
                    line_number,
                );
            };
            match data.as_ref() {
                None => {
                    let reference = match reference {
                        Some(reference) => reference,
                        None => {
                            return ftd::ftd2021::p2::utils::e2(
                                format!("expected integer, found 9: {:?}", kind),
                                doc.name,
                                line_number,
                            )
                        }
                    };

                    if let Some(ftd::ftd2021::p2::Boolean::IsNotNull {
                        value:
                            ftd::PropertyValue::Reference { name, .. }
                            | ftd::PropertyValue::Variable { name, .. },
                    }) = condition
                    {
                        if name.eq(reference) {
                            return Ok((0, complete_reference(&Some(reference.to_owned()))));
                        }
                    }

                    // In case when the optional string is null.
                    // Return the empty string

                    Ok((0, complete_reference(&Some(reference.to_owned()))))
                }
                Some(ftd::Value::Integer { value }) => {
                    Ok((value.to_owned(), complete_reference(reference)))
                }
                _ => ftd::ftd2021::p2::utils::e2(
                    format!("expected integer, found 10: {:?}", kind),
                    doc.name,
                    line_number,
                ),
            }
        }
        Some((ftd::Value::None { kind }, reference)) if condition.is_some() => {
            let kind = kind.inner();
            if !matches!(kind, ftd::ftd2021::p2::Kind::Integer { .. })
                && !matches!(kind, ftd::ftd2021::p2::Kind::Element)
            {
                return ftd::ftd2021::p2::utils::e2(
                    format!("expected integer, found 11: {:?}", kind),
                    doc.name,
                    line_number,
                );
            };

            let reference = match reference {
                Some(reference) => reference,
                None => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected integer, found 1: {:?}", kind),
                        doc.name,
                        line_number,
                    )
                }
            };
            if let Some(ftd::ftd2021::p2::Boolean::IsNotNull {
                value:
                    ftd::PropertyValue::Reference { name, .. }
                    | ftd::PropertyValue::Variable { name, .. },
            }) = condition
            {
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
            ftd::ftd2021::p2::utils::e2(
                format!("expected integer, found 2: {:?}", kind),
                doc.name,
                line_number,
            )
        }
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("expected integer, found 3: {:?}", v),
            doc.name,
            line_number,
        ),
        None => ftd::ftd2021::p2::utils::e2(format!("'{}' not found", name), doc.name, line_number),
    }
}

pub fn decimal_and_ref(
    line_number: usize,
    name: &str,
    properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>, // todo: check the string_and_source_and_ref and use
) -> ftd::ftd2021::p1::Result<(f64, Option<String>)> {
    let properties =
        ftd::ftd2021::component::resolve_properties_with_ref(line_number, properties, doc)?;
    match properties.get(name) {
        Some((ftd::Value::Decimal { value }, reference)) => {
            Ok((value.to_owned(), complete_reference(reference)))
        }
        Some((ftd::Value::Optional { data, kind }, reference)) => {
            if !matches!(kind, ftd::ftd2021::p2::Kind::Decimal { .. })
                && !matches!(kind, ftd::ftd2021::p2::Kind::Element)
            {
                return ftd::ftd2021::p2::utils::e2(
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
                            return ftd::ftd2021::p2::utils::e2(
                                format!("expected decimal, found: {:?}", kind),
                                doc.name,
                                line_number,
                            )
                        }
                    };

                    if let Some(ftd::ftd2021::p2::Boolean::IsNotNull {
                        value:
                            ftd::PropertyValue::Reference { name, .. }
                            | ftd::PropertyValue::Variable { name, .. },
                    }) = condition
                    {
                        if name.eq(reference) {
                            return Ok((0.0, complete_reference(&Some(reference.to_owned()))));
                        }
                    }

                    // In case when the optional string is null.
                    // Return the empty string

                    Ok((0.0, complete_reference(&Some(reference.to_owned()))))
                }
                Some(ftd::Value::Decimal { value }) => {
                    Ok((value.to_owned(), complete_reference(reference)))
                }
                _ => ftd::ftd2021::p2::utils::e2(
                    format!("expected decimal, found: {:?}", kind),
                    doc.name,
                    line_number,
                ),
            }
        }
        Some((ftd::Value::None { kind }, reference)) if condition.is_some() => {
            let kind = kind.inner();
            if !matches!(kind, ftd::ftd2021::p2::Kind::Decimal { .. })
                && !matches!(kind, ftd::ftd2021::p2::Kind::Element)
            {
                return ftd::ftd2021::p2::utils::e2(
                    format!("expected integer, found 4: {:?}", kind),
                    doc.name,
                    line_number,
                );
            };

            let reference = match reference {
                Some(reference) => reference,
                None => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected integer, found 5: {:?}", kind),
                        doc.name,
                        line_number,
                    )
                }
            };
            if let Some(ftd::ftd2021::p2::Boolean::IsNotNull {
                value:
                    ftd::PropertyValue::Reference { name, .. }
                    | ftd::PropertyValue::Variable { name, .. },
            }) = condition
            {
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
            ftd::ftd2021::p2::utils::e2(
                format!("expected decimal, found: {:?}", kind),
                doc.name,
                line_number,
            )
        }
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("expected decimal, found: {:?}", v),
            doc.name,
            line_number,
        ),
        None => ftd::ftd2021::p2::utils::e2(format!("'{}' not found", name), doc.name, line_number),
    }
}

pub fn string_and_source_and_ref(
    line_number: usize,
    name: &str,
    properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>,
) -> ftd::ftd2021::p1::Result<(String, ftd::TextSource, Option<String>)> {
    let properties =
        ftd::ftd2021::component::resolve_properties_with_ref(line_number, properties, doc)?;
    match properties.get(name) {
        Some((ftd::Value::String { text, source }, reference)) => {
            Ok((text.to_string(), source.to_owned(), (*reference).to_owned()))
        }
        Some((ftd::Value::Optional { data, kind }, reference)) => {
            let source = match kind {
                _ if matches!(kind, ftd::ftd2021::p2::Kind::String { .. })
                    || matches!(kind, ftd::ftd2021::p2::Kind::Element) =>
                {
                    ftd::TextSource::from_kind(kind, doc.name, line_number)?
                }
                _ => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected string, found 1: {:?}", kind),
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
                            return ftd::ftd2021::p2::utils::e2(
                                format!("expected string, found 2: {:?}", kind),
                                doc.name,
                                line_number,
                            )
                        }
                    };

                    if let Some(ftd::ftd2021::p2::Boolean::IsNotNull {
                        value:
                            ftd::PropertyValue::Reference { name, .. }
                            | ftd::PropertyValue::Variable { name, .. },
                    }) = condition
                    {
                        if name.eq(reference) {
                            return Ok((
                                "".to_string(),
                                source,
                                complete_reference(&Some(reference.to_owned())),
                            ));
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
                _ => ftd::ftd2021::p2::utils::e2(
                    format!("expected string, found 3: {:?}", kind),
                    doc.name,
                    line_number,
                ),
            }
        }
        Some((ftd::Value::None { kind }, reference)) if condition.is_some() => {
            let kind = kind.inner();
            let source = match kind {
                _ if matches!(kind, ftd::ftd2021::p2::Kind::String { .. })
                    || matches!(kind, ftd::ftd2021::p2::Kind::Element) =>
                {
                    ftd::TextSource::from_kind(kind, doc.name, line_number)?
                }
                _ => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected string, found 4: {:?}", kind),
                        doc.name,
                        line_number,
                    )
                }
            };

            let reference = match reference {
                Some(reference) => reference,
                None => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected string, found 5: {:?}", kind),
                        doc.name,
                        line_number,
                    )
                }
            };
            if let Some(ftd::ftd2021::p2::Boolean::IsNotNull {
                value:
                    ftd::PropertyValue::Reference { name, .. }
                    | ftd::PropertyValue::Variable { name, .. },
            }) = condition
            {
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
            ftd::ftd2021::p2::utils::e2(
                format!("expected string, found 6: {:?}", kind),
                doc.name,
                line_number,
            )
        }
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("expected string, found 7: {:?}", v),
            doc.name,
            line_number,
        ),
        None => ftd::ftd2021::p2::utils::e2(format!("'{}' not found", name), doc.name, line_number),
    }
}

// todo: remove this
pub fn complete_reference(reference: &Option<String>) -> Option<String> {
    let mut reference = (*reference).to_owned();
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

pub fn record_and_ref(
    line_number: usize,
    name: &str,
    properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>,
) -> ftd::ftd2021::p1::Result<(ftd::Map<ftd::PropertyValue>, Option<String>)> {
    let properties =
        ftd::ftd2021::component::resolve_properties_with_ref(line_number, properties, doc)?;
    match properties.get(name) {
        Some((ftd::Value::Record { fields, .. }, reference)) => {
            Ok((fields.to_owned(), (*reference).to_owned()))
        }
        Some((ftd::Value::Optional { data, kind }, reference)) => {
            if !matches!(kind, ftd::ftd2021::p2::Kind::Record { .. })
                && !matches!(kind, ftd::ftd2021::p2::Kind::Element)
            {
                return ftd::ftd2021::p2::utils::e2(
                    format!("expected record, found: {:?}", kind),
                    doc.name,
                    line_number,
                );
            }

            match data.as_ref() {
                None => {
                    let reference = match reference {
                        Some(reference) => reference,
                        None => {
                            return ftd::ftd2021::p2::utils::e2(
                                format!("expected record, found: {:?}", kind),
                                doc.name,
                                line_number,
                            )
                        }
                    };

                    if let Some(ftd::ftd2021::p2::Boolean::IsNotNull {
                        value:
                            ftd::PropertyValue::Reference { name, .. }
                            | ftd::PropertyValue::Variable { name, .. },
                    }) = condition
                    {
                        if name.eq(reference) {
                            return Ok((
                                Default::default(),
                                complete_reference(&Some(reference.to_owned())),
                            ));
                        }
                    }

                    // In case when the optional string is null.
                    // Return the empty string

                    Ok((
                        Default::default(),
                        complete_reference(&Some(reference.to_owned())),
                    ))
                }
                Some(ftd::Value::Record { fields, .. }) => {
                    Ok((fields.to_owned(), complete_reference(reference)))
                }
                _ => ftd::ftd2021::p2::utils::e2(
                    format!("expected record, found: {:?}", kind),
                    doc.name,
                    line_number,
                ),
            }
        }
        Some((ftd::Value::None { kind }, reference)) if condition.is_some() => {
            let kind = kind.inner();
            if !matches!(kind, ftd::ftd2021::p2::Kind::Record { .. })
                && !matches!(kind, ftd::ftd2021::p2::Kind::Element)
            {
                return ftd::ftd2021::p2::utils::e2(
                    format!("expected record, found: {:?}", kind),
                    doc.name,
                    line_number,
                );
            }

            let reference = match reference {
                Some(reference) => reference,
                None => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected record, found: {:?}", kind),
                        doc.name,
                        line_number,
                    )
                }
            };
            if let Some(ftd::ftd2021::p2::Boolean::IsNotNull {
                value:
                    ftd::PropertyValue::Reference { name, .. }
                    | ftd::PropertyValue::Variable { name, .. },
            }) = condition
            {
                if name.eq({
                    if let Some(reference) = reference.strip_prefix('@') {
                        reference
                    } else {
                        reference
                    }
                }) {
                    return Ok((
                        Default::default(),
                        complete_reference(&Some(reference.to_owned())),
                    ));
                }
            }
            ftd::ftd2021::p2::utils::e2(
                format!("expected record, found: {:?}", kind),
                doc.name,
                line_number,
            )
        }
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("expected record, found: {:?}", v),
            doc.name,
            line_number,
        ),
        None => ftd::ftd2021::p2::utils::e2(format!("'{}' not found", name), doc.name, line_number),
    }
}

#[allow(clippy::type_complexity)]
pub fn record_optional_with_ref(
    name: &str,
    properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<(Option<ftd::Map<ftd::PropertyValue>>, Option<String>)> {
    let properties =
        ftd::ftd2021::component::resolve_properties_with_ref(line_number, properties, doc)?;
    match properties.get(name) {
        Some((ftd::Value::Record { fields, .. }, reference)) => {
            Ok((Some(fields.to_owned()), (*reference).to_owned()))
        }
        Some((
            ftd::Value::None {
                kind: ftd::ftd2021::p2::Kind::Record { .. },
            },
            _,
        )) => Ok((None, None)),
        Some((ftd::Value::None { .. }, _)) => Ok((None, None)),
        Some((
            ftd::Value::Optional {
                data,
                kind: ftd::ftd2021::p2::Kind::Record { .. },
            },
            reference,
        )) => match data.as_ref() {
            Some(ftd::Value::Record { fields, .. }) => {
                Ok((Some(fields.to_owned()), (*reference).to_owned()))
            }
            None => Ok((None, None)),
            v => ftd::ftd2021::p2::utils::e2(
                format!("expected record, for: `{}` found: {:?}", name, v),
                doc.name,
                line_number,
            ),
        },
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("expected record, for: `{}` found: {:?}", name, v),
            doc.name,
            line_number,
        ),
        None => Ok((None, None)),
    }
}

pub fn record_optional(
    name: &str,
    properties: &ftd::Map<ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<Option<ftd::Map<ftd::PropertyValue>>> {
    match properties.get(name) {
        Some(ftd::Value::Record { fields, .. }) => Ok(Some(fields.to_owned())),
        Some(ftd::Value::None {
            kind: ftd::ftd2021::p2::Kind::Record { .. },
        }) => Ok(None),
        Some(ftd::Value::None { .. }) => Ok(None),
        Some(ftd::Value::Optional {
            data,
            kind: ftd::ftd2021::p2::Kind::Record { .. },
        }) => match data.as_ref() {
            Some(ftd::Value::Record { fields, .. }) => Ok(Some(fields.to_owned())),
            None => Ok(None),
            v => ftd::ftd2021::p2::utils::e2(
                format!("expected record, for: `{}` found: {:?}", name, v),
                doc_id,
                line_number,
            ),
        },
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("expected record, for: `{}` found: {:?}", name, v),
            doc_id,
            line_number,
        ),
        None => Ok(None),
    }
}

#[allow(clippy::type_complexity)]
pub fn string_optional_with_ref(
    name: &str,
    properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<(Option<String>, Option<String>)> {
    let properties =
        ftd::ftd2021::component::resolve_properties_with_ref(line_number, properties, doc)?;
    match properties.get(name) {
        Some((ftd::Value::String { text: v, .. }, reference)) => {
            Ok((Some(v.to_string()), (*reference).to_owned()))
        }
        Some((
            ftd::Value::None {
                kind: ftd::ftd2021::p2::Kind::String { .. },
            },
            _,
        )) => Ok((None, None)),
        Some((ftd::Value::None { .. }, _)) => Ok((None, None)),
        Some((
            ftd::Value::Optional {
                data,
                kind: ftd::ftd2021::p2::Kind::String { .. },
            },
            reference,
        )) => match data.as_ref() {
            Some(ftd::Value::String { text: v, .. }) => {
                Ok((Some(v.to_string()), (*reference).to_owned()))
            }
            None => Ok((None, (*reference).to_owned())),
            v => ftd::ftd2021::p2::utils::e2(
                format!("expected string, for: `{}` found: {:?}", name, v),
                doc.name,
                line_number,
            ),
        },
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("expected string, for: `{}` found: {:?}", name, v),
            doc.name,
            line_number,
        ),
        None => Ok((None, None)),
    }
}

pub fn string_optional(
    name: &str,
    properties: &ftd::Map<ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<Option<String>> {
    match properties.get(name) {
        Some(ftd::Value::String { text: v, .. }) => Ok(Some(v.to_string())),
        Some(ftd::Value::None {
            kind: ftd::ftd2021::p2::Kind::String { .. },
        }) => Ok(None),
        Some(ftd::Value::None { .. }) => Ok(None),
        Some(ftd::Value::Optional {
            data,
            kind: ftd::ftd2021::p2::Kind::String { .. },
        }) => match data.as_ref() {
            Some(ftd::Value::String { text: v, .. }) => Ok(Some(v.to_string())),
            None => Ok(None),
            v => ftd::ftd2021::p2::utils::e2(
                format!("expected string, for: `{}` found: {:?}", name, v),
                doc_id,
                line_number,
            ),
        },
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("expected string, for: `{}` found: {:?}", name, v),
            doc_id,
            line_number,
        ),
        None => Ok(None),
    }
}

pub fn string_list_optional(
    name: &str,
    properties: &ftd::Map<ftd::Value>,
    doc: &ftd::ftd2021::p2::TDoc,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<Option<Vec<String>>> {
    match properties.get(name) {
        Some(ftd::Value::List {
            data: list_values,
            kind: ftd::ftd2021::p2::Kind::String { .. },
        }) => {
            let mut string_vector: Vec<String> = vec![];
            for v in list_values {
                if let ftd::Value::String { text: str, .. } = v.resolve(line_number, doc)? {
                    string_vector.push(str);
                }
            }
            Ok(Some(string_vector))
        }
        Some(ftd::Value::None {
            kind: ftd::ftd2021::p2::Kind::List { .. },
        }) => Ok(None),
        Some(ftd::Value::None { .. }) => Ok(None),
        Some(ftd::Value::Optional {
            data: list_data,
            kind: ftd::ftd2021::p2::Kind::List { kind, .. },
        }) => {
            if kind.is_string() {
                return match list_data.as_ref() {
                    Some(ftd::Value::List {
                        data: list_values,
                        kind: ftd::ftd2021::p2::Kind::String { .. },
                    }) => {
                        let mut string_vector: Vec<String> = vec![];
                        for v in list_values.iter() {
                            if let ftd::Value::String { text: str, .. } =
                                v.resolve(line_number, doc)?
                            {
                                string_vector.push(str);
                            }
                        }
                        return Ok(Some(string_vector));
                    }
                    None => Ok(None),
                    v => ftd::ftd2021::p2::utils::e2(
                        format!("expected list of strings, for: `{}` found: {:?}", name, v),
                        doc.name,
                        line_number,
                    ),
                };
            }
            Ok(None)
        }
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("expected list of strings, for: `{}` found: {:?}", name, v),
            doc.name,
            line_number,
        ),
        None => Ok(None),
    }
}

pub fn string(
    name: &str,
    properties: &ftd::Map<ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<String> {
    match properties.get(name) {
        Some(ftd::Value::String { text: v, .. }) => Ok(v.to_string()),
        Some(ftd::Value::Optional {
            data,
            kind: ftd::ftd2021::p2::Kind::String { .. },
        }) => match data.as_ref() {
            Some(ftd::Value::String { text: v, .. }) => Ok(v.to_string()),
            v => ftd::ftd2021::p2::utils::e2(
                format!("expected string, for: `{}` found: {:?}", name, v),
                doc_id,
                line_number,
            ),
        },
        v => ftd::ftd2021::p2::utils::e2(
            format!("expected string, for: `{}` found: {:?}", name, v),
            doc_id,
            line_number,
        ),
    }
}

pub fn string_with_default(
    name: &str,
    def: &str,
    properties: &ftd::Map<ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<String> {
    match properties.get(name) {
        Some(ftd::Value::String { text: v, .. }) => Ok(v.to_string()),
        Some(ftd::Value::None {
            kind: ftd::ftd2021::p2::Kind::String { .. },
        }) => Ok(def.to_string()),
        Some(ftd::Value::None { .. }) => Ok(def.to_string()),
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("expected bool, found: {:?}", v),
            doc_id,
            line_number,
        ),
        None => Ok(def.to_string()),
    }
}

pub fn int(
    name: &str,
    properties: &ftd::Map<ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<i64> {
    match properties.get(name) {
        Some(ftd::Value::Integer { value: v, .. }) => Ok(*v),
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("[{}] expected int, found1: {:?}", name, v),
            doc_id,
            line_number,
        ),
        None => ftd::ftd2021::p2::utils::e2(format!("'{}' not found", name), doc_id, line_number),
    }
}

pub fn int_optional(
    name: &str,
    properties: &ftd::Map<ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<Option<i64>> {
    match properties.get(name) {
        Some(ftd::Value::Integer { value: v }) => Ok(Some(*v)),
        Some(ftd::Value::None {
            kind: ftd::ftd2021::p2::Kind::Integer { .. },
        }) => Ok(None),
        Some(ftd::Value::None { .. }) => Ok(None),
        Some(ftd::Value::Optional {
            data,
            kind: ftd::ftd2021::p2::Kind::Integer { .. },
        }) => match data.as_ref() {
            Some(ftd::Value::Integer { value }) => Ok(Some(*value)),
            None => Ok(None),
            v => ftd::ftd2021::p2::utils::e2(
                format!("expected integer, for: `{}` found: {:?}", name, v),
                doc_id,
                line_number,
            ),
        },
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("expected integer, found 6: {:?}", v),
            doc_id,
            line_number,
        ),
        None => Ok(None),
    }
}

pub fn int_with_default(
    name: &str,
    def: i64,
    properties: &ftd::Map<ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<i64> {
    match properties.get(name) {
        Some(ftd::Value::Integer { value: v }) => Ok(*v),
        Some(ftd::Value::None {
            kind: ftd::ftd2021::p2::Kind::Integer { .. },
        }) => Ok(def),
        Some(ftd::Value::None { .. }) => Ok(def),
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("expected int, found2: {:?}", v),
            doc_id,
            line_number,
        ),
        None => Ok(def),
    }
}

// pub fn elements(
//     name: &str,
//     properties: &ftd::Map<ftd::Value>,
// ) -> ftd_p1::Result<Vec<ftd::Element>> {
//     match properties.get(name) {
//         Some(ftd::Value::Elements(v)) => Ok((*v).clone()),
//         Some(v) => ftd::e(format!("expected elements, found: {:?}", v)),
//         None => ftd::e(format!("'{}' not found", name)),
//     }
// }

pub fn bool_with_default(
    name: &str,
    def: bool,
    properties: &ftd::Map<ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<bool> {
    match properties.get(name) {
        Some(ftd::Value::Boolean { value: v }) => Ok(*v),
        Some(ftd::Value::None {
            kind: ftd::ftd2021::p2::Kind::Boolean { .. },
        }) => Ok(def),
        Some(ftd::Value::None { .. }) => Ok(def),
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("expected bool, found: {:?}", v),
            doc_id,
            line_number,
        ),
        None => Ok(def),
    }
}

pub fn bool_(
    name: &str,
    properties: &ftd::Map<ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<bool> {
    match properties.get(name) {
        Some(ftd::Value::Boolean { value: v, .. }) => Ok(*v),
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("[{}] expected bool, found: {:?}", name, v),
            doc_id,
            line_number,
        ),
        None => ftd::ftd2021::p2::utils::e2(format!("'{}' not found", name), doc_id, line_number),
    }
}

pub fn bool_optional(
    name: &str,
    properties: &ftd::Map<ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<Option<bool>> {
    match properties.get(name) {
        Some(ftd::Value::Boolean { value: v }) => Ok(Some(*v)),
        Some(ftd::Value::None {
            kind: ftd::ftd2021::p2::Kind::Boolean { .. },
        }) => Ok(None),
        Some(ftd::Value::Optional {
            data,
            kind: ftd::ftd2021::p2::Kind::Boolean { .. },
        }) => match data.as_ref() {
            Some(ftd::Value::Boolean { value: v }) => Ok(Some(*v)),
            None => Ok(None),
            v => ftd::ftd2021::p2::utils::e2(
                format!("expected bool, for: `{}` found: {:?}", name, v),
                doc_id,
                line_number,
            ),
        },
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("expected bool, found: {:?}", v),
            doc_id,
            line_number,
        ),
        None => Ok(None),
    }
}

pub fn decimal(
    name: &str,
    properties: &ftd::Map<ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<f64> {
    match properties.get(name) {
        Some(ftd::Value::Decimal { value: v, .. }) => Ok(*v),
        Some(v) => ftd::ftd2021::p2::utils::e2(
            format!("[{}] expected Decimal, found: {:?}", name, v),
            doc_id,
            line_number,
        ),
        None => ftd::ftd2021::p2::utils::e2(format!("'{}' not found", name), doc_id, line_number),
    }
}

pub fn decimal_optional(
    name: &str,
    properties: &ftd::Map<ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<Option<f64>> {
    match properties.get(name) {
        Some(ftd::Value::Decimal { value: v }) => Ok(Some(*v)),
        Some(ftd::Value::None {
            kind: ftd::ftd2021::p2::Kind::Decimal { .. },
        }) => Ok(None),
        Some(ftd::Value::None { .. }) => Ok(None),
        Some(ftd::Value::Optional {
            data,
            kind: ftd::ftd2021::p2::Kind::Decimal { .. },
        }) => match data.as_ref() {
            Some(ftd::Value::Decimal { value: v }) => Ok(Some(*v)),
            None => Ok(None),
            v => ftd::ftd2021::p2::utils::e2(
                format!("expected decimal, for: `{}` found: {:?}", name, v),
                doc_id,
                line_number,
            ),
        },
        Some(v) => ftd::ftd2021::p2::utils::e2(
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

pub(crate) fn get_doc_name_and_remaining(
    s: &str,
) -> ftd::ftd2021::p1::Result<(String, Option<String>)> {
    let mut part1 = "".to_string();
    let mut pattern_to_split_at = s.to_string();
    if let Some((p1, p2)) = s.split_once('#') {
        part1 = format!("{}#", p1);
        pattern_to_split_at = p2.to_string();
    }
    Ok(if pattern_to_split_at.contains('.') {
        let (p1, p2) = ftd::ftd2021::p2::utils::split(pattern_to_split_at, ".")?;
        (format!("{}{}", part1, p1), Some(p2))
    } else {
        (s.to_string(), None)
    })
}

pub(crate) fn resolve_local_variable_name(
    line_number: usize,
    name: &str,
    container: &str,
    doc_name: &str,
    aliases: &ftd::Map<String>,
) -> ftd::ftd2021::p1::Result<String> {
    if name.contains('@') {
        return Ok(name.to_string());
    }
    let (part1, part2) = ftd::ftd2021::p2::utils::get_doc_name_and_remaining(name)?;
    Ok(if let Some(ref p2) = part2 {
        ftd::ftd2021::p2::utils::resolve_name(
            line_number,
            format!("{}@{}.{}", part1, container, p2).as_str(),
            doc_name,
            aliases,
        )?
    } else {
        ftd::ftd2021::p2::utils::resolve_name(
            line_number,
            format!("{}@{}", part1, container).as_str(),
            doc_name,
            aliases,
        )?
    })
}

pub fn resolve_name(
    line_number: usize,
    name: &str,
    doc_name: &str,
    aliases: &ftd::Map<String>,
) -> ftd::ftd2021::p1::Result<String> {
    if name.contains('#') {
        return Ok(name.to_string());
    }
    Ok(
        match ftd::ftd2021::p2::utils::split_module(name, doc_name, line_number)? {
            (Some(m), v, None) => match aliases.get(m) {
                Some(m) => format!("{}#{}", m, v),
                None => format!("{}#{}.{}", doc_name, m, v),
            },
            (Some(m), v, Some(c)) => match aliases.get(m) {
                Some(m) => format!("{}#{}.{}", m, v, c),
                None => format!("{}#{}.{}.{}", doc_name, m, v, c),
            },
            (None, v, None) => format!("{}#{}", doc_name, v),
            _ => unimplemented!(),
        },
    )
}

pub fn split(name: String, split_at: &str) -> ftd::ftd2021::p1::Result<(String, String)> {
    if !name.contains(split_at) {
        return ftd::ftd2021::p2::utils::e2(
            format!("{} is not found in {}", split_at, name),
            "",
            0,
        );
    }
    let mut part = name.splitn(2, split_at);
    let part_1 = part.next().unwrap().trim();
    let part_2 = part.next().unwrap().trim();
    Ok((part_1.to_string(), part_2.to_string()))
}

pub fn reorder(
    p1: &[ftd::ftd2021::p1::Section],
    doc: &ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<(Vec<ftd::ftd2021::p1::Section>, Vec<String>)> {
    fn is_kernel_component(comp: String) -> bool {
        if ["ftd.row", "ftd.column"].contains(&comp.as_str()) {
            return true;
        }
        false
    }

    fn reorder_component(
        p1_map: &ftd::Map<ftd::ftd2021::p1::Section>,
        new_p1: &mut Vec<ftd::ftd2021::p1::Section>,
        dependent_p1: Option<String>,
        inserted: &mut Vec<String>,
        doc: &ftd::ftd2021::p2::TDoc,
        var_types: &[String],
    ) -> ftd::ftd2021::p1::Result<()> {
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
                let var_data = ftd::ftd2021::variable::VariableData::get_name_kind(
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
                        let (name, _) = ftd::ftd2021::p2::utils::split(v.to_string(), ":")?;
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
                    let (name, _) = ftd::ftd2021::p2::utils::split(v.to_string(), ":")?;
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
            let var_data = ftd::ftd2021::variable::VariableData::get_name_kind(
                &v.name,
                doc,
                v.line_number,
                var_types,
            )?;
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

    let mut p1_map: ftd::Map<ftd::ftd2021::p1::Section> = Default::default();
    let mut inserted_p1 = vec![];
    let mut new_p1 = vec![];
    let mut list_or_var = vec![];
    let mut var_types = vec![];
    for (idx, p1) in p1.iter().enumerate() {
        let var_data = ftd::ftd2021::variable::VariableData::get_name_kind(
            &p1.name,
            doc,
            p1.line_number,
            &var_types,
        );
        if p1.name == "import"
            || p1.name.starts_with("record ")
            || p1.name.starts_with("or-type ")
            || p1.name.starts_with("map ")
        {
            inserted_p1.push(idx);
            new_p1.push(p1.to_owned());
        }
        if let Ok(ftd::ftd2021::variable::VariableData {
            type_: ftd::ftd2021::variable::Type::Variable,
            ref name,
            ..
        }) = var_data
        {
            inserted_p1.push(idx);
            new_p1.push(p1.to_owned());
            list_or_var.push(name.to_string());
        }

        if p1.name.starts_with("record ") {
            let name = ftd::ftd2021::p2::utils::get_name("record", &p1.name, "")?;
            var_types.push(name.to_string());
        }

        if p1.name.starts_with("or-type ") {
            let name = ftd::ftd2021::p2::utils::get_name("or-type", &p1.name, "")?;
            var_types.push(name.to_string());
            for s in &p1.sub_sections.0 {
                var_types.push(format!("{}.{}", name, s.name));
            }
        }

        if list_or_var.contains(&p1.name) {
            inserted_p1.push(idx);
            new_p1.push(p1.to_owned());
        }

        if let Ok(ftd::ftd2021::variable::VariableData {
            type_: ftd::ftd2021::variable::Type::Component,
            ref name,
            ..
        }) = var_data
        {
            if p1_map.contains_key(name) {
                return ftd::ftd2021::p2::utils::e2(
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
    doc: &ftd::ftd2021::p2::TDoc,
    name: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<String> {
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
    sub: &ftd::ftd2021::p1::SubSection,
    doc: &ftd::ftd2021::p2::TDoc,
    arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
) -> ftd::ftd2021::p1::Result<ftd::ChildComponent> {
    let (sub_name, ref_name) = match sub.name.split_once(' ') {
        Some((sub_name, ref_name)) => (sub_name.trim(), ref_name.trim()),
        _ => {
            return ftd::ftd2021::p2::utils::e2(
                "the component should have name",
                doc.name,
                sub.line_number,
            )
        }
    };
    let sub_caption = if sub.caption.is_none() && sub.body.is_none() {
        Some(ref_name.to_string())
    } else {
        sub.caption.clone()
    };
    let mut child = ftd::ChildComponent::from_p1(
        sub.line_number,
        sub_name,
        &sub.header,
        &sub_caption,
        &sub.body,
        doc,
        arguments,
    )?;
    child.root = format!("{} {}", child.root, ref_name);
    Ok(child)
}

pub fn structure_header_to_properties(
    s: &str,
    arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
    doc: &ftd::ftd2021::p2::TDoc,
    line_number: usize,
    p1: &ftd::ftd2021::p1::Header,
) -> ftd::ftd2021::p1::Result<ftd::Map<ftd::ftd2021::component::Property>> {
    let (name, caption) = ftd::ftd2021::p2::utils::split(s.to_string(), ":")?;
    match doc.get_thing(line_number, &name) {
        Ok(ftd::ftd2021::p2::Thing::Component(c)) => ftd::ftd2021::component::read_properties(
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
        t => ftd::ftd2021::p2::utils::e2(
            format!("expected component, found: {:?}", t),
            doc.name,
            line_number,
        ),
    }
}

pub fn arguments_on_condition(
    condition: &ftd::ftd2021::p2::Boolean,
    line_number: usize,
    doc: &ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<(ftd::Map<ftd::Value>, bool)> {
    let mut arguments: ftd::Map<ftd::Value> = Default::default();
    let mut is_visible = true;
    if let ftd::ftd2021::p2::Boolean::IsNotNull { value } = condition {
        match value {
            ftd::PropertyValue::Value { .. } => {}
            ftd::PropertyValue::Reference { name, kind }
            | ftd::PropertyValue::Variable { name, kind } => {
                if let ftd::ftd2021::p2::Kind::Optional { kind, .. } = kind {
                    if doc.get_value(line_number, name).is_err() {
                        is_visible = false;
                        arguments.insert(
                            name.to_string(),
                            kind_to_value(kind, line_number, doc.name)?,
                        );
                    }
                }
                // TODO: Check if it's parent variable then don't throw error else throw error
                /* else {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected optional kind, found: {} {:?}", name, kind),
                        doc.name,
                        line_number,
                    );
                }*/
            }
        }
    }
    return Ok((arguments, is_visible));

    fn kind_to_value(
        kind: &ftd::ftd2021::p2::Kind,
        line_number: usize,
        doc_id: &str,
    ) -> ftd::ftd2021::p1::Result<ftd::Value> {
        if let Ok(value) = kind.to_value(line_number, doc_id) {
            return Ok(value);
        }
        // todo implement for all the kind
        Ok(match kind {
            ftd::ftd2021::p2::Kind::String { .. } => ftd::Value::String {
                text: "".to_string(),
                source: ftd::TextSource::Header,
            },
            ftd::ftd2021::p2::Kind::Integer { .. } => ftd::Value::Integer { value: 0 },
            ftd::ftd2021::p2::Kind::Decimal { .. } => ftd::Value::Decimal { value: 0.0 },
            ftd::ftd2021::p2::Kind::Boolean { .. } => ftd::Value::Boolean { value: false },
            _ => {
                return ftd::ftd2021::p2::utils::e2(
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

pub fn split_module<'a>(
    id: &'a str,
    _doc_id: &str,
    _line_number: usize,
) -> ftd::ftd2021::p1::Result<(Option<&'a str>, &'a str, Option<&'a str>)> {
    match id.split_once('.') {
        Some((p1, p2)) => match p2.split_once('.') {
            Some((p21, p22)) => Ok((Some(p1), p21, Some(p22))),
            None => Ok((Some(p1), p2, None)),
        },
        None => Ok((None, id, None)),
    }
}

pub fn e2<T, S1>(m: S1, doc_id: &str, line_number: usize) -> ftd::ftd2021::p1::Result<T>
where
    S1: Into<String>,
{
    Err(ftd::ftd2021::p1::Error::ParseError {
        message: m.into(),
        doc_id: doc_id.to_string(),
        line_number,
    })
}

pub fn unknown_processor_error<T, S>(
    m: S,
    doc_id: String,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<T>
where
    S: Into<String>,
{
    Err(ftd::ftd2021::p1::Error::ParseError {
        message: m.into(),
        doc_id,
        line_number,
    })
}

/// return true if the component with the given name is a markdown component
/// otherwise returns false
pub fn is_markdown_component(
    doc: &ftd::ftd2021::p2::TDoc,
    name: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<bool> {
    let mut name = name.to_string();
    // check if the component is derived from ftd#text
    while !name.eq("ftd.kernel") {
        if doc.get_thing(line_number, name.as_str()).is_err() {
            return Ok(false);
        }
        match doc.get_thing(line_number, name.as_str())? {
            ftd::ftd2021::p2::Thing::Component(component) => {
                if name.eq("ftd#text") {
                    return Ok(true);
                }
                name = component.root;
            }
            _ => return Ok(false),
        }
    }
    Ok(false)
}

/// return true if the component with the given name is a container type component
/// otherwise returns false
pub fn is_container_component(
    doc: &ftd::ftd2021::p2::TDoc,
    name: &str,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<bool> {
    let mut name = name.to_string();
    // check if the component is derived from ftd#row or ftd#column
    while !name.eq("ftd.kernel") {
        if doc.get_thing(line_number, name.as_str()).is_err() {
            return Ok(false);
        }
        match doc.get_thing(line_number, name.as_str())? {
            ftd::ftd2021::p2::Thing::Component(component) => {
                if name.eq("ftd#row") || name.eq("ftd#column") {
                    return Ok(true);
                }
                name = component.root;
            }
            _ => return Ok(false),
        }
    }
    Ok(false)
}

/// return true if the section is an invoked component not a variable component
/// otherwise returns false
pub fn is_section_subsection_component(
    name: &str,
    doc: &ftd::ftd2021::p2::TDoc,
    var_types: &[String],
    line_number: usize,
) -> ftd::ftd2021::p1::Result<bool> {
    let var_data =
        ftd::ftd2021::variable::VariableData::get_name_kind(name, doc, line_number, var_types);

    if name.starts_with("record ")
        || name.starts_with("or-type ")
        || name.starts_with("map ")
        || name.starts_with("container")
    {
        return Ok(false);
    }

    if var_data.is_ok() {
        return Ok(false);
    }

    if doc.get_thing(line_number, name).is_ok() {
        if let ftd::ftd2021::p2::Thing::Component(_) = doc.get_thing(line_number, name)? {
            return Ok(true);
        }
    }

    Ok(false)
}

/// converts the document_name/document-full-id to document_id
/// and returns it as String
///
///
/// ## Examples
/// ```rust
/// # use ftd::ftd2021::p2::utils::convert_to_document_id;
///assert_eq!(convert_to_document_id("/bar/index.ftd/"), "/bar/");
///assert_eq!(convert_to_document_id("index.ftd"), "/");
///assert_eq!(convert_to_document_id("/foo/-/x/"), "/foo/");
///assert_eq!(convert_to_document_id("/fpm.dev/doc.txt"), "/fpm.dev/doc/");
///assert_eq!(convert_to_document_id("foo.png/"), "/foo/");
///assert_eq!(convert_to_document_id("README.md"), "/README/");
/// ```
pub fn convert_to_document_id(doc_name: &str) -> String {
    let doc_name = ftd::regex::EXT.replace_all(doc_name, "");

    // Discard document suffix if there
    // Also discard trailing index
    let document_id = doc_name
        .split_once("/-/")
        .map(|x| x.0)
        .unwrap_or_else(|| doc_name.as_ref())
        .trim_end_matches("index")
        .trim_matches('/');

    // In case if doc_id = index.ftd
    if document_id.is_empty() {
        return "/".to_string();
    }

    // Attach /{doc_id}/ before returning
    format!("/{}/", document_id)
}

#[cfg(test)]
mod test {
    macro_rules! p {
        ($s:expr, $id: expr_2021, $alias: expr_2021) => {
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
