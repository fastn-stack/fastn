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
        .replace(['/', '.'], "_")
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
    string_needs_no_quotes: bool,
) -> ftd::html1::Result<Option<String>> {
    /*let field = match field {
        None if property_value.kind().is_ftd_length()
            || property_value.kind().is_ftd_resizing_fixed() =>
        {
            Some("value".to_string())
        }
        Some(a) => Some(a),
        None => None,
    };*/

    let value_string = if let Some(value_string) =
        property_value.to_string(doc, field, id, string_needs_no_quotes)?
    {
        value_string
    } else {
        return Ok(None);
    };

    Ok(Some(match pattern_with_eval {
        Some((p, eval)) => {
            let mut pattern = format!("`{}`.format(JSONstringify({}))", p, value_string);
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
    Ok(value.is_record(ftd::interpreter2::FTD_IMAGE_SRC)
        || value.is_record(ftd::interpreter2::FTD_COLOR)
        || value.is_or_type_variant(ftd::interpreter2::FTD_BACKGROUND_SOLID))
}

pub(crate) fn dependencies_from_property_value(
    property_value: &ftd::interpreter2::PropertyValue,
    doc: &ftd::interpreter2::TDoc,
) -> Vec<String> {
    if let Some(ref_name) = property_value.reference_name() {
        vec![ref_name.to_string()]
    } else if let Some(function_call) = property_value.get_function() {
        let mut result = vec![];
        for property_value in function_call.values.values() {
            result.extend(dependencies_from_property_value(property_value, doc));
        }
        result
    } else if property_value.is_value() && property_value.kind().is_ftd_length() {
        let value = property_value.value("", 0).unwrap();
        let fields = value.or_type_fields(doc, 0).unwrap();
        dependencies_from_property_value(
            fields.get(ftd::interpreter2::FTD_LENGTH_VALUE).unwrap(),
            doc,
        )
    } else if property_value.is_value() && property_value.kind().is_ftd_resizing_fixed() {
        let value = property_value.value("", 0).unwrap();
        let property_value = value
            .get_or_type(doc.name, property_value.line_number())
            .unwrap()
            .2;
        if property_value.is_value() && property_value.kind().is_ftd_length() {
            let value = property_value.value("", 0).unwrap();
            let fields = value.or_type_fields(doc, 0).unwrap();
            dependencies_from_property_value(
                fields.get(ftd::interpreter2::FTD_LENGTH_VALUE).unwrap(),
                doc,
            )
        } else {
            vec![]
        }
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
        string_needs_no_quotes: bool,
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
            } => value.to_string(doc, *line_number, field, id, string_needs_no_quotes)?,
            _ => None,
        })
    }
}

impl ftd::interpreter2::Value {
    // string_needs_no_quotes: for class attribute the value should be red-block not "red-block"
    pub(crate) fn to_string(
        &self,
        doc: &ftd::interpreter2::TDoc,
        line_number: usize,
        field: Option<String>,
        id: &str,
        string_needs_no_quotes: bool,
    ) -> ftd::html1::Result<Option<String>> {
        Ok(match self {
            ftd::interpreter2::Value::String { text } if !string_needs_no_quotes => {
                Some(format!("\"{}\"", text))
            }
            ftd::interpreter2::Value::String { text } if string_needs_no_quotes => {
                Some(text.to_string())
            }
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
                        string_needs_no_quotes,
                    )? {
                        v
                    } else {
                        continue;
                    };
                    values.push(v);
                }
                Some(format!("{:?}", values.join(", ")))
            }
            ftd::interpreter2::Value::Record { fields, .. }
                if field
                    .as_ref()
                    .map(|v| fields.contains_key(v))
                    .unwrap_or(false) =>
            {
                fields.get(&field.unwrap()).unwrap().to_string(
                    doc,
                    None,
                    id,
                    string_needs_no_quotes,
                )?
            }
            ftd::interpreter2::Value::OrType {
                value,
                variant,
                full_variant,
                ..
            } => {
                let value = value.to_string(doc, field, id, string_needs_no_quotes)?;
                match value {
                    Some(value) if variant.ne(ftd::interpreter2::FTD_RESIZING_FIXED) => {
                        if let Ok(pattern) = ftd::executor::Resizing::set_pattern_from_variant_str(
                            variant,
                            full_variant,
                            doc.name,
                            line_number,
                        ) {
                            Some(format!("`{}`.format(JSONstringify({}))", pattern, value))
                        } else {
                            Some(value)
                        }
                    }
                    Some(value) => Some(value),
                    None => None,
                }
            }
            ftd::interpreter2::Value::Record { fields, name } => {
                let mut values = vec![];
                for (k, v) in fields {
                    let value = if let Some(v) =
                        v.to_string(doc, field.clone(), id, string_needs_no_quotes)?
                    {
                        if let Ok(pattern) = ftd::executor::Length::set_pattern_from_variant_str(
                            name,
                            doc.name,
                            line_number,
                        ) {
                            format!("`{}`.format(JSONstringify({}))", pattern, v)
                        } else {
                            v
                        }
                    } else {
                        "null".to_string()
                    };
                    values.push(format!("\"{}\": {}", k, value));
                }

                Some(format!("{{{}}}", values.join(",")))
            }
            t => unimplemented!("{:?}", t),
        })
    }
}
