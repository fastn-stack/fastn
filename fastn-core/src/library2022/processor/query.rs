pub async fn process(
    value: ftd_ast::VariableValue,
    kind: fastn_type::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &mut fastn_core::RequestConfig,
    preview_session_id: &Option<String>,
) -> ftd::interpreter::Result<fastn_type::Value> {
    // TODO: document key should be optional

    let headers = match value.get_record(doc.name) {
        Ok(val) => val.2.to_owned(),
        Err(_e) => ftd_ast::HeaderValues::new(vec![]),
    };

    let path = headers
        .get_optional_string_by_key("file", doc.name, value.line_number())?
        .unwrap_or_else(|| req_config.document_id.to_string());

    let stage = headers
        .get_optional_string_by_key("stage", doc.name, value.line_number())?
        .unwrap_or_else(|| "ast".to_string());

    let file = req_config
        .get_file_and_package_by_id(path.as_str(), preview_session_id)
        .await
        .map_err(|e| ftd::interpreter::Error::ParseError {
            message: format!("Cannot get path: {} {:?}", path.as_str(), e),
            doc_id: req_config.document_id.to_string(),
            line_number: value.line_number(),
        })?;
    doc.from_json(
        &fastn_core::commands::query::get_ftd_json(&file, stage.as_str()).map_err(|e| {
            ftd::interpreter::Error::ParseError {
                message: format!("Cannot resolve json for path: {} {:?}", path.as_str(), e),
                doc_id: req_config.document_id.to_string(),
                line_number: value.line_number(),
            }
        })?,
        &kind,
        &value,
    )
}
