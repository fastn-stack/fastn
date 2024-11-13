#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WebComponentDefinition {
    pub name: String,
    pub arguments: Vec<fastn_type::Argument>,
    pub js: fastn_type::PropertyValue,
    pub line_number: usize,
}

impl WebComponentDefinition {
    pub fn new(
        name: &str,
        arguments: Vec<fastn_type::Argument>,
        js: fastn_type::PropertyValue,
        line_number: usize,
    ) -> fastn_type::WebComponentDefinition {
        fastn_type::WebComponentDefinition {
            name: name.to_string(),
            arguments,
            js,
            line_number,
        }
    }
}
