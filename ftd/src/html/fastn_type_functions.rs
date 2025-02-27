pub(crate) trait KindExt {
    fn is_ftd_responsive_type(&self) -> bool;
    fn is_ftd_type(&self) -> bool;
    fn is_ftd_font_size(&self) -> bool;
    fn is_ftd_background_color(&self) -> bool;
    fn is_ftd_length(&self) -> bool;
    fn is_ftd_image_src(&self) -> bool;
    fn is_ftd_color(&self) -> bool;
    fn is_ftd_resizing(&self) -> bool;
    fn is_ftd_resizing_fixed(&self) -> bool;
}

impl KindExt for fastn_resolved::Kind {
    fn is_ftd_responsive_type(&self) -> bool {
        matches!(self, fastn_resolved::Kind::Record { name, .. } if name.eq
            (ftd::interpreter::FTD_RESPONSIVE_TYPE))
    }

    fn is_ftd_type(&self) -> bool {
        matches!(self, fastn_resolved::Kind::Record { name, .. } if name.eq(ftd::interpreter::FTD_TYPE))
    }

    fn is_ftd_font_size(&self) -> bool {
        matches!(self, fastn_resolved::Kind::Record { name, .. } if name.eq
            (ftd::interpreter::FTD_FONT_SIZE))
    }

    fn is_ftd_background_color(&self) -> bool {
        matches!(self, fastn_resolved::Kind::OrType { name, variant, .. } if name.eq
            (ftd::interpreter::FTD_BACKGROUND) &&
            variant.is_some() && variant.as_ref().unwrap().starts_with(ftd::interpreter::FTD_BACKGROUND_SOLID))
    }

    fn is_ftd_length(&self) -> bool {
        matches!(self, fastn_resolved::Kind::OrType { name, .. } if name.eq
            (ftd::interpreter::FTD_LENGTH))
    }

    fn is_ftd_image_src(&self) -> bool {
        matches!(self, fastn_resolved::Kind::Record { name, .. } if name.eq
            (ftd::interpreter::FTD_IMAGE_SRC))
    }

    fn is_ftd_color(&self) -> bool {
        matches!(self, fastn_resolved::Kind::Record { name, .. } if name.eq
            (ftd::interpreter::FTD_COLOR))
    }

    fn is_ftd_resizing(&self) -> bool {
        matches!(self, fastn_resolved::Kind::OrType { name, .. } if name.eq
            (ftd::interpreter::FTD_RESIZING))
    }

    fn is_ftd_resizing_fixed(&self) -> bool {
        matches!(self, fastn_resolved::Kind::OrType { name, variant, .. } if name.eq
            (ftd::interpreter::FTD_RESIZING) && variant.is_some() && variant.as_ref().unwrap().starts_with(ftd::interpreter::FTD_RESIZING_FIXED))
    }
}

pub(crate) trait PropertyValueExt {
    fn to_html_string(
        &self,
        doc: &ftd::interpreter::TDoc,
        field: Option<String>,
        id: &str,
        string_needs_no_quotes: bool,
    ) -> ftd::html::Result<Option<String>>;
}

impl PropertyValueExt for fastn_resolved::PropertyValue {
    fn to_html_string(
        &self,
        doc: &ftd::interpreter::TDoc,
        field: Option<String>,
        id: &str,
        string_needs_no_quotes: bool,
    ) -> ftd::html::Result<Option<String>> {
        Ok(match self {
            fastn_resolved::PropertyValue::Reference { name, .. } => Some(format!(
                "resolve_reference(\"{}\", data){}",
                ftd::html::utils::js_reference_name(name),
                field.map(|v| format!(".{}", v)).unwrap_or_default()
            )),
            fastn_resolved::PropertyValue::FunctionCall(function_call) => {
                let action = serde_json::to_string(&ftd::html::Action::from_function_call(
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
            fastn_resolved::PropertyValue::Value {
                value, line_number, ..
            } => value.to_html_string(doc, *line_number, field, id, string_needs_no_quotes)?,
            _ => None,
        })
    }
}

pub(crate) trait ValueExt {
    fn to_html_string(
        &self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
        field: Option<String>,
        id: &str,
        string_needs_no_quotes: bool,
    ) -> ftd::html::Result<Option<String>>;
}

impl ValueExt for fastn_resolved::Value {
    // string_needs_no_quotes: for class attribute the value should be red-block not "red-block"
    fn to_html_string(
        &self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
        field: Option<String>,
        id: &str,
        string_needs_no_quotes: bool,
    ) -> ftd::html::Result<Option<String>> {
        use ftd::html::fastn_type_functions::PropertyValueExt as _;
        use ftd::interpreter::PropertyValueExt;

        Ok(match self {
            fastn_resolved::Value::String { text } if !string_needs_no_quotes => {
                Some(format!("\"{}\"", text))
            }
            fastn_resolved::Value::String { text } if string_needs_no_quotes => {
                Some(text.to_string())
            }
            fastn_resolved::Value::Integer { value } => Some(value.to_string()),
            fastn_resolved::Value::Decimal { value } => Some(value.to_string()),
            fastn_resolved::Value::Boolean { value } => Some(value.to_string()),
            fastn_resolved::Value::List { data, .. } => {
                let mut values = vec![];
                for value in data {
                    let v = if let Some(v) = value
                        .clone()
                        .resolve(doc, line_number)?
                        .to_html_string(doc, value.line_number(), None, id, true)?
                    {
                        v
                    } else {
                        continue;
                    };
                    values.push(v);
                }
                Some(format!("{:?}", values.join(" ")))
            }
            fastn_resolved::Value::Record { fields, .. }
                if field
                    .as_ref()
                    .map(|v| fields.contains_key(v))
                    .unwrap_or(false) =>
            {
                fields.get(&field.unwrap()).unwrap().to_html_string(
                    doc,
                    None,
                    id,
                    string_needs_no_quotes,
                )?
            }
            fastn_resolved::Value::OrType {
                value,
                variant,
                full_variant,
                name,
                ..
            } => {
                let value = value.to_html_string(doc, field, id, string_needs_no_quotes)?;
                match value {
                    Some(value) if name.eq(ftd::interpreter::FTD_LENGTH) => {
                        if let Ok(pattern) = ftd::executor::Length::set_pattern_from_variant_str(
                            variant,
                            doc.name,
                            line_number,
                        ) {
                            Some(format!("`{}`.format(JSON.stringify({}))", pattern, value))
                        } else {
                            Some(value)
                        }
                    }
                    Some(value)
                        if name.eq(ftd::interpreter::FTD_RESIZING)
                            && variant.ne(ftd::interpreter::FTD_RESIZING_FIXED) =>
                    {
                        if let Ok(pattern) = ftd::executor::Resizing::set_pattern_from_variant_str(
                            variant,
                            full_variant,
                            doc.name,
                            line_number,
                        ) {
                            Some(format!("`{}`.format(JSON.stringify({}))", pattern, value))
                        } else {
                            Some(value)
                        }
                    }
                    Some(value) => Some(value),
                    None => None,
                }
            }
            fastn_resolved::Value::Record { fields, .. } => {
                let mut values = vec![];
                for (k, v) in fields {
                    let value = if let Some(v) =
                        v.to_html_string(doc, field.clone(), id, string_needs_no_quotes)?
                    {
                        v
                    } else {
                        "null".to_string()
                    };
                    values.push(format!("\"{}\": {}", k, value));
                }

                Some(format!("{{{}}}", values.join(", ")))
            }
            fastn_resolved::Value::Optional { data, .. } if data.is_none() => None,
            t => unimplemented!("{:?}", t),
        })
    }
}
