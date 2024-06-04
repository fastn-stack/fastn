use crate::library2022::processor::sqlite::result_to_value;

pub async fn process(
    value: ftd_ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &mut fastn_core::RequestConfig,
    q_kind: &str,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    // we can in future do a more fine-grained analysis if the response
    // is cacheable or not, say depending on HTTP Vary header, etc.
    config.response_is_cacheable = false;

    let (headers, query) = super::sqlite::get_p1_data(q_kind, &value, doc.name)?;
    let db = match headers.get_optional_string_by_key("db$", doc.name, value.line_number())? {
        Some(db) => db,
        None => fastn_core::migrations::get_db_url(&config.config),
    };

    let (query, params) = crate::library2022::processor::sqlite::extract_named_parameters(
        query.as_str(),
        doc,
        headers,
        value.line_number(),
    )?;

    if !params.is_empty() && q_kind == "sql-batch" {
        return ftd::interpreter::utils::e2(
            "Named parameters are not allowed in sql-batch queries",
            doc.name,
            value.line_number(),
        );
    }

    let ds = &config.config.ds;

    let res = match if q_kind == "sql-query" {
        ds.sql_query(db.as_str(), query.as_str(), params).await
    } else if q_kind == "sql-execute" {
        ds.sql_execute(db.as_str(), query.as_str(), params).await
    } else {
        ds.sql_batch(db.as_str(), query.as_str()).await
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
