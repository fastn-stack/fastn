pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::Config,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let (headers, query) =
        fastn_core::library2022::processor::sqlite::get_p1_data("package-data", &value, doc.name)?;

    fastn_core::library2022::utils::log_deprecation_warning(
        "`package-query` has been deprecated, use `sql` processor instead.",
    );

    let use_db = match headers.get_optional_string_by_key("use", doc.name, value.line_number()) {
        Ok(Some(k)) => Some(k),
        _ => match headers
            .get_optional_string_by_key("db", doc.name, value.line_number())
            .ok()
        {
            Some(k) => {
                fastn_core::library2022::utils::log_deprecation_warning(
                    "`db` header is deprecated, use `use` instead.",
                );
                k
            }
            None => None,
        },
    };

    let sqlite_database_path = if let Some(db_path) = use_db {
        let db_path = config.root.join(db_path.as_str());
        if !db_path.exists() {
            return ftd::interpreter::utils::e2(
                "`use` does not exist for package-query processor".to_string(),
                doc.name,
                value.line_number(),
            );
        }
        db_path
    } else {
        return ftd::interpreter::utils::e2(
            "Neither `use` nor `db` is specified for package-query processor".to_string(),
            doc.name,
            value.line_number(),
        );
    };

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
