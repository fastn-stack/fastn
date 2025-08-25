use ftd::interpreter::PropertyValueExt;

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
                    );
                }
            },
        )),
        Err(e) => Err(e.into()),
    }
}

pub(crate) fn result_to_value(
    result: Vec<Vec<serde_json::Value>>,
    kind: fastn_resolved::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    value: &ftd_ast::VariableValue,
) -> ftd::interpreter::Result<fastn_resolved::Value> {
    if kind.is_list() {
        doc.rows_to_value(result.as_slice(), &kind, value)
    } else {
        match result.len() {
            1 => doc.row_to_value(&result[0], &kind, value),
            0 if kind.is_optional() => Ok(fastn_resolved::Value::Optional {
                data: Box::new(None),
                kind: fastn_resolved::KindData::new(kind),
            }),
            len => ftd::interpreter::utils::e2(
                format!("Query returned {len} rows, expected one row"),
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
            );
        }
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("${var} not found in the document: {e:?}"),
                doc.name,
                line_number,
            );
        }
    };

    Ok(value_to_bind(thing))
}

fn value_to_bind(v: fastn_resolved::Value) -> ft_sys_shared::SqliteRawValue {
    match v {
        fastn_resolved::Value::String { text } => ft_sys_shared::SqliteRawValue::Text(text),
        fastn_resolved::Value::Integer { value } => ft_sys_shared::SqliteRawValue::Integer(value),
        fastn_resolved::Value::Decimal { value } => ft_sys_shared::SqliteRawValue::Real(value),
        fastn_resolved::Value::Optional { data, .. } => match data.as_ref() {
            Some(v) => value_to_bind(v.to_owned()),
            None => ft_sys_shared::SqliteRawValue::Null,
        },
        fastn_resolved::Value::Boolean { value } => {
            ft_sys_shared::SqliteRawValue::Integer(value as i64)
        }
        _ => unimplemented!(), // Handle other types as needed
    }
}

fn resolve_variable_from_headers(
    var: &str,
    doc: &ftd::interpreter::TDoc,
    headers: &ftd_ast::HeaderValues,
    line_number: usize,
) -> ftd::interpreter::Result<ft_sys_shared::SqliteRawValue> {
    let header = match headers.optional_header_by_name(var, doc.name, line_number)? {
        Some(v) => v,
        None => return Ok(ft_sys_shared::SqliteRawValue::Null),
    };

    if let ftd_ast::VariableValue::String { value, .. } = &header.value
        && let Some(stripped) = value.strip_prefix('$') {
            return resolve_variable_from_doc(stripped, doc, line_number);
        }

    Ok(header_value_to_bind(&header.value))
}

fn header_value_to_bind(v: &ftd_ast::VariableValue) -> ft_sys_shared::SqliteRawValue {
    match v {
        ftd_ast::VariableValue::String { value, .. } => {
            ft_sys_shared::SqliteRawValue::Text(value.clone())
        }
        ftd_ast::VariableValue::Constant { value, .. } => {
            ft_sys_shared::SqliteRawValue::Text(value.clone())
        }
        ftd_ast::VariableValue::Optional { value, .. } => match value.as_ref() {
            Some(v) => header_value_to_bind(v),
            None => ft_sys_shared::SqliteRawValue::Null,
        },
        _ => unimplemented!(), // Handle other types as needed
    }
}

fn resolve_param(
    param_name: &str,
    doc: &ftd::interpreter::TDoc,
    headers: &ftd_ast::HeaderValues,
    line_number: usize,
) -> ftd::interpreter::Result<ft_sys_shared::SqliteRawValue> {
    resolve_variable_from_headers(param_name, doc, headers, line_number)
        .or_else(|_| resolve_variable_from_doc(param_name, doc, line_number))
}

pub fn extract_named_parameters(
    query: &str,
    doc: &ftd::interpreter::TDoc,
    headers: ftd_ast::HeaderValues,
    line_number: usize,
) -> ftd::interpreter::Result<(String, Vec<ft_sys_shared::SqliteRawValue>)> {
    let mut params: Vec<ft_sys_shared::SqliteRawValue> = Vec::new();

    let (query, args) =
        match fastn_utils::sql::extract_arguments(query, fastn_utils::sql::SQLITE_SUB) {
            Ok(v) => v,
            Err(e) => {
                return ftd::interpreter::utils::e2(
                    format!("Error parsing query: {e:?}"),
                    doc.name,
                    line_number,
                );
            }
        };

    for (param_name, _) in args {
        params.push(resolve_param(&param_name, doc, &headers, line_number)?);
    }

    Ok((query, params))
}
