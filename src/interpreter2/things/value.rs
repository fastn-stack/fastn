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
        value: &ftd::ast::VariableValue,
        doc: &ftd::interpreter2::TDoc,
        kind: &ftd::interpreter2::KindData,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::PropertyValue> {
        let property_value = match kind.kind {
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
            _ => unimplemented!(),
        };

        Ok(property_value)
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
