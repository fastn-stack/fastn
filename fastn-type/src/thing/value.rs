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

impl PropertyValue {
    pub(crate) fn kind(&self) -> fastn_type::Kind {
        match self {
            PropertyValue::Value { value, .. } => value.kind(),
            PropertyValue::Reference { kind, .. } => kind.kind.to_owned(),
            PropertyValue::Clone { kind, .. } => kind.kind.to_owned(),
            PropertyValue::FunctionCall(fastn_type::FunctionCall { kind, .. }) => {
                kind.kind.to_owned()
            }
        }
    }

    pub fn line_number(&self) -> usize {
        match self {
            PropertyValue::Value { line_number, .. }
            | PropertyValue::Reference { line_number, .. }
            | PropertyValue::Clone { line_number, .. }
            | PropertyValue::FunctionCall(fastn_type::FunctionCall { line_number, .. }) => {
                *line_number
            }
        }
    }
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
        component: fastn_type::Component,
    },
    Module {
        name: String,
        things: fastn_type::Map<fastn_type::ModuleThing>,
    },
}

impl Value {
    pub(crate) fn kind(&self) -> fastn_type::Kind {
        match self {
            Value::String { .. } => fastn_type::Kind::string(),
            Value::Integer { .. } => fastn_type::Kind::integer(),
            Value::Decimal { .. } => fastn_type::Kind::decimal(),
            Value::Boolean { .. } => fastn_type::Kind::boolean(),
            Value::Object { .. } => fastn_type::Kind::object(),
            Value::Record { name, .. } => fastn_type::Kind::record(name),
            Value::KwArgs { .. } => fastn_type::Kind::kwargs(),
            Value::List { kind, .. } => kind.kind.clone().into_list(),
            Value::Optional { kind, .. } => fastn_type::Kind::Optional {
                kind: Box::new(kind.kind.clone()),
            },
            Value::UI { name, .. } => fastn_type::Kind::ui_with_name(name),
            Value::OrType {
                name,
                variant,
                full_variant,
                ..
            } => fastn_type::Kind::or_type_with_variant(name, variant, full_variant),
            Value::Module { .. } => fastn_type::Kind::module(),
        }
    }
}
