#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WebComponentDefinition {
    pub name: String,
    pub arguments: Vec<fastn_type::Argument>,
    pub js: fastn_type::PropertyValue,
    pub line_number: usize,
}
