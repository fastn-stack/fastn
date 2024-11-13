use ftd::interpreter::PropertyValueExt;
use serde::{Deserialize, Serialize};

pub fn process_typography_tokens(
    value: ftd_ast::VariableValue,
    kind: fastn_type::Kind,
    doc: &mut ftd::interpreter::TDoc,
) -> ftd::interpreter::Result<fastn_type::Value> {
    let line_number = value.line_number();
    let mut variable_name: Option<String> = None;

    let mut desktop_types: ftd::Map<TypeData> = ftd::Map::new();
    let mut mobile_types: ftd::Map<TypeData> = ftd::Map::new();

    extract_types(
        &value,
        doc,
        &mut variable_name,
        &mut desktop_types,
        &mut mobile_types,
        line_number,
    )?;

    let json_formatted_desktop_types =
        serde_json::to_string_pretty(&desktop_types).expect("Not a serializable type");
    let json_formatted_mobile_types =
        serde_json::to_string_pretty(&mobile_types).expect("Not a serializable type");

    let full_typography = format!(
        "{{\n\"{}-desktop\": {},\n\"{}-mobile\": {}\n}}",
        variable_name
            .clone()
            .unwrap_or_else(|| "Unnamed-typo".to_string())
            .as_str(),
        json_formatted_desktop_types,
        variable_name
            .unwrap_or_else(|| "Unnamed-typo".to_string())
            .as_str(),
        json_formatted_mobile_types
    );

    let response_json: serde_json::Value = serde_json::Value::String(full_typography);
    doc.from_json(&response_json, &kind, &value)
}

fn extract_types(
    value: &ftd_ast::VariableValue,
    doc: &mut ftd::interpreter::TDoc,
    variable_name: &mut Option<String>,
    desktop_types: &mut ftd::Map<TypeData>,
    mobile_types: &mut ftd::Map<TypeData>,
    line_number: usize,
) -> ftd::interpreter::Result<()> {
    let headers = match &value {
        ftd_ast::VariableValue::Record { headers, .. } => headers,
        _ => {
            return Err(ftd::interpreter::Error::InvalidKind {
                message: format!("Expected record of ftd.type-data found: {:?}", value),
                doc_id: doc.name.to_string(),
                line_number,
            })
        }
    };

    let variable = headers.get_by_key_optional("variable", doc.name, line_number)?;
    let name = headers.get_by_key_optional("name", doc.name, line_number)?;

    if let Some(name) = name {
        match &name.value {
            ftd_ast::VariableValue::String { value: hval, .. } => {
                *variable_name = Some(hval.to_string())
            }
            _ => {
                return Err(ftd::interpreter::Error::InvalidKind {
                    doc_id: doc.name.to_string(),
                    line_number,
                    message: format!("Expected string kind for name found: {:?}", variable_name),
                })
            }
        };
    }

    let variable_header = if let Some(variable) = variable {
        variable
    } else {
        return Err(ftd::interpreter::Error::InvalidKind {
            message: format!("`variable` header not found: {:?}", value),
            doc_id: doc.name.to_string(),
            line_number,
        });
    };

    let variable_header_value = match &variable_header.value {
        ftd_ast::VariableValue::String { value: hval, .. } => hval,
        t => {
            return Err(ftd::interpreter::Error::InvalidKind {
                message: format!(
                    "Expected `variable` header as key value pair: found: {:?}",
                    t
                ),
                doc_id: doc.name.to_string(),
                line_number,
            })
        }
    };

    if variable_name.is_none() {
        *variable_name = Some(variable_header_value.trim_start_matches('$').to_string());
    }

    let bag_entry = doc.resolve_name(variable_header_value);
    let bag_thing = doc.bag().get(bag_entry.as_str());

    let v = match bag_thing {
        Some(ftd::interpreter::Thing::Variable(v)) => v,
        t => {
            return Err(ftd::interpreter::Error::InvalidKind {
                message: format!("Expected Variable reference, found: {:?}", t),
                doc_id: doc.name.to_string(),
                line_number,
            })
        }
    };

    let fields = match &v.value {
        fastn_type::PropertyValue::Value {
            value: fastn_type::Value::Record { fields, .. },
            ..
        } => fields,
        t => {
            return Err(ftd::interpreter::Error::InvalidKind {
                message: format!(
                    "Expected variable of type record `ftd.color-scheme`: found {:?}",
                    t
                ),
                doc_id: doc.name.to_string(),
                line_number,
            })
        }
    };

    for (k, v) in fields.iter() {
        let resolved_responsive_value = v.clone().resolve(doc, v.line_number())?;

        extract_desktop_mobile_values(
            k.to_string(),
            &resolved_responsive_value,
            doc,
            desktop_types,
            mobile_types,
            v.line_number(),
        )?;
    }

    Ok(())
}

fn extract_desktop_mobile_values(
    type_name: String,
    responsive_value: &fastn_type::Value,
    doc: &ftd::interpreter::TDoc,
    desktop_types: &mut ftd::Map<TypeData>,
    mobile_types: &mut ftd::Map<TypeData>,
    line_number: usize,
) -> ftd::interpreter::Result<()> {
    if let fastn_type::Value::Record { fields, .. } = responsive_value {
        if responsive_value.is_record(ftd::interpreter::FTD_RESPONSIVE_TYPE) {
            if let Some(desktop_value) = fields.get("desktop") {
                let resolved_desktop_value = desktop_value
                    .clone()
                    .resolve(doc, desktop_value.line_number())?;
                extract_type_data(
                    type_name.clone(),
                    &resolved_desktop_value,
                    doc,
                    desktop_types,
                    line_number,
                )?;
            }

            if let Some(mobile_value) = fields.get("mobile") {
                let resolved_mobile_value = mobile_value
                    .clone()
                    .resolve(doc, mobile_value.line_number())?;
                extract_type_data(
                    type_name,
                    &resolved_mobile_value,
                    doc,
                    mobile_types,
                    line_number,
                )?;
            }
        } else {
            return Err(ftd::interpreter::Error::InvalidKind {
                message: format!(
                    "Expected value of type record `ftd.responsive-type`: found {:?}",
                    responsive_value,
                ),
                doc_id: doc.name.to_string(),
                line_number,
            });
        }
    }
    Ok(())
}

fn extract_type_data(
    type_name: String,
    type_value: &fastn_type::Value,
    doc: &ftd::interpreter::TDoc,
    save_types: &mut ftd::Map<TypeData>,
    line_number: usize,
) -> ftd::interpreter::Result<()> {
    if let fastn_type::Value::Record { fields, .. } = type_value {
        if type_value.is_record(ftd::interpreter::FTD_TYPE) {
            let size_field = fields.get("size").cloned();
            let letter_spacing_field = fields.get("letter-spacing").cloned();
            let font_family_field = fields.get("font-family").cloned();
            let weight_field = fields.get("weight").cloned();
            let line_height_field = fields.get("line-height").cloned();

            let size = extract_raw_data(size_field);
            let letter_spacing = extract_raw_data(letter_spacing_field);
            let font_family = extract_raw_data(font_family_field);
            let weight = extract_raw_data(weight_field);
            let line_height = extract_raw_data(line_height_field);

            save_types.insert(
                type_name,
                TypeData {
                    font_family,
                    size,
                    letter_spacing,
                    weight,
                    line_height,
                },
            );
        } else {
            return Err(ftd::interpreter::Error::InvalidKind {
                message: format!(
                    "Expected value of type record `ftd.type`: found {:?}",
                    type_value,
                ),
                doc_id: doc.name.to_string(),
                line_number,
            });
        }
    }

    Ok(())
}

fn extract_raw_data(property_value: Option<fastn_type::PropertyValue>) -> Option<ValueType> {
    return match property_value.as_ref() {
        Some(fastn_type::PropertyValue::Value { value, .. }) => match value {
            fastn_type::Value::String { text } => Some(ValueType {
                value: text.to_string(),
                type_: "string".to_string(),
            }),
            fastn_type::Value::Integer { value, .. } => Some(ValueType {
                value: value.to_string(),
                type_: "integer".to_string(),
            }),
            fastn_type::Value::Decimal { value, .. } => Some(ValueType {
                value: value.to_string(),
                type_: "decimal".to_string(),
            }),
            fastn_type::Value::Boolean { value, .. } => Some(ValueType {
                value: value.to_string(),
                type_: "boolean".to_string(),
            }),
            fastn_type::Value::OrType {
                value,
                full_variant,
                ..
            } => {
                let (_, variant) = full_variant
                    .rsplit_once('.')
                    .unwrap_or(("", full_variant.as_str()));
                let inner_value = extract_raw_data(Some(*value.clone()));
                if let Some(value) = inner_value {
                    return Some(ValueType {
                        value: value.value,
                        type_: variant.to_string(),
                    });
                }
                None
            }
            _ => None,
        },
        Some(fastn_type::PropertyValue::Reference { name, .. }) => Some(ValueType {
            value: name.to_string(),
            type_: "reference".to_string(),
        }),
        Some(fastn_type::PropertyValue::Clone { .. }) => None,
        Some(fastn_type::PropertyValue::FunctionCall { .. }) => None,
        None => None,
    };
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct TypeData {
    #[serde(rename = "font-family")]
    font_family: Option<ValueType>,
    size: Option<ValueType>,
    #[serde(rename = "letter-spacing")]
    letter_spacing: Option<ValueType>,
    weight: Option<ValueType>,
    #[serde(rename = "line-height")]
    line_height: Option<ValueType>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ValueType {
    value: String,
    #[serde(rename = "type")]
    type_: String,
}
