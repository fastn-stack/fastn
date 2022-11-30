pub fn trim_all_lines(s: &str) -> String {
    use itertools::Itertools;

    s.split('\n').into_iter().map(|v| v.trim()).join("\n")
}

pub fn trim_start_once(s: &str, matches: &str) -> String {
    if let Some((_, p2)) = s.split_once(matches) {
        return p2.to_string();
    }
    s.to_string()
}

pub fn trim_end_once(s: &str, matches: &str) -> String {
    if let Some((p1, _)) = s.rsplit_once(matches) {
        return p1.to_string();
    }
    s.to_string()
}

pub fn trim_brackets(s: &str) -> String {
    if s.starts_with('(') && s.ends_with(')') {
        return s[1..s.len() - 1].to_string();
    }
    s.to_string()
}

pub(crate) fn name_with_id(s: &str, id: &str) -> String {
    if is_ftd_function(s) {
        return s.to_string();
    }
    format!("{}:{}", s, id)
}

pub(crate) fn is_ftd_function(s: &str) -> bool {
    s.starts_with("ftd#")
}

pub(crate) fn function_name_to_js_function(s: &str) -> String {
    s.replace('#', "__")
        .replace('-', "_")
        .replace(':', "___")
        .replace(',', "$")
        .replace('/', "_")
}

pub(crate) fn full_data_id(id: &str, data_id: &str) -> String {
    if data_id.trim().is_empty() {
        id.to_string()
    } else {
        format!("{}:{}", data_id, id)
    }
}

pub(crate) fn node_change_id(id: &str, attr: &str) -> String {
    format!("{}__{}", id, attr)
}

pub(crate) fn get_formatted_dep_string_from_property_value(
    id: &str,
    doc: &ftd::interpreter2::TDoc,
    property_value: &ftd::interpreter2::PropertyValue,
    pattern_with_eval: &Option<(String, bool)>,
    field: Option<String>,
) -> ftd::html1::Result<Option<String>> {
    let field = match field {
        None if property_value.kind().is_ftd_length() => Some("value".to_string()),
        Some(a) => Some(a),
        None => None,
    };

    let value_string = if let Some(value_string) = property_value.to_string(doc, field, id)? {
        value_string
    } else {
        return Ok(None);
    };

    Ok(Some(match pattern_with_eval {
        Some((p, eval)) => {
            let mut pattern = format!("`{}`.format({})", p, value_string);
            if *eval {
                pattern = format!("eval({})", pattern)
            }
            pattern
        }
        None => value_string,
    }))
}

pub(crate) fn get_condition_string(condition: &ftd::interpreter2::Expression) -> String {
    let node = condition
        .expression
        .update_node_with_variable_reference(&condition.references);
    let expression = ftd::html1::ExpressionGenerator.to_string(&node, true, &[]);
    format!(
        indoc::indoc! {"
                function(){{
                    {expression}
                }}()"
        },
        expression = expression.trim(),
    )
}

pub(crate) fn js_expression_from_list(
    expressions: Vec<(Option<String>, String)>,
    key: Option<&str>,
) -> String {
    let mut conditions = vec![];
    let mut default = None;
    for (condition, expression) in expressions {
        if let Some(condition) = condition {
            conditions.push(format!(
                indoc::indoc! {"
                        {if_exp}({condition}){{
                            {expression}
                        }}
                    "},
                if_exp = if conditions.is_empty() {
                    "if"
                } else {
                    "else if"
                },
                condition = condition,
                expression = expression.trim(),
            ));
        } else {
            default = Some(expression)
        }
    }

    let default = match default {
        Some(d) if conditions.is_empty() => d,
        Some(d) => format!("else {{{}}}", d),
        None if !conditions.is_empty() && key.is_some() => {
            format!("else {{ {} = null; }}", key.unwrap())
        }
        None => "".to_string(),
    };

    format!(
        indoc::indoc! {"
            {expressions}{default}
        "},
        expressions = conditions.join(" "),
        default = default,
    )
}

pub(crate) fn is_dark_mode_dependent(
    value: &ftd::interpreter2::PropertyValue,
    doc: &ftd::interpreter2::TDoc,
) -> ftd::html1::Result<bool> {
    let value = value.clone().resolve(doc, value.line_number())?;
    Ok(value.is_record("ftd#image-src"))
}

pub(crate) fn dependencies_from_property_value(
    value: &ftd::interpreter2::PropertyValue,
) -> Vec<String> {
    if let Some(ref_name) = value.reference_name() {
        vec![ref_name.to_string()]
    } else if let Some(function_call) = value.get_function() {
        let mut result = vec![];
        for property_value in function_call.values.values() {
            result.extend(dependencies_from_property_value(property_value));
        }
        result
    } else if value.is_value() && value.kind().is_ftd_length() {
        let value = value.value("", 0).unwrap();
        let fields = value.or_type_fields("", 0).unwrap();
        dependencies_from_property_value(fields.get(ftd::interpreter2::FTD_LENGTH_VALUE).unwrap())
    } else {
        vec![]
    }
}

impl ftd::interpreter2::PropertyValue {
    pub(crate) fn to_string(
        &self,
        doc: &ftd::interpreter2::TDoc,
        field: Option<String>,
        id: &str,
    ) -> ftd::html1::Result<Option<String>> {
        Ok(match self {
            ftd::interpreter2::PropertyValue::Reference { name, .. } => Some(format!(
                "resolve_reference(\"{}\", data){}",
                name,
                field
                    .map(|v| format!(".{}", v))
                    .unwrap_or_else(|| "".to_string())
            )),
            ftd::interpreter2::PropertyValue::FunctionCall(function_call) => {
                let action = serde_json::to_string(&ftd::html1::Action::from_function_call(
                    function_call,
                    id,
                    doc,
                )?)
                .unwrap();
                Some(format!(
                    "window.ftd.handle_function(event, '{}', '{}', this)",
                    id, action
                ))
            }
            ftd::interpreter2::PropertyValue::Value {
                value, line_number, ..
            } => value.to_string(doc, *line_number, field, id)?,
            _ => None,
        })
    }
}

impl ftd::interpreter2::Value {
    pub(crate) fn to_string(
        &self,
        doc: &ftd::interpreter2::TDoc,
        line_number: usize,
        field: Option<String>,
        id: &str,
    ) -> ftd::html1::Result<Option<String>> {
        Ok(match self {
            ftd::interpreter2::Value::String { text } => Some(format!("\"{}\"", text)),
            ftd::interpreter2::Value::Integer { value } => Some(value.to_string()),
            ftd::interpreter2::Value::Decimal { value } => Some(value.to_string()),
            ftd::interpreter2::Value::Boolean { value } => Some(value.to_string()),
            ftd::interpreter2::Value::List { data, .. } => {
                let mut values = vec![];
                for value in data {
                    let v = if let Some(v) = value.clone().resolve(doc, line_number)?.to_string(
                        doc,
                        value.line_number(),
                        None,
                        id,
                    )? {
                        v
                    } else {
                        continue;
                    };
                    values.push(v);
                }
                Some(format!("({:?})", values.join(",")))
            }
            ftd::interpreter2::Value::Record { fields, .. }
                if field
                    .as_ref()
                    .map(|v| fields.contains_key(v))
                    .unwrap_or(false) =>
            {
                fields
                    .get(&field.unwrap())
                    .unwrap()
                    .to_string(doc, None, id)?
            }
            ftd::interpreter2::Value::OrType { fields, .. }
                if field
                    .as_ref()
                    .map(|v| fields.contains_key(v))
                    .unwrap_or(false) =>
            {
                fields
                    .get(&field.unwrap())
                    .unwrap()
                    .to_string(doc, None, id)?
            }
            t => unimplemented!("{:?}", t),
        })
    }
}
