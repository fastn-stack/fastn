#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Function {
    pub name: String,
    pub return_kind: fastn_type::KindData,
    pub arguments: Vec<fastn_type::Argument>,
    pub expression: Vec<Expression>,
    pub js: Option<fastn_type::PropertyValue>,
    pub line_number: usize,
    pub external_implementation: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Expression {
    pub expression: String,
    pub line_number: usize,
}

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
