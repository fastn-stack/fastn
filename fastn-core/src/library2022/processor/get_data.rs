pub fn process(
    value: ftd_ast::VariableValue,
    kind: fastn_resolved::Kind,
    doc: &ftd::interpreter::TDoc,
    req_config: &mut fastn_core::RequestConfig,
) -> ftd::interpreter::Result<fastn_resolved::Value> {
    req_config.response_is_cacheable = false;

    let (section_name, headers, body, line_number) = match value.get_record(doc.name) {
        Ok(val) => (
            val.0.to_owned(),
            val.2.to_owned(),
            val.3.to_owned(),
            val.5.to_owned(),
        ),
        Err(e) => return Err(e.into()),
    };

    let key = match headers.get_optional_string_by_key("key", doc.name, line_number)? {
        Some(k) => k,
        None => section_name
            .rsplit_once(' ')
            .map(|(_, name)| name.to_string())
            .unwrap_or_else(|| section_name.to_string()),
    };

    if body.is_some() && value.caption().is_some() {
        return Err(ftd::interpreter::Error::ParseError {
            message: "Cannot pass both caption and body".to_string(),
            doc_id: doc.name.to_string(),
            line_number,
        });
    }

    if let Some(data) = req_config.extra_data.get(key.as_str()) {
        return match kind {
            fastn_resolved::Kind::Integer => {
                let value2 =
                    data.parse::<i64>()
                        .map_err(|e| ftd::interpreter::Error::ParseError {
                            message: e.to_string(),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })?;
                doc.from_json(&value2, &kind, &value)
            }
            fastn_resolved::Kind::Decimal { .. } => {
                let value2 =
                    data.parse::<f64>()
                        .map_err(|e| ftd::interpreter::Error::ParseError {
                            message: e.to_string(),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })?;
                doc.from_json(&value2, &kind, &value)
            }
            fastn_resolved::Kind::Boolean { .. } => {
                let value2 =
                    data.parse::<bool>()
                        .map_err(|e| ftd::interpreter::Error::ParseError {
                            message: e.to_string(),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })?;
                doc.from_json(&value2, &kind, &value)
            }
            _ => doc.from_json(data, &kind, &value),
        };
    }

    if let Ok(Some(path)) =
        headers.get_optional_string_by_key("file", doc.name, value.line_number())
    {
        match camino::Utf8Path::new(path.as_str()).extension() {
            Some(extension) => {
                if !extension.eq("json") {
                    return Err(ftd::interpreter::Error::ParseError {
                        message: format!("only json file supported {}", path),
                        doc_id: doc.name.to_string(),
                        line_number,
                    });
                }
            }
            None => {
                return Err(ftd::interpreter::Error::ParseError {
                    message: format!("file does not have any extension {}", path),
                    doc_id: doc.name.to_string(),
                    line_number,
                });
            }
        }

        let file = std::fs::read_to_string(path.as_str()).map_err(|_e| {
            ftd::interpreter::Error::ParseError {
                message: format!("file path not found {}", path),
                doc_id: doc.name.to_string(),
                line_number,
            }
        })?;
        return doc.from_json(
            &serde_json::from_str::<serde_json::Value>(&file)?,
            &kind,
            &value,
        );
    }

    if let Some(b) = body {
        return doc.from_json(
            &serde_json::from_str::<serde_json::Value>(b.value.as_str())?,
            &kind,
            &value,
        );
    }

    let caption = match value.caption() {
        Some(ref caption) => caption.to_string(),
        None => {
            return Err(ftd::interpreter::Error::ParseError {
                message: format!("caption name not passed for section: {}", section_name),
                doc_id: doc.name.to_string(),
                line_number,
            });
        }
    };

    if let Ok(val) = caption.parse::<bool>() {
        return doc.from_json(&serde_json::json!(val), &kind, &value);
    }

    if let Ok(val) = caption.parse::<i64>() {
        return doc.from_json(&serde_json::json!(val), &kind, &value);
    }

    if let Ok(val) = caption.parse::<f64>() {
        return doc.from_json(&serde_json::json!(val), &kind, &value);
    }

    doc.from_json(&serde_json::json!(caption), &kind, &value)
}
