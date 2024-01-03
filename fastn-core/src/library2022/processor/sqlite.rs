pub(crate) fn get_p1_data(
    name: &str,
    value: &ftd::ast::VariableValue,
    doc_name: &str,
) -> ftd::interpreter::Result<(ftd::ast::HeaderValues, String)> {
    match value.get_record(doc_name) {
        Ok(val) => Ok((
            val.2.to_owned(),
            match val.3 {
                Some(b) => b.value.clone(),
                None => {
                    return ftd::interpreter::utils::e2(
                        format!(
                            "$processor$: `{}` query is not specified in the processor body",
                            name
                        ),
                        doc_name,
                        value.line_number(),
                    )
                }
            },
        )),
        Err(e) => Err(e.into()),
    }
}

pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &fastn_core::RequestConfig,
    db_config: &fastn_core::library2022::processor::sql::DatabaseConfig,
    headers: ftd::ast::HeaderValues,
    query: &str,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let sqlite_database_path = req_config.config.ds.root().join(&db_config.db_url);

    // need the query params
    // question is they can be multiple
    // so lets say start with passing attributes from ftd file
    // db-<param-name1>: value
    // db-<param-name2>: value
    // for now they wil be ordered
    // select * from users where

    let query_response = execute_query(
        &sqlite_database_path,
        query,
        doc,
        headers,
        value.line_number(),
    )
    .await;

    match query_response {
        Ok(result) => result_to_value(Ok(result), kind, doc, &value, super::sql::STATUS_OK),
        Err(e) => result_to_value(
            Err(e.to_string()),
            kind,
            doc,
            &value,
            super::sql::STATUS_ERROR,
        ),
    }
}

pub(crate) fn result_to_value(
    result: Result<Vec<Vec<serde_json::Value>>, String>,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    value: &ftd::ast::VariableValue,
    status: usize,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    match result {
        Ok(result) => {
            dbg!(&result);
            if kind.is_list() {
                doc.rows_to_value(result.as_slice(), &kind, value)
            } else {
                match result.len() {
                    1 => doc.row_to_value(&result[0], &kind, value),
                    0 if kind.is_integer() => Ok(ftd::interpreter::Value::Integer {
                        value: status as i64,
                    }),
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
        Err(e) => match kind.get_name().as_str() {
            "integer" => Ok(ftd::interpreter::Value::Integer {
                value: status as i64,
            }),
            "string" => Ok(ftd::interpreter::Value::String { text: (e) }),
            _ => unimplemented!(),
        },
    }
}

fn resolve_variable_from_doc(
    var: &str,
    doc: &ftd::interpreter::TDoc,
    line_number: usize,
) -> ftd::interpreter::Result<Box<dyn rusqlite::ToSql>> {
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

    let param_value: Box<dyn rusqlite::ToSql> = match thing {
        ftd::interpreter::Value::String { text } => Box::new(text),
        ftd::interpreter::Value::Integer { value } => Box::new(value as i32),
        ftd::interpreter::Value::Decimal { value } => Box::new(value as f32),
        ftd::interpreter::Value::Boolean { value } => Box::new(value as i32),
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
) -> ftd::interpreter::Result<Box<dyn rusqlite::ToSql>> {
    let header = match headers.optional_header_by_name(var, doc.name, line_number)? {
        Some(v) => v,
        None => return Ok(Box::new(None::<Box<dyn rusqlite::ToSql>>)),
    };

    if let ftd::ast::VariableValue::String { value, .. } = &header.value {
        if let Some(stripped) = value.strip_prefix('$') {
            return Ok(Box::new(
                resolve_variable_from_doc(stripped, doc, line_number).map(Some)?,
            ));
        }
    }

    let param_value: Box<dyn rusqlite::ToSql> = match (param_type, &header.value) {
        ("TEXT", ftd::ast::VariableValue::String { value, .. }) => Box::new(value.clone()),
        ("INTEGER", ftd::ast::VariableValue::String { value, .. }) => {
            Box::new(value.parse::<i32>().unwrap())
        }
        ("REAL", ftd::ast::VariableValue::String { value, .. }) => {
            Box::new(value.parse::<f32>().unwrap())
        }
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
) -> ftd::interpreter::Result<Box<dyn rusqlite::ToSql>> {
    resolve_variable_from_headers(param_name, param_type, doc, headers, line_number)
        .or_else(|_| resolve_variable_from_doc(param_name, doc, line_number))
        .map(|v| Box::new(v) as Box<dyn rusqlite::ToSql>)
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

fn extract_named_parameters(
    query: &str,
    doc: &ftd::interpreter::TDoc,
    headers: ftd::ast::HeaderValues,
    line_number: usize,
) -> ftd::interpreter::Result<Vec<Box<dyn rusqlite::ToSql>>> {
    let mut params = Vec::new();
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

                params.push(Box::new(param_value) as Box<dyn rusqlite::ToSql>);

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
    }

    // Handle the last param if there was no trailing comma or space
    if [State::InsideParam, State::PushParam].contains(&state) && !param_name.is_empty() {
        let param_value = resolve_param(&param_name, &param_type, doc, &headers, line_number)?;
        params.push(Box::new(param_value) as Box<dyn rusqlite::ToSql>);
    }

    Ok(params)
}

pub(crate) async fn execute_query(
    database_path: &camino::Utf8PathBuf,
    query: &str,
    doc: &ftd::interpreter::TDoc<'_>,
    headers: ftd::ast::HeaderValues,
    line_number: usize,
) -> ftd::interpreter::Result<Vec<Vec<serde_json::Value>>> {
    let doc_name = doc.name;

    let conn = match rusqlite::Connection::open_with_flags(
        database_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
    ) {
        Ok(conn) => conn,
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("Failed to open `{}`: {:?}", database_path, e),
                doc_name,
                line_number,
            );
        }
    };

    let mut stmt = match conn.prepare(query) {
        Ok(v) => v,
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("Failed to prepare query: {:?}", e),
                doc_name,
                line_number,
            )
        }
    };

    let count = stmt.column_count();

    // let mut stmt = conn.prepare("SELECT * FROM test where name = :name")?;
    // let mut rows = stmt.query(rusqlite::named_params! { ":name": "one" })?

    // let mut stmt = conn.prepare("SELECT * FROM test where name = ?")?;
    // let mut rows = stmt.query([name])?;
    let params = extract_named_parameters(query, doc, headers, line_number)?;

    let mut rows = match stmt.query(rusqlite::params_from_iter(params)) {
        Ok(v) => v,
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("Failed to prepare query: {:?}", e),
                doc_name,
                line_number,
            )
        }
    };

    let mut result: Vec<Vec<serde_json::Value>> = vec![];
    loop {
        match rows.next() {
            Ok(None) => break,
            Ok(Some(r)) => {
                result.push(row_to_json(r, count, doc_name, line_number)?);
            }
            Err(e) => {
                return ftd::interpreter::utils::e2(
                    format!("Failed to execute query: {:?}", e),
                    doc_name,
                    line_number,
                )
            }
        }
    }
    Ok(result)
}

fn row_to_json(
    r: &rusqlite::Row,
    count: usize,
    doc_name: &str,
    line_number: usize,
) -> ftd::interpreter::Result<Vec<serde_json::Value>> {
    let mut row: Vec<serde_json::Value> = Vec::with_capacity(count);
    for i in 0..count {
        match r.get::<usize, rusqlite::types::Value>(i) {
            Ok(rusqlite::types::Value::Null) => row.push(serde_json::Value::Null),
            Ok(rusqlite::types::Value::Integer(i)) => row.push(serde_json::Value::Number(i.into())),
            Ok(rusqlite::types::Value::Real(i)) => row.push(serde_json::Value::Number(
                serde_json::Number::from_f64(i).unwrap(),
            )),
            Ok(rusqlite::types::Value::Text(i)) => row.push(serde_json::Value::String(i)),
            Ok(rusqlite::types::Value::Blob(_)) => {
                return ftd::interpreter::utils::e2(
                    format!("Query returned blob for column: {}", i),
                    doc_name,
                    line_number,
                );
            }
            Err(e) => {
                return ftd::interpreter::utils::e2(
                    format!("Failed to read response: {:?}", e),
                    doc_name,
                    line_number,
                );
            }
        }
    }
    Ok(row)
}
