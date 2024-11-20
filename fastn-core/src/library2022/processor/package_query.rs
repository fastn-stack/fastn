pub async fn process(
    value: ftd_ast::VariableValue,
    kind: fastn_resolved::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<fastn_resolved::Value> {
    let (headers, query) =
        fastn_core::library2022::processor::sqlite::get_p1_data("package-data", &value, doc.name)?;

    fastn_core::warning!("`package-query` has been deprecated, use `sql` processor instead.");

    let sqlite_database =
        match headers.get_optional_string_by_key("db", doc.name, value.line_number())? {
            Some(k) => k,
            None => {
                return ftd::interpreter::utils::e2(
                    "`db` is not specified".to_string(),
                    doc.name,
                    value.line_number(),
                )
            }
        };

    let sqlite_database_path = req_config.config.ds.root().join(sqlite_database.as_str());

    if !req_config.config.ds.exists(&sqlite_database_path).await {
        return ftd::interpreter::utils::e2(
            "`db` does not exists for package-query processor".to_string(),
            doc.name,
            value.line_number(),
        );
    }

    let query_response = fastn_core::library2022::processor::sqlite::execute_query(
        &sqlite_database_path,
        query.as_str(),
        doc,
        headers,
        value.line_number(),
    )
    .await;

    match query_response {
        Ok(result) => fastn_core::library2022::processor::sqlite::result_to_value(
            Ok(result),
            kind,
            doc,
            &value,
            super::sql::STATUS_OK,
        ),
        Err(e) => fastn_core::library2022::processor::sqlite::result_to_value(
            Err(e.to_string()),
            kind,
            doc,
            &value,
            super::sql::STATUS_ERROR,
        ),
    }
}
