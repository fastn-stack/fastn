pub async fn process<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    tokio::task::block_in_place(move || processor_(value, kind, doc, config))
}

pub fn processor_<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    let (headers, body) = match value.get_record(doc.name) {
        Ok(val) => (val.2.to_owned(), val.3.to_owned()),
        Err(e) => return Err(e.into()),
    };

    let sqlite_database =
        match headers.get_optional_string_by_key("db", doc.name, value.line_number())? {
            Some(k) => k,
            None => {
                return ftd::interpreter2::utils::e2(
                    "`db` is not specified".to_string(),
                    doc.name,
                    value.line_number(),
                )
            }
        };

    let query = match body {
        Some(b) => b.value.as_str(),
        None => {
            return ftd::interpreter2::utils::e2(
                "$processor: `package-query` query is not specified in the processor body$"
                    .to_string(),
                doc.name,
                value.line_number(),
            )
        }
    };

    Ok(ftd::interpreter2::Value::Boolean { value: false })
}
