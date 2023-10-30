pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &mut fastn_core::RequestConfig,
    document_id: &str,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    // TODO: document key should be optional

    let headers = match value.get_record(doc.name) {
        Ok(val) => val.2.to_owned(),
        Err(_e) => ftd::ast::HeaderValues::new(vec![]),
    };

    let path = headers
        .get_optional_string_by_key("file", doc.name, value.line_number())?
        .unwrap_or_else(|| document_id.to_string());

    let stage = headers
        .get_optional_string_by_key("stage", doc.name, value.line_number())?
        .unwrap_or_else(|| "ast".to_string());

    let file = req_config
        .get_file_and_package_by_id(path.as_str())
        .await
        .map_err(|e| ftd::interpreter::Error::ParseError {
            message: format!("Cannot get path: {} {:?}", path.as_str(), e),
            doc_id: document_id.to_string(),
            line_number: value.line_number(),
        })?;
    doc.from_json(
        &fastn_core::commands::query::get_ftd_json(&file, stage.as_str()).map_err(|e| {
            ftd::interpreter::Error::ParseError {
                message: format!("Cannot resolve json for path: {} {:?}", path.as_str(), e),
                doc_id: document_id.to_string(),
                line_number: value.line_number(),
            }
        })?,
        &kind,
        &value,
    )
}
