#[derive(Debug, serde::Deserialize, PartialEq)]
pub(crate) struct DataColumn {
    id: String,
    label: String,
    // https://support.google.com/area120-tables/answer/9904372?hl=en
    r#type: String,
    pattern: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct DataRow {
    c: Vec<Option<DataValue>>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct DataValue {
    v: serde_json::Value,
    #[serde(default)]
    f: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct DataTable {
    #[serde(rename = "cols")]
    schema: Vec<DataColumn>,
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
    kind: &fastn_type::Kind,
    value: &ftd_ast::VariableValue,
    rows: &[DataRow],
    schema: &[DataColumn],
) -> ftd::interpreter::Result<fastn_type::Value> {
    Ok(match kind {
        fastn_type::Kind::List { kind, .. } => {
            let mut data = vec![];
            for row in rows.iter() {
                data.push(
                    row_to_value(doc, kind, value, row, schema)?
                        .into_property_value(false, value.line_number()),
                );
            }

            fastn_type::Value::List {
                data,
                kind: kind.to_owned().into_kind_data(),
            }
        }
        t => {
            return ftd::interpreter::utils::e2(
                format!("{:?} not yet implemented", t),
                doc.name,
                value.line_number(),
            )
        }
    })
}

fn row_to_record(
    doc: &ftd::interpreter::TDoc<'_>,
    name: &str,
    value: &ftd_ast::VariableValue,
    row: &DataRow,
    schema: &[DataColumn],
) -> ftd::interpreter::Result<fastn_type::Value> {
    let rec = doc.get_record(name, value.line_number())?;
    let rec_fields = rec.fields;
    let mut fields: ftd::Map<ftd::interpreter::PropertyValue> = Default::default();

    for field in rec_fields.iter() {
        let idx = match schema
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
                &schema[idx],
                &row.c[idx],
                value.caption(),
                value.record_name(),
                value.line_number(),
            )?
            .into_property_value(false, value.line_number()),
        );
    }

    Ok(fastn_type::Value::Record {
        name: name.to_string(),
        fields,
    })
}

fn row_to_value(
    doc: &ftd::interpreter::TDoc<'_>,
    kind: &fastn_type::Kind,
    value: &ftd_ast::VariableValue,
    row: &DataRow,
    schema: &[DataColumn],
) -> ftd::interpreter::Result<fastn_type::Value> {
    if let fastn_type::Kind::Record { name } = kind {
        return row_to_record(doc, name, value, row, schema);
    }

    if row.c.len() != 1 {
        return ftd::interpreter::utils::e2(
            format!("expected one column, found: {}", row.c.len()),
            doc.name,
            value.line_number(),
        );
    }

    to_interpreter_value(
        doc,
        kind,
        &schema[0],
        &row.c[0],
        value.caption(),
        value.record_name(),
        value.line_number(),
    )
}

fn to_interpreter_value(
    doc: &ftd::interpreter::TDoc<'_>,
    kind: &fastn_type::Kind,
    column: &DataColumn,
    data_value: &Option<DataValue>,
    _default_value: Option<String>,
    _record_name: Option<String>,
    line_number: usize,
) -> ftd::interpreter::Result<fastn_type::Value> {
    let val = match data_value {
        Some(v) => v,
        None => {
            if !kind.is_optional() {
                return ftd::interpreter::utils::e2(
                    format!("value cannot be null, expected value of kind: {:?}", &kind),
                    doc.name,
                    line_number,
                );
            } else {
                &DataValue {
                    v: serde_json::Value::Null,
                    f: None,
                }
            }
        }
    };

    Ok(match kind {
        // Available kinds: https://support.google.com/area120-tables/answer/9904372?hl=en
        fastn_type::Kind::String { .. } => fastn_type::Value::String {
            text: match column.r#type.as_str() {
                "string" => match &val.v {
                    serde_json::Value::String(v) => v.to_string(),
                    _ => {
                        return ftd::interpreter::utils::e2(
                            format!("Can't parse to string, found: {}", &val.v),
                            doc.name,
                            line_number,
                        )
                    }
                },
                _ => match &val.f {
                    Some(v) => v.to_string(),
                    None => val.v.to_string(),
                },
            },
        },
        fastn_type::Kind::Integer => fastn_type::Value::Integer {
            value: match column.r#type.as_str() {
                "number" => match &val.v {
                    serde_json::Value::Number(n) => {
                        n.as_f64().map(|f| f as i64).ok_or_else(|| {
                            ftd::interpreter::Error::ParseError {
                                message: format!("Can't parse to integer, found: {}", &val.v),
                                doc_id: doc.name.to_string(),
                                line_number,
                            }
                        })?
                    }
                    serde_json::Value::String(s) => {
                        s.parse::<i64>()
                            .map_err(|_| ftd::interpreter::Error::ParseError {
                                message: format!("Can't parse to integer, found: {}", &val.v),
                                doc_id: doc.name.to_string(),
                                line_number,
                            })?
                    }
                    _ => {
                        return Err(ftd::interpreter::Error::ParseError {
                            message: format!("Can't parse to integer, found: {}", &val.v),
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
        fastn_type::Kind::Decimal => fastn_type::Value::Decimal {
            value: match &val.v {
                serde_json::Value::Number(n) => {
                    n.as_f64()
                        .ok_or_else(|| ftd::interpreter::Error::ParseError {
                            message: format!("Can't parse to decimal, found: {}", &val.v),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })?
                }
                serde_json::Value::String(s) => {
                    s.parse::<f64>()
                        .map_err(|_| ftd::interpreter::Error::ParseError {
                            message: format!("Can't parse to decimal, found: {}", &val.v),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })?
                }
                _ => {
                    return Err(ftd::interpreter::Error::ParseError {
                        message: format!("Can't parse to decimal, found: {}", &val.v),
                        doc_id: doc.name.to_string(),
                        line_number,
                    })
                }
            },
        },
        fastn_type::Kind::Boolean => fastn_type::Value::Boolean {
            value: match &val.v {
                serde_json::Value::Bool(n) => *n,
                serde_json::Value::String(s) => {
                    s.parse::<bool>()
                        .map_err(|_| ftd::interpreter::Error::ParseError {
                            message: format!("Can't parse to boolean, found: {}", &val.v),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })?
                }
                _ => {
                    return Err(ftd::interpreter::Error::ParseError {
                        message: format!("Can't parse to boolean, found: {}", &val.v),
                        doc_id: doc.name.to_string(),
                        line_number,
                    })
                }
            },
        },
        fastn_type::Kind::Optional { kind, .. } => {
            let kind = kind.as_ref();
            match &val.v {
                serde_json::Value::Null => fastn_type::Value::Optional {
                    kind: kind.clone().into_kind_data(),
                    data: Box::new(None),
                },
                _ => to_interpreter_value(
                    doc,
                    kind,
                    column,
                    data_value,
                    _default_value,
                    _record_name,
                    line_number,
                )?,
            }
        }
        kind => {
            return ftd::interpreter::utils::e2(
                format!("{:?} not supported yet", kind),
                doc.name,
                line_number,
            )
        }
    })
}

fn result_to_value(
    query_response: QueryResponse,
    kind: fastn_type::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    value: &ftd_ast::VariableValue,
) -> ftd::interpreter::Result<fastn_type::Value> {
    if kind.is_list() {
        rows_to_value(
            doc,
            &kind,
            value,
            &query_response.table.rows,
            &query_response.table.schema,
        )
    } else {
        match query_response.table.rows.len() {
            1 => row_to_value(
                doc,
                &kind,
                value,
                &query_response.table.rows[0],
                &query_response.table.schema,
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

fn parse_json(
    json: &str,
    doc_name: &str,
    line_number: usize,
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

// Parser for processing the Google Visualization Query Language
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
        fastn_type::Value::String { text } => escape_string_value(text.as_str()),
        fastn_type::Value::Integer { value } => value.to_string(),
        fastn_type::Value::Decimal { value } => value.to_string(),
        fastn_type::Value::Boolean { value } => value.to_string(),
        v => {
            return ftd::interpreter::utils::e2(
                format!("kind {:?} is not supported yet.", v),
                doc.name,
                line_number,
            )
        }
    };

    Ok(param_value)
}

fn resolve_variable_from_headers(
    var: &str,
    param_type: &str,
    doc: &ftd::interpreter::TDoc,
    headers: &ftd_ast::HeaderValues,
    line_number: usize,
) -> ftd::interpreter::Result<String> {
    let header = match headers.optional_header_by_name(var, doc.name, line_number)? {
        Some(v) => v,
        None => return Ok("null".to_string()),
    };

    if let ftd_ast::VariableValue::String { value, .. } = &header.value {
        if let Some(stripped) = value.strip_prefix('$') {
            return resolve_variable_from_doc(stripped, doc, line_number);
        }
    }

    let param_value: String = match (param_type, &header.value) {
        ("STRING", ftd_ast::VariableValue::String { value, .. }) => escape_string_value(value),
        ("INTEGER", ftd_ast::VariableValue::String { value, .. })
        | ("DECIMAL", ftd_ast::VariableValue::String { value, .. })
        | ("BOOLEAN", ftd_ast::VariableValue::String { value, .. }) => value.to_string(),
        _ => {
            return ftd::interpreter::utils::e2(
                format!("kind {} is not supported yet.", param_type),
                doc.name,
                line_number,
            )
        }
    };

    Ok(param_value)
}

fn resolve_param(
    param_name: &str,
    param_type: &str,
    doc: &ftd::interpreter::TDoc,
    headers: &ftd_ast::HeaderValues,
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

pub(crate) fn parse_query(
    query: &str,
    doc: &ftd::interpreter::TDoc,
    headers: &ftd_ast::HeaderValues,
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
                    resolve_param(&param_name, &param_type, doc, headers, line_number)?;

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
        let param_value = resolve_param(&param_name, &param_type, doc, headers, line_number)?;
        output.push_str(&param_value);
    }

    Ok(output)
}

pub(crate) async fn process(
    ds: &fastn_ds::DocumentStore,
    value: ftd_ast::VariableValue,
    kind: fastn_type::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    db_config: &fastn_core::library2022::processor::sql::DatabaseConfig,
    headers: ftd_ast::HeaderValues,
    query: &str,
) -> ftd::interpreter::Result<fastn_type::Value> {
    let query = parse_query(query, doc, &headers, value.line_number())?;
    let sheet = &headers.get_optional_string_by_key("sheet", doc.name, value.line_number())?;
    let request_url =
        fastn_core::google_sheets::prepare_query_url(&db_config.db_url, query.as_str(), sheet);

    let response = match fastn_core::http::http_get_str(ds, &request_url).await {
        Ok(v) => v,
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("HTTP::get failed: {:?}", e),
                doc.name,
                value.line_number(),
            )
        }
    };

    let json = match fastn_core::google_sheets::extract_json(&response)? {
        Some(json) => json,
        None => {
            return ftd::interpreter::utils::e2(
                "Invalid Query Response. Please ensure that your Google Sheet is public."
                    .to_string(),
                doc.name,
                value.line_number(),
            )
        }
    };

    let result = parse_json(json.as_str(), doc.name, value.line_number())?;

    result_to_value(result, kind, doc, &value)
}
