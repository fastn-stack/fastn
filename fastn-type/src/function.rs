#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct FunctionCall {
    pub name: String,
    pub kind: fastn_type::KindData,
    pub is_mutable: bool,
    pub line_number: usize,
    pub values: fastn_type::Map<fastn_type::PropertyValue>,
    pub order: Vec<String>,
    // (Default module, Argument name of module kind)
    pub module_name: Option<(String, String)>,
}
