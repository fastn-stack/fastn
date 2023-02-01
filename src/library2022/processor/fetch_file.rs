pub async fn fetch_files<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    config: &fastn::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    if !kind.is_string() {
        return ftd::interpreter2::utils::e2(
            format!("Expected kind is `string`, found: `{:?}`", kind),
            doc.name,
            value.line_number(),
        );
    }
    let headers = match value.get_record(doc.name) {
        Ok(val) => val.2.to_owned(),
        Err(_e) => ftd::ast::HeaderValues::new(vec![]),
    };
    let path = headers
        .get_optional_string_by_key("path", doc.name, value.line_number())?
        .ok_or(ftd::interpreter2::Error::ParseError {
            message: "`path` not found".to_string(),
            doc_id: doc.name.to_string(),
            line_number: value.line_number(),
        })?;

    Ok(ftd::interpreter2::Value::String {
        text: tokio::fs::read_to_string(config.root.join(path))
            .await
            .map_err(|v| ftd::interpreter2::Error::ParseError {
                message: v.to_string(),
                doc_id: doc.name.to_string(),
                line_number: value.line_number(),
            })?,
    })
}
