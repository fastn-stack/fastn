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
    match config.extra_data.get(name.as_str()) {
        Some(data) => doc.from_json(data, section),
        _ => match section.body {
            Some(ref b) => {
                doc.from_json(&serde_json::from_str::<serde_json::Value>(&b.1)?, section)
            }
            _ => Err(ftd::p1::Error::ParseError {
                message: format!("Value is not passed for {}", name),
                doc_id: doc.name.to_string(),
                line_number: section.line_number,
            }),
        },
    }
}
