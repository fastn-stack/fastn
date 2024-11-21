#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WebComponentDefinition {
    pub name: String,
    pub arguments: Vec<fastn_resolved::Argument>,
    pub js: fastn_resolved::PropertyValue,
    pub line_number: usize,
}

impl WebComponentDefinition {
    pub fn new(
        name: &str,
        arguments: Vec<fastn_resolved::Argument>,
        js: fastn_resolved::PropertyValue,
        line_number: usize,
    ) -> fastn_resolved::WebComponentDefinition {
        fastn_resolved::WebComponentDefinition {
            name: name.to_string(),
            arguments,
            js,
            line_number,
        }
    }

    pub fn js(&self) -> Option<&str> {
        match self.js {
            fastn_resolved::PropertyValue::Value { ref value, .. } => match value {
                fastn_resolved::Value::String { text } => Some(text),
                _ => None,
            },
            _ => None,
        }
    }
}
