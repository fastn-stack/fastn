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
    pub fn to_js(&self) -> String {
        match self {
            SetPropertyValue::Reference(name) => name.to_string(),
            SetPropertyValue::Value(v) => v.to_js(),
        }
    }
}

pub enum Value {
    String(String),
    Integer(i64),
    Decimal(f64),
    OrType {
        variant: String,
        value: Option<Box<SetPropertyValue>>,
    },
}

impl Value {
    pub(crate) fn to_js(&self) -> String {
        match self {
            Value::String(s) => format!("\"{s}\""),
            Value::Integer(i) => i.to_string(),
            Value::Decimal(f) => f.to_string(),
            Value::OrType { variant, value } => {
                if let Some(value) = value {
                    format!("{}({})", variant, value.to_js())
                } else {
                    variant.to_owned()
                }
            }
        }
    }
}

pub enum PropertyKind {
    StringValue,
    Width,
    Padding,
}

impl PropertyKind {
    pub(crate) fn to_js(&self) -> &'static str {
        match self {
            PropertyKind::StringValue => "fastn_dom.PropertyKind.StringValue",
            PropertyKind::Width => "fastn_dom.PropertyKind.Width",
            PropertyKind::Padding => "fastn_dom.PropertyKind.Padding",
        }
    }
}
