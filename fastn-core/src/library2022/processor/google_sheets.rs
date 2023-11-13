const GOOGLE_SHEET_API_BASE_URL: &str = "https://docs.google.com/a/google.com/spreadsheets/d";

static GOOGLE_SHEETS_ID_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"/spreadsheets/d/([a-zA-Z0-9-_]+)").unwrap());

static JSON_RESPONSE_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"^/\*O_o\*/\s*google.visualization.Query.setResponse\((.*?)\);$")
            .unwrap()
    });

pub(crate) fn extract_google_sheets_id(url: &str) -> Option<String> {
    if let Some(captures) = GOOGLE_SHEETS_ID_REGEX.captures(url) {
        if let Some(id) = captures.get(1) {
            return Some(id.as_str().to_string());
        }
    }

    None
}

fn extract_json(input: &str) -> ftd::interpreter::Result<Option<String>> {
    if let Some(captures) = JSON_RESPONSE_REGEX.captures(input) {
        match captures.get(1) {
            Some(m) => Ok(Some(m.as_str().to_string())),
            None => Ok(None),
        }
    } else {
        Ok(None)
    }
}

pub(crate) fn generate_google_sheet_url(google_sheet_id: &str) -> String {
    format!(
        "{}/{}/gviz/tq?tqx=out:json",
        GOOGLE_SHEET_API_BASE_URL, google_sheet_id,
    )
}

pub(crate) fn prepare_query_url(url: &str, query: &str) -> String {
    url::form_urlencoded::Serializer::new(url.to_string())
        .append_pair("tq", query)
        .finish()
}

#[derive(Debug, serde::Deserialize, PartialEq)]
pub(crate) struct DataColumn {
    id: String,
    label: String,
    r#type: String,
    pattern: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct DataRow {
    c: Vec<DataValue>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct DataValue {
    v: serde_json::Value,
    #[serde(default)]
    f: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct DataTable {
    cols: Vec<DataColumn>,
    rows: Vec<DataRow>,
    // #[serde(rename = "parsedNumHeaders")]
    // parsed_num_headers: usize,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct QueryResponse {
    // version: String,
    // #[serde(rename = "reqId")]
    // req_id: String,
    // status: String,
    // sig: String,
    table: DataTable,
}

pub(crate) fn rows_to_value(
    doc: &ftd::interpreter::TDoc<'_>,
    kind: &ftd::interpreter::Kind,
    value: &ftd::ast::VariableValue,
    rows: &[DataRow],
    columns: &[DataColumn],
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    Ok(match kind {
        ftd::interpreter::Kind::List { kind, .. } => {
            let mut data = vec![];
            for row in rows.iter() {
                data.push(
                    row_to_value(doc, kind, value, row, columns)?
                        .into_property_value(false, value.line_number()),
                );
            }

            ftd::interpreter::Value::List {
                data,
                kind: kind.to_owned().into_kind_data(),
            }
        }
        t => unimplemented!(
            "{:?} not yet implemented, line number: {}, doc: {}",
            t,
            value.line_number(),
            doc.name.to_string()
        ),
    })
}

fn row_to_record(
    doc: &ftd::interpreter::TDoc<'_>,
    name: &str,
    value: &ftd::ast::VariableValue,
    row: &DataRow,
    columns: &[DataColumn],
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let rec = doc.get_record(name, value.line_number())?;
    let rec_fields = rec.fields;
    let mut fields: ftd::Map<ftd::interpreter::PropertyValue> = Default::default();

    for field in rec_fields.iter() {
        let idx = match columns
            .iter()
            .position(|column| column.label.to_string().eq(&field.name))
        {
            Some(idx) => idx,
            None => {
                return ftd::interpreter::utils::e2(
                    format!("key not found: {}", field.name.as_str()),
                    doc.name,
                    value.line_number(),
                )
            }
        };

        fields.insert(
            field.name.to_string(),
            to_interpreter_value(
                doc,
                &field.kind.kind,
                &columns[idx],
                &row.c[idx],
                value.line_number(),
            )?
            .into_property_value(false, value.line_number()),
        );
    }

    Ok(ftd::interpreter::Value::Record {
        name: name.to_string(),
        fields,
    })
}

pub(crate) fn row_to_value(
    doc: &ftd::interpreter::TDoc<'_>,
    kind: &ftd::interpreter::Kind,
    value: &ftd::ast::VariableValue,
    row: &DataRow,
    columns: &[DataColumn],
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    if let ftd::interpreter::Kind::Record { name } = kind {
        return row_to_record(doc, name, value, row, columns);
    }

    if row.c.len() != 1 {
        return ftd::interpreter::utils::e2(
            format!("expected one column, found: {}", row.c.len()),
            doc.name,
            value.line_number(),
        );
    }

    to_interpreter_value(doc, kind, &columns[0], &row.c[0], value.line_number())
}

fn to_interpreter_value(
    doc: &ftd::interpreter::TDoc<'_>,
    kind: &ftd::interpreter::Kind,
    column: &DataColumn,
    data_value: &DataValue,
    line_number: usize,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    Ok(match kind {
        ftd::interpreter::Kind::String { .. } => ftd::interpreter::Value::String {
            text: match column.r#type.as_str() {
                "string" => match &data_value.v {
                    serde_json::Value::String(v) => v.to_string(),
                    _ => {
                        return ftd::interpreter::utils::e2(
                            format!("Can't parse to string, found: {}", &data_value.v),
                            doc.name,
                            line_number,
                        )
                    }
                },
                _ => match &data_value.f {
                    Some(v) => v.to_string(),
                    None => data_value.v.to_string(),
                },
            },
        },
        ftd::interpreter::Kind::Integer => ftd::interpreter::Value::Integer {
            value: match column.r#type.as_str() {
                "number" => match &data_value.v {
                    serde_json::Value::Number(n) => {
                        n.as_i64()
                            .ok_or_else(|| ftd::interpreter::Error::ParseError {
                                message: format!(
                                    "Can't parse to integer, found: {}",
                                    &data_value.v
                                ),
                                doc_id: doc.name.to_string(),
                                line_number,
                            })?
                    }
                    serde_json::Value::String(s) => {
                        s.parse::<i64>()
                            .map_err(|_| ftd::interpreter::Error::ParseError {
                                message: format!(
                                    "Can't parse to integer, found: {}",
                                    &data_value.v
                                ),
                                doc_id: doc.name.to_string(),
                                line_number,
                            })?
                    }
                    _ => {
                        return Err(ftd::interpreter::Error::ParseError {
                            message: format!("Can't parse to integer, found: {}", &data_value.v),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })
                    }
                },
                "datetime" => match &data_value.v {
                    serde_json::Value::String(s) => {
                        let parsed_date =
                            fastn_core::library2022::utils::ParsedDate::from_str(s.as_str())
                                .ok_or_else(|| ftd::interpreter::Error::ParseError {
                                    message: "Can't parse to datetime to integer".to_string(),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                })?;

                        parsed_date.timestamp
                    }
                    t => {
                        return Err(ftd::interpreter::Error::ParseError {
                            message: format!("Can't parse to datetime to integer, found: {}", t),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })
                    }
                },
                t => {
                    return Err(ftd::interpreter::Error::ParseError {
                        message: format!("Can't parse to integer, found: {t}"),
                        doc_id: doc.name.to_string(),
                        line_number,
                    })
                }
            },
        },
        ftd::interpreter::Kind::Decimal => ftd::interpreter::Value::Decimal {
            value: match &data_value.v {
                serde_json::Value::Number(n) => {
                    n.as_f64()
                        .ok_or_else(|| ftd::interpreter::Error::ParseError {
                            message: format!("Can't parse to decimal, found: {}", &data_value.v),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })?
                }
                serde_json::Value::String(s) => {
                    s.parse::<f64>()
                        .map_err(|_| ftd::interpreter::Error::ParseError {
                            message: format!("Can't parse to decimal, found: {}", &data_value.v),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })?
                }
                _ => {
                    return Err(ftd::interpreter::Error::ParseError {
                        message: format!("Can't parse to decimal, found: {}", &data_value.v),
                        doc_id: doc.name.to_string(),
                        line_number,
                    })
                }
            },
        },
        ftd::interpreter::Kind::Boolean => ftd::interpreter::Value::Boolean {
            value: match &data_value.v {
                serde_json::Value::Bool(n) => *n,
                serde_json::Value::String(s) => {
                    s.parse::<bool>()
                        .map_err(|_| ftd::interpreter::Error::ParseError {
                            message: format!("Can't parse to boolean, found: {}", &data_value.v),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })?
                }
                _ => {
                    return Err(ftd::interpreter::Error::ParseError {
                        message: format!("Can't parse to boolean, found: {}", &data_value.v),
                        doc_id: doc.name.to_string(),
                        line_number,
                    })
                }
            },
        },
        _ => unimplemented!(),
    })
}

pub(crate) fn result_to_value(
    query_response: QueryResponse,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    value: &ftd::ast::VariableValue,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    if kind.is_list() {
        rows_to_value(
            doc,
            &kind,
            value,
            &query_response.table.rows,
            &query_response.table.cols,
        )
    } else {
        match query_response.table.rows.len() {
            1 => row_to_value(
                doc,
                &kind,
                value,
                &query_response.table.rows[0],
                &query_response.table.cols,
            ),
            0 => ftd::interpreter::utils::e2(
                "Query returned no result, expected one row".to_string(),
                doc.name,
                value.line_number(),
            ),
            len => ftd::interpreter::utils::e2(
                format!("Query returned {} rows, expected one row", len),
                doc.name,
                value.line_number(),
            ),
        }
    }
}

pub(crate) fn parse_json(
    json: &str,
    doc_name: &str,
    line_number: usize,
    // ) -> ftd::interpreter::Result<Vec<Vec<serde_json::Value>>> {
) -> ftd::interpreter::Result<QueryResponse> {
    match serde_json::from_str::<QueryResponse>(json) {
        Ok(response) => Ok(response),
        Err(e) => ftd::interpreter::utils::e2(
            format!("Failed to parse query response: {:?}", e),
            doc_name,
            line_number,
        ),
    }
}

fn escape_string_value(value: &str) -> String {
    format!("\"{}\"", value.replace('\"', "\\\""))
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
        ftd::interpreter::Value::String { text } => escape_string_value(text.as_str()),
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
        ("STRING", ftd::ast::VariableValue::String { value, .. }) => escape_string_value(value),
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

    let json = match extract_json(&response)? {
        Some(json) => json,
        None => {
            return ftd::interpreter::utils::e2(
                "Invalid Query Response".to_string(),
                doc.name,
                value.line_number(),
            )
        }
    };

    let result = parse_json(json.as_str(), doc.name, value.line_number())?;

    result_to_value(result, kind, doc, &value)
}
