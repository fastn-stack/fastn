pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    let name = match section.header.string(doc.name, section.line_number, "key") {
        Ok(name) => name,
        _ => {
            if let Some((_, name)) = section.name.rsplit_once(' ') {
                name.to_string()
            } else {
                section.name.to_string()
            }
        }
    };

    if section.body.is_some() && section.caption.is_some() {
        return Err(ftd::p1::Error::ParseError {
            message: "Cannot pass both caption and body".to_string(),
            doc_id: doc.name.to_string(),
            line_number: section.line_number,
        });
    }

    if let Some(data) = config.extra_data.get(name.as_str()) {
        return doc.from_json(data, section);
    }

    if let Some(ref sitemap) = config.package.sitemap {
        let doc_id = config
            .current_document
            .clone()
            .map(|v| fpm::utils::id_to_path(v.as_str()))
            .unwrap_or_else(|| {
                doc.name
                    .to_string()
                    .replace(config.package.name.as_str(), "")
            })
            .trim()
            .replace(std::path::MAIN_SEPARATOR, "/");

        if let Some(extra_data) = sitemap.get_extra_data_by_id(doc_id.as_str()) {
            if let Some(data) = extra_data.get(name.as_str()) {
                let kind = doc.get_variable_kind(section)?;
                return match kind {
                    ftd::p2::Kind::Integer { .. } => {
                        let value =
                            data.parse::<i64>()
                                .map_err(|e| ftd::p1::Error::ParseError {
                                    message: e.to_string(),
                                    doc_id: doc.name.to_string(),
                                    line_number: section.line_number,
                                })?;
                        doc.from_json(&value, section)
                    }
                    ftd::p2::Kind::Decimal { .. } => {
                        let value =
                            data.parse::<f64>()
                                .map_err(|e| ftd::p1::Error::ParseError {
                                    message: e.to_string(),
                                    doc_id: doc.name.to_string(),
                                    line_number: section.line_number,
                                })?;
                        doc.from_json(&value, section)
                    }
                    ftd::p2::Kind::Boolean { .. } => {
                        let value =
                            data.parse::<bool>()
                                .map_err(|e| ftd::p1::Error::ParseError {
                                    message: e.to_string(),
                                    doc_id: doc.name.to_string(),
                                    line_number: section.line_number,
                                })?;
                        doc.from_json(&value, section)
                    }
                    _ => doc.from_json(data, section),
                };
            }
        }
    }

    if let Ok(path) = section.header.str(doc.name, section.line_number, "file") {
        match camino::Utf8Path::new(path).extension() {
            Some(extension) => {
                if !extension.eq("json") {
                    return Err(ftd::p1::Error::ParseError {
                        message: format!("only json file supported {}", path),
                        doc_id: doc.name.to_string(),
                        line_number: section.line_number,
                    });
                }
            }
            None => {
                return Err(ftd::p1::Error::ParseError {
                    message: format!("file does not have any extension {}", path),
                    doc_id: doc.name.to_string(),
                    line_number: section.line_number,
                });
            }
        }

        let file = std::fs::read_to_string(path).map_err(|_e| ftd::p1::Error::ParseError {
            message: format!("Value is not passed for {}", name),
            doc_id: doc.name.to_string(),
            line_number: section.line_number,
        })?;
        return doc.from_json(&serde_json::from_str::<serde_json::Value>(&file)?, section);
    }

    if let Some((_, ref b)) = section.body {
        return doc.from_json(&serde_json::from_str::<serde_json::Value>(b)?, section);
    }

    let caption = match section.caption {
        Some(ref caption) => caption,
        None => {
            return Err(ftd::p1::Error::ParseError {
                message: format!("Value is not passed for {}", name),
                doc_id: doc.name.to_string(),
                line_number: section.line_number,
            })
        }
    };

    if let Ok(val) = caption.parse::<bool>() {
        return doc.from_json(&serde_json::json!(val), section);
    }

    if let Ok(val) = caption.parse::<i64>() {
        return doc.from_json(&serde_json::json!(val), section);
    }

    if let Ok(val) = caption.parse::<f64>() {
        return doc.from_json(&serde_json::json!(val), section);
    }

    doc.from_json(&serde_json::json!(caption), section)
}
