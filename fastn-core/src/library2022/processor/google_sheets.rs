const GOOGLE_SHEET_API_BASE_URL: &str = "https://docs.google.com/a/google.com/spreadsheets/d";

static GOOGLE_SHEETS_ID_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"/spreadsheets/d/([a-zA-Z0-9-_]+)").unwrap());

pub fn extract_google_sheets_id(url: &str) -> Option<String> {
    if let Some(captures) = GOOGLE_SHEETS_ID_REGEX.captures(url) {
        if let Some(id) = captures.get(1) {
            return Some(id.as_str().to_string());
        }
    }

    None
}

pub fn generate_google_sheet_url(google_sheet_id: &str) -> String {
    format!(
        "{}/{}/gviz/tq?tqx=out:csv",
        GOOGLE_SHEET_API_BASE_URL, google_sheet_id,
    )
}

pub fn prepare_query_url(url: &str, query: &str) -> String {
    url::form_urlencoded::Serializer::new(url.to_string())
        .append_pair("tq", query)
        .finish()
}

pub fn parse_csv(
    csv: &str,
    doc_name: &str,
    line_number: usize,
) -> ftd::interpreter::Result<Vec<Vec<serde_json::Value>>> {
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    let mut result: Vec<Vec<serde_json::Value>> = vec![];
    for record in reader.records() {
        match record {
            Ok(r) => {
                let mut row: Vec<serde_json::Value> = vec![];
                for value in r.iter() {
                    row.push(serde_json::Value::String(value.to_string()));
                }
                result.push(row);
            }
            Err(e) => {
                return ftd::interpreter::utils::e2(
                    format!("Failed to parse result: {:?}", e),
                    doc_name,
                    line_number,
                )
            }
        }
    }
    Ok(result)
}

// Todo: Resolve variable

pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    db_config: &fastn_core::library2022::processor::sql::DatabaseConfig,
    _headers: ftd::ast::HeaderValues,
    query: &str,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let request_url = prepare_query_url(&db_config.db_url, query);

    let response = match fastn_core::http::http_get_str(&request_url).await {
        Ok(v) => v,
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("HTTP::get failed: {:?}", e),
                doc.name,
                value.line_number(),
            )
        }
    };

    let result = parse_csv(response.as_str(), doc.name, value.line_number());

    match result {
        Ok(result) => fastn_core::library2022::processor::sqlite::result_to_value(
            Ok(result),
            kind,
            doc,
            &value,
            fastn_core::library2022::processor::sql::STATUS_OK,
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
