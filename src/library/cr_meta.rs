pub async fn processor<'a>(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc<'a>,
    config: &fastn::Config,
) -> ftd::p1::Result<ftd::Value> {
    let cr_number = fastn::cr::get_cr_path_from_url(
        config.current_document.clone().unwrap_or_default().as_str(),
    )
    .ok_or_else(|| ftd::p1::Error::ParseError {
        message: format!("This is not CR Document `{:?}`", config.current_document),
        doc_id: doc.name.to_string(),
        line_number: section.line_number,
    })?;
    let cr_meta = fastn::cr::get_cr_meta(config, cr_number)
        .await
        .map_err(|e| ftd::p1::Error::ParseError {
            message: e.to_string(),
            doc_id: doc.name.to_string(),
            line_number: section.line_number,
        })?;
    doc.from_json(&cr_meta, section)
}
