pub(crate) fn get_p1_data(
    name: &str,
    value: &ftd_ast::VariableValue,
    doc_name: &str,
) -> ftd::interpreter::Result<(ftd_ast::HeaderValues, String)> {
    if let ftd_ast::VariableValue::String { value, .. } = value {
        return Ok((ftd_ast::HeaderValues::new(vec![]), value.clone()));
    }

    match value.get_record(doc_name) {
        Ok(val) => Ok((
            val.2.to_owned(),
            match val.3 {
                Some(b) => b.value.clone(),
                None => {
                    return ftd::interpreter::utils::e2(
                        format!(
                            "$processor$: `{name}` query is not specified in the processor body",
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

pub(crate) fn result_to_value(
    result: Vec<Vec<serde_json::Value>>,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    value: &ftd_ast::VariableValue,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    if kind.is_list() {
        doc.rows_to_value(result.as_slice(), &kind, value)
    } else {
        match result.len() {
            1 => doc.row_to_value(&result[0], &kind, value),
            0 if kind.is_optional() => Ok(ftd::interpreter::Value::Optional {
                data: Box::new(None),
                kind: ftd::interpreter::KindData::new(kind),
            }),
            len => ftd::interpreter::utils::e2(
                format!("Query returned {} rows, expected one row", len),
                doc.name,
                value.line_number(),
            ),
        }
    }
}

fn resolve_variable_from_doc(
    var: &str,
    doc: &ftd::interpreter::TDoc,
    line_number: usize,
) -> ftd::interpreter::Result<ft_sys_shared::SqliteRawValue> {
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

    Ok(value_to_bind(thing))
}

fn value_to_bind(v: ftd::interpreter::Value) -> ft_sys_shared::SqliteRawValue {
    match v {
        ftd::interpreter::Value::String { text } => ft_sys_shared::SqliteRawValue::Text(text),
        ftd::interpreter::Value::Integer { value } => ft_sys_shared::SqliteRawValue::Integer(value),
        ftd::interpreter::Value::Decimal { value } => ft_sys_shared::SqliteRawValue::Real(value),
        ftd::interpreter::Value::Optional { data, .. } => match data.as_ref() {
            Some(v) => value_to_bind(v.to_owned()),
            None => ft_sys_shared::SqliteRawValue::Null,
        },
        ftd::interpreter::Value::Boolean { value } => {
            ft_sys_shared::SqliteRawValue::Integer(value as i64)
        }
        _ => unimplemented!(), // Handle other types as needed
    }
}
fn resolve_variable_from_headers(
    var: &str,
    param_type: &str,
    doc: &ftd::interpreter::TDoc,
    headers: &ftd_ast::HeaderValues,
    line_number: usize,
) -> ftd::interpreter::Result<ft_sys_shared::SqliteRawValue> {
    let header = match headers.optional_header_by_name(var, doc.name, line_number)? {
        Some(v) => v,
        None => return Ok(ft_sys_shared::SqliteRawValue::Null),
    };

    if let ftd_ast::VariableValue::String { value, .. } = &header.value {
        if let Some(stripped) = value.strip_prefix('$') {
            return resolve_variable_from_doc(stripped, doc, line_number);
        }
    }

    let param_value = match (param_type, &header.value) {
        ("TEXT", ftd_ast::VariableValue::String { value, .. }) => {
            ft_sys_shared::SqliteRawValue::Text(value.clone())
        }
        ("INTEGER", ftd_ast::VariableValue::String { value, .. }) => {
            ft_sys_shared::SqliteRawValue::Integer(value.parse::<i64>().unwrap())
        }
        ("REAL", ftd_ast::VariableValue::String { value, .. }) => {
            ft_sys_shared::SqliteRawValue::Real(value.parse::<f64>().unwrap())
        }
        _ => unimplemented!(), // Handle other types as needed
    };

    Ok(param_value)
}

fn resolve_param(
    param_name: &str,
    param_type: &str,
    doc: &ftd::interpreter::TDoc,
    headers: &ftd_ast::HeaderValues,
    line_number: usize,
) -> ftd::interpreter::Result<ft_sys_shared::SqliteRawValue> {
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

pub fn extract_named_parameters(
    query: &str,
    doc: &ftd::interpreter::TDoc,
    headers: ftd_ast::HeaderValues,
    line_number: usize,
) -> ftd::interpreter::Result<(String, Vec<ft_sys_shared::SqliteRawValue>)> {
    let mut params: Vec<ft_sys_shared::SqliteRawValue> = Vec::new();
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

                // todo: handle empty param_name
                params.push(resolve_param(
                    &param_name,
                    &param_type,
                    doc,
                    &headers,
                    line_number,
                )?);

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
        params.push(resolve_param(
            &param_name,
            &param_type,
            doc,
            &headers,
            line_number,
        )?);
    }

    Ok((query.to_string(), params))
}
