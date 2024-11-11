#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PropertyValue {
    Value {
        value: fastn_type::Value,
        is_mutable: bool,
        line_number: usize,
    },
    Reference {
        name: String,
        kind: fastn_type::KindData,
        source: fastn_type::PropertyValueSource,
        is_mutable: bool,
        line_number: usize,
    },
    Clone {
        name: String,
        kind: fastn_type::KindData,
        source: fastn_type::PropertyValueSource,
        is_mutable: bool,
        line_number: usize,
    },
    FunctionCall(fastn_type::FunctionCall),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PropertyValueSource {
    Global,
    Local(String),
    Loop(String),
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
        values: fastn_type::Map<PropertyValue>,
    },
    Record {
        name: String,
        fields: fastn_type::Map<PropertyValue>,
    },
    KwArgs {
        arguments: fastn_type::Map<PropertyValue>,
    },
    OrType {
        name: String,
        variant: String,
        full_variant: String,
        value: Box<PropertyValue>, // Todo: Make it optional
    },
    List {
        data: Vec<PropertyValue>,
        kind: fastn_type::KindData,
    },
    Optional {
        data: Box<Option<Value>>,
        kind: fastn_type::KindData,
    },
    UI {
        name: String,
        kind: fastn_type::KindData,
        // component: fastn_type::Component,
    },
    Module {
        name: String,
        // things: fastn_type::Map<fastn_type::ModuleThing>,
    },
}

impl Value {
    pub fn new_none(kind: fastn_type::KindData) -> fastn_type::Value {
        fastn_type::Value::Optional {
            data: Box::new(None),
            kind,
        }
    }

    pub fn new_string(text: &str) -> fastn_type::Value {
        fastn_type::Value::String {
            text: text.to_string(),
        }
    }

    pub fn new_or_type(
        name: &str,
        variant: &str,
        full_variant: &str,
        value: fastn_type::PropertyValue,
    ) -> fastn_type::Value {
        fastn_type::Value::OrType {
            name: name.to_string(),
            variant: variant.to_string(),
            full_variant: full_variant.to_string(),
            value: Box::new(value),
        }
    }
}
