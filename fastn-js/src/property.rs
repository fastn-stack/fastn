pub struct SetProperty {
    pub kind: PropertyKind,
    pub value: SetPropertyValue,
    pub element_name: String,
}

pub enum SetPropertyValue {
    Reference(String),
    Value(Value),
}

impl SetPropertyValue {
    pub(crate) fn to_js(&self) -> String {
        match self {
            SetPropertyValue::Reference(name) => name.to_string(),
            SetPropertyValue::Value(v) => v.to_js(),
        }
    }
}

pub enum Value {
    String(String),
}

impl Value {
    pub(crate) fn to_js(&self) -> String {
        match self {
            Value::String(s) => format!("\"{s}\""),
        }
    }
}

pub enum PropertyKind {
    StringValue,
}

impl PropertyKind {
    pub(crate) fn to_js(&self) -> &'static str {
        match self {
            PropertyKind::StringValue => "fastn_dom.PropertyKind.StringValue",
        }
    }
}
