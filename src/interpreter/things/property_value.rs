#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Value {
        value: ftd::interpreter::Value,
    },
    Reference {
        name: String,
        kind: ftd::interpreter::KindData,
    },
}

impl PropertyValue {
    pub(crate) fn to_property_value(
        s: &ftd::p11::Section,
        doc_id: &str,
        kind_data: &ftd::interpreter::KindData,
    ) -> ftd::interpreter::Result<PropertyValue> {
        Ok(match &kind_data.kind {
            ftd::interpreter::Kind::String
            | ftd::interpreter::Kind::Integer
            | ftd::interpreter::Kind::Decimal
            | ftd::interpreter::Kind::Boolean
            | ftd::interpreter::Kind::List { .. } => {
                let value = section_value_from_caption_or_body(s, doc_id)?;
                if let Some(reference) = get_reference(value.as_str()) {
                    PropertyValue::reference(reference.to_string(), kind_data.to_owned())
                } else {
                    let value = Value::to_value(s, doc_id, &kind_data)?;
                    PropertyValue::value(value)
                }
            }
            _ => unimplemented!(),
        })
    }

    pub(crate) fn reference(name: String, kind: ftd::interpreter::KindData) -> PropertyValue {
        PropertyValue::Reference { name, kind }
    }

    pub(crate) fn value(value: ftd::interpreter::Value) -> PropertyValue {
        PropertyValue::Value { value }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    None {
        kind: ftd::interpreter::KindData,
    },
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
    OrType {
        name: String,
        variant: String,
        fields: ftd::Map<PropertyValue>,
    },
    List {
        data: Vec<PropertyValue>,
        kind: ftd::interpreter::KindData,
    },
    Optional {
        data: Box<Option<Value>>,
        kind: ftd::interpreter::KindData,
    },
    Map {
        data: ftd::Map<Value>,
        kind: ftd::p2::Kind,
    },
    // TODO: UI
    // UI {
    //     name: String,
    //     component: ftd::interpreter::Component,
    // },
}

impl Value {
    pub(crate) fn to_value(
        s: &ftd::p11::Section,
        doc_id: &str,
        kind_data: &ftd::interpreter::KindData,
    ) -> ftd::interpreter::Result<Value> {
        match &kind_data.kind {
            ftd::interpreter::Kind::String
            | ftd::interpreter::Kind::Integer
            | ftd::interpreter::Kind::Decimal
            | ftd::interpreter::Kind::Boolean => {
                let value = section_value_from_caption_or_body(s, doc_id)?;
                Value::to_value_for_basic_kind(value.as_str(), &kind_data.kind)
            }
            ftd::interpreter::Kind::List { kind } => {
                let mut data = vec![];
                for subsection in s.sub_sections.iter() {
                    data.push(PropertyValue::to_property_value(
                        subsection,
                        doc_id,
                        &kind
                            .to_owned()
                            .into_kind_data(kind_data.caption, kind_data.body),
                    )?);
                }
                Ok(Value::List {
                    data,
                    kind: kind_data.to_owned(),
                })
            }
            _ => unimplemented!(),
        }
    }

    pub(crate) fn to_value_for_basic_kind(
        s: &str,
        kind: &ftd::interpreter::Kind,
    ) -> ftd::interpreter::Result<Value> {
        Ok(match kind {
            ftd::interpreter::Kind::String => Value::String {
                text: s.to_string(),
            },
            ftd::interpreter::Kind::Integer => Value::Integer {
                value: s.parse::<i64>()?,
            },
            ftd::interpreter::Kind::Decimal => Value::Decimal {
                value: s.parse::<f64>()?,
            },
            ftd::interpreter::Kind::Boolean => Value::Boolean {
                value: s.parse::<bool>()?,
            },
            _ => unreachable!(),
        })
    }
}

fn section_value_from_caption_or_body(
    section: &ftd::p11::Section,
    doc_id: &str,
) -> ftd::interpreter::Result<String> {
    if let Some(ref header) = section.caption {
        if let Some(value) = header.get_value(doc_id, section.line_number)? {
            return Ok(value);
        }
    }

    if let Some(ref body) = section.body {
        return Ok(body.value.to_string());
    }

    Err(ftd::interpreter::Error::ValueNotFound {
        doc_id: doc_id.to_string(),
        line_number: section.line_number,
        message: format!("Caption and body not found {}", section.name),
    })
}

pub(crate) fn get_reference(s: &str) -> Option<&str> {
    s.trim().strip_prefix('$')
}

// #[cfg(test)]
// mod test {
//     fn
// }
