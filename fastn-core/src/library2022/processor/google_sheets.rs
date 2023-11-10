const GOOGLE_SHEET_API_BASE_URL: &str = "https://docs.google.com/a/google.com/spreadsheets/d";

static GOOGLE_SHEETS_ID_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"/spreadsheets/d/([a-zA-Z0-9-_]+)").unwrap());

pub(crate) fn extract_google_sheets_id(url: &str) -> Option<String> {
    if let Some(captures) = GOOGLE_SHEETS_ID_REGEX.captures(url) {
        if let Some(id) = captures.get(1) {
            return Some(id.as_str().to_string());
        }
    }

    None
}

pub(crate) fn generate_google_sheet_url(google_sheet_id: &str) -> String {
    format!(
        "{}/{}/gviz/tq?tqx=out:csv",
        GOOGLE_SHEET_API_BASE_URL, google_sheet_id,
    )
}

pub(crate) fn prepare_query_url(url: &str, query: &str) -> String {
    url::form_urlencoded::Serializer::new(url.to_string())
        .append_pair("tq", query)
        .finish()
}

pub(crate) fn parse_csv(
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

fn resolve_variable_from_doc(
    var: &str,
    doc: &ftd::interpreter::TDoc,
    line_number: usize,
) -> ftd::interpreter::Result<String> {
    let thing = match doc.get_thing(var, line_number) {
        Ok(ftd::interpreter::Thing::Variable(v)) => v.value.resolve(doc, line_number)?,
        Ok(v) => {
            return ftd::interpreter::utils::e2(
                format!("{var} is not a variable, it's a {v:?}"),
                doc.name,
                line_number,
            )
        }
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("${var} not found in the document: {e:?}"),
                doc.name,
                line_number,
            )
        }
    };

    let param_value: String = match thing {
        ftd::interpreter::Value::String { text } => format!("\"{}\"", text),
        ftd::interpreter::Value::Integer { value } => value.to_string(),
        ftd::interpreter::Value::Decimal { value } => value.to_string(),
        ftd::interpreter::Value::Boolean { value } => value.to_string(),
        _ => unimplemented!(), // Handle other types as needed
    };

    Ok(param_value)
}

fn resolve_variable_from_headers(
    var: &str,
    param_type: &str,
    doc: &ftd::interpreter::TDoc,
    headers: &ftd::ast::HeaderValues,
    line_number: usize,
) -> ftd::interpreter::Result<String> {
    let header = match headers.optional_header_by_name(var, doc.name, line_number)? {
        Some(v) => v,
        None => return Ok("null".to_string()),
    };

    if let ftd::ast::VariableValue::String { value, .. } = &header.value {
        if let Some(stripped) = value.strip_prefix('$') {
            return resolve_variable_from_doc(stripped, doc, line_number);
        }
    }

    let param_value: String = match (param_type, &header.value) {
        ("STRING", ftd::ast::VariableValue::String { value, .. }) => format!("\"{}\"", value),
        ("INTEGER", ftd::ast::VariableValue::String { value, .. }) => value.to_string(),
        ("DECIMAL", ftd::ast::VariableValue::String { value, .. }) => value.to_string(),
        ("BOOLEAN", ftd::ast::VariableValue::String { value, .. }) => value.to_string(),
        _ => unimplemented!(), // Handle other types as needed
    };

    Ok(param_value)
}

fn resolve_param(
    param_name: &str,
    param_type: &str,
    doc: &ftd::interpreter::TDoc,
    headers: &ftd::ast::HeaderValues,
    line_number: usize,
) -> ftd::interpreter::Result<String> {
    resolve_variable_from_headers(param_name, param_type, doc, headers, line_number)
        .or_else(|_| resolve_variable_from_doc(param_name, doc, line_number))
}

#[derive(Debug, PartialEq)]
enum State {
    OutsideParam,
    InsideParam,
    InsideStringLiteral,
    InsideEscapeSequence(usize),
    ConsumeEscapedChar,
    StartTypeHint,
    InsideTypeHint,
    PushParam,
    ParseError(String),
}

fn parse_query(
    query: &str,
    doc: &ftd::interpreter::TDoc,
    headers: ftd::ast::HeaderValues,
    line_number: usize,
) -> ftd::interpreter::Result<String> {
    let mut output = String::new();
    let mut param_name = String::new();
    let mut param_type = String::new();
    let mut state = State::OutsideParam;

    for c in query.chars() {
        match state {
            State::OutsideParam => {
                if c == '$' {
                    state = State::InsideParam;
                    param_name.clear();
                    param_type.clear();
                } else if c == '"' {
                    state = State::InsideStringLiteral;
                }
            }
            State::InsideStringLiteral => {
                if c == '"' {
                    state = State::OutsideParam;
                } else if c == '\\' {
                    state = State::InsideEscapeSequence(0);
                }
            }
            State::InsideEscapeSequence(escape_count) => {
                if c == '\\' {
                    state = State::InsideEscapeSequence(escape_count + 1);
                } else {
                    state = if escape_count % 2 == 0 {
                        State::InsideStringLiteral
                    } else {
                        State::ConsumeEscapedChar
                    };
                }
            }
            State::ConsumeEscapedChar => {
                state = State::InsideStringLiteral;
            }
            State::StartTypeHint => {
                if c == ':' {
                    state = State::InsideTypeHint;
                } else {
                    state = State::ParseError("Type hint must start with `::`".to_string());
                }
            }
            State::InsideParam => {
                if c == ':' {
                    state = State::StartTypeHint;
                } else if c.is_alphanumeric() {
                    param_name.push(c);
                } else if c == ',' || c == ';' || c.is_whitespace() && !param_name.is_empty() {
                    state = State::PushParam;
                }
            }
            State::InsideTypeHint => {
                if c.is_alphanumeric() {
                    param_type.push(c);
                } else {
                    state = State::PushParam;
                }
            }
            State::PushParam => {
                state = State::OutsideParam;

                let param_value =
                    resolve_param(&param_name, &param_type, doc, &headers, line_number)?;

                output.push_str(&param_value);

                param_name.clear();
                param_type.clear();
            }
            State::ParseError(error) => {
                return Err(ftd::interpreter::Error::ParseError {
                    message: format!("Failed to parse SQL Query: {}", error),
                    doc_id: doc.name.to_string(),
                    line_number,
                });
            }
        }

        if ![
            State::InsideParam,
            State::PushParam,
            State::InsideTypeHint,
            State::StartTypeHint,
        ]
        .contains(&state)
        {
            output.push(c);
        }
    }

    // Handle the last param if there was no trailing comma or space
    if [State::InsideParam, State::PushParam, State::InsideTypeHint].contains(&state)
        && !param_name.is_empty()
    {
        let param_value = resolve_param(&param_name, &param_type, doc, &headers, line_number)?;
        output.push_str(&param_value);
    }

    Ok(output)
}

pub(crate) async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    db_config: &fastn_core::library2022::processor::sql::DatabaseConfig,
    headers: ftd::ast::HeaderValues,
    query: &str,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let query = parse_query(query, doc, headers, value.line_number())?;

    let request_url = prepare_query_url(&db_config.db_url, query.as_str());

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
            fastn_core::library2022::processor::sql::STATUS_ERROR,
        ),
    }
}
