use crate::interpreter2::Kind;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PropertyValue {
    Value {
        value: ftd::interpreter2::Value,
        line_number: usize,
    },
    Reference {
        name: String,
        kind: ftd::interpreter2::KindData,
        line_number: usize,
    },
}

impl PropertyValue {
    pub(crate) fn from_ast_value_with_kind(
        value: ftd::ast::VariableValue,
        doc: &ftd::interpreter2::TDoc,
        kind_data: &ftd::interpreter2::KindData,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::PropertyValue> {
        if let Ok(reference) = PropertyValue::reference_from_ast_value(&value, doc, kind_data) {
            return Ok(reference);
        }
        Ok(match &kind_data.kind {
            Kind::String => PropertyValue::Value {
                value: Value::String {
                    text: value.string(doc.name)?,
                },
                line_number: value.line_number(),
            },
            Kind::Integer => PropertyValue::Value {
                value: Value::Integer {
                    value: value.string(doc.name)?.parse()?,
                },
                line_number: value.line_number(),
            },
            Kind::Decimal => PropertyValue::Value {
                value: Value::Decimal {
                    value: value.string(doc.name)?.parse()?,
                },
                line_number: value.line_number(),
            },
            Kind::Boolean => PropertyValue::Value {
                value: Value::Boolean {
                    value: value.string(doc.name)?.parse()?,
                },
                line_number: value.line_number(),
            },
            Kind::List { kind } if value.is_list() => {
                let line_number = value.line_number();
                let value_list = value.into_list(doc.name)?;
                let mut values = vec![];
                for value in value_list {
                    values.push(PropertyValue::from_ast_value_with_kind(
                        value,
                        doc,
                        &ftd::interpreter2::KindData {
                            kind: kind.as_ref().clone(),
                            caption: kind_data.caption,
                            body: kind_data.body,
                        },
                    )?);
                }
                PropertyValue::Value {
                    value: ftd::interpreter2::Value::List {
                        data: values,
                        kind: kind_data.clone(),
                    },
                    line_number,
                }
            }
            Kind::Record { name } if value.is_record(name) => {
                let record = doc.get_record(value.line_number(), name)?;
                let (_, caption, headers, body, line_number) = value.get_record(name, doc.name)?;
                let mut result_field: ftd::Map<PropertyValue> = Default::default();
                for field in record.fields {
                    if field.is_caption() && caption.is_some() {
                        let caption = caption.as_ref().as_ref().unwrap().clone();
                        result_field.insert(
                            field.name.to_string(),
                            PropertyValue::from_ast_value_with_kind(caption, doc, &field.kind)?,
                        );
                        continue;
                    }
                    if field.is_body() && body.is_some() {
                        let body = body.as_ref().unwrap();
                        result_field.insert(
                            field.name.to_string(),
                            PropertyValue::from_ast_value_with_kind(
                                ftd::ast::VariableValue::String {
                                    value: body.value.to_string(),
                                    line_number: body.line_number,
                                },
                                doc,
                                &field.kind,
                            )?,
                        );
                        continue;
                    }
                    let headers = headers.get_by_key(field.name.as_str());
                    if headers.is_empty() && field.kind.is_optional() {
                        result_field.insert(
                            field.name.to_string(),
                            PropertyValue::Value {
                                value: ftd::interpreter2::Value::Optional {
                                    data: Box::new(None),
                                    kind: kind_data.to_owned(),
                                },
                                line_number,
                            },
                        );
                        continue;
                    }
                    if field.kind.is_list() {
                        let mut header_list = vec![];
                        for header in headers {
                            header_list.extend(match &header.value {
                                ftd::ast::VariableValue::List { value, .. } => value.to_owned(),
                                t => vec![t.to_owned()],
                            });
                        }
                        result_field.insert(
                            field.name.to_string(),
                            PropertyValue::from_ast_value_with_kind(
                                ftd::ast::VariableValue::List {
                                    value: header_list,
                                    line_number: value.line_number(),
                                },
                                doc,
                                &field.kind,
                            )?,
                        );
                        continue;
                    }
                    if headers.len() != 1 {
                        return ftd::interpreter2::utils::e2(
                            format!(
                                "Expected `{}` of type `{:?}`, found: `{:?}`",
                                field.name, field.kind, headers
                            ),
                            doc.name,
                            value.line_number(),
                        );
                    }
                    let first_header = headers.first().unwrap();
                    result_field.insert(
                        field.name.to_string(),
                        PropertyValue::from_ast_value_with_kind(
                            first_header.value.clone(),
                            doc,
                            &field.kind,
                        )?,
                    );
                }
                PropertyValue::Value {
                    value: ftd::interpreter2::Value::Record {
                        name: name.to_string(),
                        fields: result_field,
                    },
                    line_number,
                }
            }
            _ => unimplemented!(),
        })
    }

    fn reference_from_ast_value(
        value: &ftd::ast::VariableValue,
        doc: &ftd::interpreter2::TDoc,
        kind: &ftd::interpreter2::KindData,
    ) -> ftd::ast::Result<ftd::interpreter2::PropertyValue> {
        match value.string(doc.name) {
            Ok(name) if name.starts_with(ftd::interpreter2::utils::REFERENCE) => {
                let reference = name
                    .trim_start_matches(ftd::interpreter2::utils::REFERENCE)
                    .to_string();
                Ok(PropertyValue::Reference {
                    name: reference,
                    kind: kind.clone(),
                    line_number: value.line_number(),
                })
            }
            _ => ftd::ast::parse_error(
                format!("Expected reference, found: `{:?}`", value),
                doc.name,
                value.line_number(),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Value {
    String {
        text: String,
    },
    Integer {
        value: i64,
    },
    Decimal {
        value: f64,
    },
    Boolean {
        value: bool,
    },
    Object {
        values: ftd::Map<PropertyValue>,
    },
    Record {
        name: String,
        fields: ftd::Map<PropertyValue>,
    },
    List {
        data: Vec<PropertyValue>,
        kind: ftd::interpreter2::KindData,
    },
    Optional {
        data: Box<Option<Value>>,
        kind: ftd::interpreter2::KindData,
    },
    // TODO: UI
    // UI {
    //     name: String,
    //     component: ftd::interpreter::Component,
    // },
}
