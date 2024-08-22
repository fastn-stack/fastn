pub async fn fetch_files(
    value: ftd_ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &fastn_core::RequestConfig,
    preview_session_id: &Option<String>,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    if !kind.is_string() {
        return ftd::interpreter::utils::e2(
            format!("Expected kind is `string`, found: `{:?}`", kind),
            doc.name,
            value.line_number(),
        );
    }
    let headers = match value.get_record(doc.name) {
        Ok(val) => val.2.to_owned(),
        Err(_e) => ftd_ast::HeaderValues::new(vec![]),
    };
    let path = headers
        .get_optional_string_by_key("path", doc.name, value.line_number())?
        .ok_or(ftd::interpreter::Error::ParseError {
            message: "`path` not found".to_string(),
            doc_id: doc.name.to_string(),
            line_number: value.line_number(),
        })?;

    Ok(ftd::interpreter::Value::String {
        text: req_config
            .config
            .ds
            .read_to_string(&req_config.config.ds.root().join(path), preview_session_id)
            .await
            .map_err(|v| ftd::interpreter::Error::ParseError {
                message: v.to_string(),
                doc_id: doc.name.to_string(),
                line_number: value.line_number(),
            })?,
    })
}
