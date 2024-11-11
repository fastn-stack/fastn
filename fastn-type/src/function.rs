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

impl FunctionCall {
    pub fn new(
        name: &str,
        kind: fastn_type::KindData,
        is_mutable: bool,
        line_number: usize,
        values: fastn_type::Map<fastn_type::PropertyValue>,
        order: Vec<String>,
        module_name: Option<(String, String)>,
    ) -> FunctionCall {
        FunctionCall {
            name: name.to_string(),
            kind,
            is_mutable,
            line_number,
            values,
            order,
            module_name,
        }
    }
}
