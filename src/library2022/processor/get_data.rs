pub fn process<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    config: &fastn::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
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
        return Err(ftd::interpreter2::Error::ParseError {
            message: "Cannot pass both caption and body".to_string(),
            doc_id: doc.name.to_string(),
            line_number,
        });
    }

    if let Some(data) = config.extra_data.get(key.as_str()) {
        return doc.from_json(data, &kind, line_number);
    }

    if let Some(ref sitemap) = config.package.sitemap {
        let doc_id = config
            .current_document
            .clone()
            .map(|v| fastn::utils::id_to_path(v.as_str()))
            .unwrap_or_else(|| {
                doc.name
                    .to_string()
                    .replace(config.package.name.as_str(), "")
            })
            .trim()
            .replace(std::path::MAIN_SEPARATOR, "/");

        if let Some(extra_data) = sitemap.get_extra_data_by_id(doc_id.as_str()) {
            if let Some(data) = extra_data.get(key.as_str()) {
                return match kind {
                    ftd::interpreter2::Kind::Integer => {
                        let value = data.parse::<i64>().map_err(|e| {
                            ftd::interpreter2::Error::ParseError {
                                message: e.to_string(),
                                doc_id: doc.name.to_string(),
                                line_number,
                            }
                        })?;
                        doc.from_json(&value, &kind, line_number)
                    }
                    ftd::interpreter2::Kind::Decimal { .. } => {
                        let value = data.parse::<f64>().map_err(|e| {
                            ftd::interpreter2::Error::ParseError {
                                message: e.to_string(),
                                doc_id: doc.name.to_string(),
                                line_number,
                            }
                        })?;
                        doc.from_json(&value, &kind, line_number)
                    }
                    ftd::interpreter2::Kind::Boolean { .. } => {
                        let value = data.parse::<bool>().map_err(|e| {
                            ftd::interpreter2::Error::ParseError {
                                message: e.to_string(),
                                doc_id: doc.name.to_string(),
                                line_number,
                            }
                        })?;
                        doc.from_json(&value, &kind, line_number)
                    }
                    _ => doc.from_json(data, &kind, line_number),
                };
            }
        }
    }

    if let Ok(Some(path)) =
        headers.get_optional_string_by_key("file", doc.name, value.line_number())
    {
        match camino::Utf8Path::new(path.as_str()).extension() {
            Some(extension) => {
                if !extension.eq("json") {
                    return Err(ftd::interpreter2::Error::ParseError {
                        message: format!("only json file supported {}", path),
                        doc_id: doc.name.to_string(),
                        line_number,
                    });
                }
            }
            None => {
                return Err(ftd::interpreter2::Error::ParseError {
                    message: format!("file does not have any extension {}", path),
                    doc_id: doc.name.to_string(),
                    line_number,
                });
            }
        }

        let file = std::fs::read_to_string(path.as_str()).map_err(|_e| {
            ftd::interpreter2::Error::ParseError {
                message: format!("file path not found {}", path),
                doc_id: doc.name.to_string(),
                line_number,
            }
        })?;
        return doc.from_json(
            &serde_json::from_str::<serde_json::Value>(&file)?,
            &kind,
            line_number,
        );
    }

    if let Some(b) = body {
        return doc.from_json(
            &serde_json::from_str::<serde_json::Value>(b.value.as_str())?,
            &kind,
            line_number,
        );
    }

    let caption = match value.caption() {
        Some(ref caption) => caption.to_string(),
        None => {
            return Err(ftd::interpreter2::Error::ParseError {
                message: format!("caption name not passed for section: {}", section_name),
                doc_id: doc.name.to_string(),
                line_number,
            })
        }
    };

    if let Ok(val) = caption.parse::<bool>() {
        return doc.from_json(&serde_json::json!(val), &kind, line_number);
    }

    if let Ok(val) = caption.parse::<i64>() {
        return doc.from_json(&serde_json::json!(val), &kind, line_number);
    }

    if let Ok(val) = caption.parse::<f64>() {
        return doc.from_json(&serde_json::json!(val), &kind, line_number);
    }

    doc.from_json(&serde_json::json!(caption), &kind, line_number)
}
