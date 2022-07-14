pub async fn processor<'a>(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    processor_(section, doc, config)
        .await
        .map_err(|e| ftd::p1::Error::ParseError {
            message: e.to_string(),
            doc_id: doc.name.to_string(),
            line_number: section.line_number,
        })
}

pub fn processor_sync<'a>(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    let f = futures::executor::block_on(processor_(section, doc, config));

    f.map_err(|e| ftd::p1::Error::ParseError {
        message: e.to_string(),
        doc_id: doc.name.to_string(),
        line_number: section.line_number,
    })
}

pub async fn processor_<'a>(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc<'a>,
    config: &fpm::Config,
) -> fpm::Result<ftd::Value> {
    let path = section
        .header
        .string(doc.name, section.line_number, "path")?;
    Ok(ftd::Value::String {
        text: tokio::fs::read_to_string(config.root.join(path)).await?,
        source: ftd::TextSource::Body,
    })
}
