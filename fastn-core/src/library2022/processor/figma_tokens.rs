use serde::{Deserialize, Serialize};

pub fn process_figma_tokens(
    value: ftd_ast::VariableValue,
    kind: fastn_type::Kind,
    doc: &mut ftd::interpreter::TDoc,
) -> ftd::interpreter::Result<fastn_type::Value> {
    let line_number = value.line_number();
    let mut variable_name: Option<String> = None;

    let mut light_colors: ftd::Map<ftd::Map<VT>> = ftd::Map::new();
    let mut dark_colors: ftd::Map<ftd::Map<VT>> = ftd::Map::new();

    extract_light_dark_colors(
        &value,
        doc,
        &mut variable_name,
        &mut light_colors,
        &mut dark_colors,
        line_number,
    )?;

    let json_formatted_light =
        serde_json::to_string_pretty(&light_colors).expect("Not a serializable type");
    let json_formatted_dark =
        serde_json::to_string_pretty(&dark_colors).expect("Not a serializable type");

    let full_cs = format!(
        "{{\n\"{}-light\": {},\n\"{}-dark\": {}\n}}",
        variable_name
            .clone()
            .unwrap_or_else(|| "Unnamed-cs".to_string())
            .as_str(),
        json_formatted_light,
        variable_name
            .unwrap_or_else(|| "Unnamed-cs".to_string())
            .as_str(),
        json_formatted_dark
    );

    let response_json: serde_json::Value = serde_json::Value::String(full_cs);
    doc.from_json(&response_json, &kind, &value)
}

pub fn process_figma_tokens_old(
    value: ftd_ast::VariableValue,
    kind: fastn_type::Kind,
    doc: &mut ftd::interpreter::TDoc,
) -> ftd::interpreter::Result<fastn_type::Value> {
    let line_number = value.line_number();
    let mut variable_name: Option<String> = None;

    let mut light_colors: ftd::Map<ftd::Map<VT>> = ftd::Map::new();
    let mut dark_colors: ftd::Map<ftd::Map<VT>> = ftd::Map::new();

    extract_light_dark_colors(
        &value,
        doc,
        &mut variable_name,
        &mut light_colors,
        &mut dark_colors,
        line_number,
    )?;

    let mut final_light: String = String::new();
    let mut final_dark: String = String::new();

    for (color_title, values) in light_colors.iter() {
        let color_key = color_title
            .trim_end_matches(" Colors")
            .to_lowercase()
            .replace(' ', "-");
        let json_string_light_values =
            serde_json::to_string_pretty(&values).expect("Not a serializable type");

        match color_key.as_str() {
            "accent" | "cta-primary" => {
                final_light = format!(
                    indoc::indoc! {
                        "{previous}\"{color_title}\": {{
                            \"$fpm\": {{
                                \"color\": {{
                                    \"{color_key}\": {color_list}
                                }}
                            }}
                        }},\n"
                    },
                    previous = final_light,
                    color_key = color_key,
                    color_title = color_title,
                    color_list = json_string_light_values,
                );
            }
            "cta-secondary" => {
                final_light = format!(
                    indoc::indoc! {
                        "{previous}\"{color_title}\": {{
                            \"$fpm\": {{
                                \"color\": {{
                                    \"{color_key}\": {color_list}
                                }}
                            }}
                        }},\n"
                    },
                    previous = final_light,
                    color_key = color_key,
                    color_title = color_title.trim_end_matches('s'),
                    color_list = json_string_light_values,
                );
            }
            "standalone" => {
                final_light = format!(
                    indoc::indoc! {
                        "{previous}\"{color_title}\": {{
                            \"$fpm\": {{
                                \"color\": {{
                                    \"main\": {color_list}
                                }}
                            }}
                        }},\n"
                    },
                    previous = final_light,
                    color_title = color_title,
                    color_list = json_string_light_values,
                );
            }
            _ => {
                final_light = format!(
                    indoc::indoc! {
                        "{previous}\"{color_title}\": {{
                            \"$fpm\": {{
                                \"color\": {{
                                    \"main\": {{
                                        \"{color_key}\": {color_list}
                                    }}
                                }}
                            }}
                        }},\n"
                    },
                    previous = final_light,
                    color_key = color_key,
                    color_title = color_title,
                    color_list = json_string_light_values,
                );
            }
        }
    }

    for (color_title, values) in dark_colors.iter() {
        let color_key = color_title
            .trim_end_matches(" Colors")
            .to_lowercase()
            .replace(' ', "-");
        let json_string_dark_values =
            serde_json::to_string_pretty(&values).expect("Not a serializable type");

        match color_key.as_str() {
            "accent" | "cta-primary" => {
                final_dark = format!(
                    indoc::indoc! {
                        "{previous}\"{color_title}\": {{
                            \"$fpm\": {{
                                \"color\": {{
                                    \"{color_key}\": {color_list}
                                }}
                            }}
                        }},\n"
                    },
                    previous = final_dark,
                    color_key = color_key,
                    color_title = color_title,
                    color_list = json_string_dark_values,
                );
            }
            "cta-secondary" => {
                final_dark = format!(
                    indoc::indoc! {
                        "{previous}\"{color_title}\": {{
                            \"$fpm\": {{
                                \"color\": {{
                                    \"{color_key}\": {color_list}
                                }}
                            }}
                        }},\n"
                    },
                    previous = final_dark,
                    color_key = color_key,
                    color_title = color_title.trim_end_matches('s'),
                    color_list = json_string_dark_values,
                );
            }
            "standalone" => {
                final_dark = format!(
                    indoc::indoc! {
                        "{previous}\"{color_title}\": {{
                            \"$fpm\": {{
                                \"color\": {{
                                    \"main\": {color_list}
                                }}
                            }}
                        }},\n"
                    },
                    previous = final_dark,
                    color_title = color_title,
                    color_list = json_string_dark_values,
                );
            }
            _ => {
                final_dark = format!(
                    indoc::indoc! {
                        "{previous}\"{color_title}\": {{
                            \"$fpm\": {{
                                \"color\": {{
                                    \"main\": {{
                                        \"{color_key}\": {color_list}
                                    }}
                                }}
                            }}
                        }},\n"
                    },
                    previous = final_dark,
                    color_key = color_key,
                    color_title = color_title,
                    color_list = json_string_dark_values,
                );
            }
        }
    }

    let json_formatted_light = final_light.trim_end_matches(",\n").to_string();
    let json_formatted_dark = final_dark.trim_end_matches(",\n").to_string();

    let full_cs = format!(
        "{{\n\"{} light\": {{\n{}\n}},\n\"{} dark\": {{\n{}\n}}\n}}",
        variable_name
            .clone()
            .unwrap_or_else(|| "Unnamed-cs".to_string())
            .as_str(),
        json_formatted_light,
        variable_name
            .unwrap_or_else(|| "Unnamed-cs".to_string())
            .as_str(),
        json_formatted_dark
    );

    let response_json: serde_json::Value = serde_json::Value::String(full_cs);
    doc.from_json(&response_json, &kind, &value)
}

pub fn capitalize_word(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn extract_light_dark_colors(
    value: &ftd_ast::VariableValue,
    doc: &mut ftd::interpreter::TDoc,
    variable_name: &mut Option<String>,
    light_colors: &mut ftd::Map<ftd::Map<VT>>,
    dark_colors: &mut ftd::Map<ftd::Map<VT>>,
    line_number: usize,
) -> ftd::interpreter::Result<()> {
    let headers = match &value {
        ftd_ast::VariableValue::Record { headers, .. } => headers,
        _ => {
            return Err(ftd::interpreter::Error::InvalidKind {
                message: format!("Expected record of color-scheme found: {:?}", value),
                doc_id: doc.name.to_string(),
                line_number,
            })
        }
    };

    let header = headers.get_by_key_optional("variable", doc.name, line_number)?;
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

    let variable = if let Some(variable) = header {
        variable
    } else {
        return Err(ftd::interpreter::Error::InvalidKind {
            message: format!("`variable` named header not found: {:?}", value),
            doc_id: doc.name.to_string(),
            line_number,
        });
    };

    let hval = match &variable.value {
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
        *variable_name = Some(hval.trim_start_matches('$').to_string());
    }
    let bag_entry = doc.resolve_name(hval);
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
        ftd::interpreter::PropertyValue::Value {
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

    let mut standalone_light_colors: ftd::Map<VT> = ftd::Map::new();
    let mut standalone_dark_colors: ftd::Map<VT> = ftd::Map::new();

    for (k, v) in fields.iter() {
        let field_value = v.clone().resolve(doc, v.line_number())?;
        let color_title = format_color_title(k.as_str());
        match k.as_str() {
            "accent" | "background" | "custom" | "cta-danger" | "cta-primary" | "cta-tertiary"
            | "cta-secondary" | "error" | "info" | "success" | "warning" => {
                let mut extracted_light_colors: ftd::Map<VT> = ftd::Map::new();
                let mut extracted_dark_colors: ftd::Map<VT> = ftd::Map::new();
                extract_colors(
                    k.to_string(),
                    &field_value,
                    doc,
                    &mut extracted_light_colors,
                    &mut extracted_dark_colors,
                )?;
                light_colors.insert(color_title.clone(), extracted_light_colors);
                dark_colors.insert(color_title, extracted_dark_colors);
            }
            _ => {
                // Standalone colors
                extract_colors(
                    k.to_string(),
                    &field_value,
                    doc,
                    &mut standalone_light_colors,
                    &mut standalone_dark_colors,
                )?;
            }
        }
    }
    light_colors.insert("Standalone Colors".to_string(), standalone_light_colors);
    dark_colors.insert("Standalone Colors".to_string(), standalone_dark_colors);

    Ok(())
}

fn format_color_title(title: &str) -> String {
    let words = title.split('-');
    let mut res = String::new();
    for word in words {
        let mut capitalized_word = capitalize_word(word);
        if capitalized_word.eq("Cta") {
            capitalized_word = capitalized_word.to_uppercase();
        }
        res.push_str(capitalized_word.as_str());
        res.push(' ')
    }
    res.push_str("Colors");
    res.trim().to_string()
}

fn extract_colors(
    color_name: String,
    color_value: &fastn_type::Value,
    doc: &ftd::interpreter::TDoc,
    extracted_light_colors: &mut ftd::Map<VT>,
    extracted_dark_colors: &mut ftd::Map<VT>,
) -> ftd::interpreter::Result<()> {
    if let fastn_type::Value::Record { fields, .. } = color_value {
        if color_value.is_record("ftd#color") {
            if let Some(ftd::interpreter::PropertyValue::Value {
                value: fastn_type::Value::String { text: light_value },
                ..
            }) = fields.get("light")
            {
                extracted_light_colors.insert(
                    color_name.to_string(),
                    VT {
                        value: light_value.to_lowercase(),
                        type_: "color".to_string(),
                    },
                );
            }
            if let Some(ftd::interpreter::PropertyValue::Value {
                value: fastn_type::Value::String { text: dark_value },
                ..
            }) = fields.get("dark")
            {
                extracted_dark_colors.insert(
                    color_name,
                    VT {
                        value: dark_value.to_lowercase(),
                        type_: "color".to_string(),
                    },
                );
            }
        } else {
            for (k, v) in fields.iter() {
                let inner_field_value = v.clone().resolve(doc, v.line_number())?;
                extract_colors(
                    k.to_string(),
                    &inner_field_value,
                    doc,
                    extracted_light_colors,
                    extracted_dark_colors,
                )?;
            }
        }
    }
    Ok(())
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct VT {
    value: String,
    #[serde(rename = "type")]
    type_: String,
}
