use crate::library2022::processor::sqlite::result_to_value;

pub async fn process(
    value: ftd_ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::RequestConfig,
    is_query: bool,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let (headers, query) = super::sqlite::get_p1_data(
        if is_query { "sql-query" } else { "sql-execute" },
        &value,
        doc.name,
    )?;
    let db = match headers.get_optional_string_by_key("db$", doc.name, value.line_number())? {
        Some(db) => db,
        None => match config.config.ds.env("FASTN_DB_URL").await {
            Ok(db_url) => db_url,
            Err(_) => config
                .config
                .ds
                .env("DATABASE_URL")
                .await
                .unwrap_or_else(|_| "fastn.sqlite".to_string()),
        },
    };

    let (query, params) = crate::library2022::processor::sqlite::extract_named_parameters(
        query.as_str(),
        doc,
        headers,
        value.line_number(),
    )?;

    let ds = &config.config.ds;

    let res = match if is_query {
        ds.sql_query(db.as_str(), query.as_str(), params).await
    } else {
        ds.sql_execute(db.as_str(), query.as_str(), params).await
    } {
        Ok(v) => v,
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("Error executing query: {e:?}"),
                doc.name,
                value.line_number(),
            )
        }
    };

    result_to_value(res, kind, doc, &value)
}

