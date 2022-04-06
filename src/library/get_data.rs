pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    let name = if let Some((_, name)) = section.name.rsplit_once(' ') {
        name.to_string()
    } else {
        section.name.to_string()
    };
    if let serde_json::Value::Object(o) = &config.extra_data {
        if let Some(data) = o.get(name.as_str()) {
            return doc.from_json(data, section);
        }
    }
    Err(ftd::p1::Error::ParseError {
        message: format!("Value is not passed for {}", name),
        doc_id: doc.name.to_string(),
        line_number: section.line_number,
    })
}
