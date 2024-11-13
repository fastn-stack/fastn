#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Function {
    pub name: String,
    pub return_kind: fastn_type::KindData,
    pub arguments: Vec<fastn_type::Argument>,
    pub expression: Vec<fastn_type::FunctionExpression>,
    pub js: Option<fastn_type::PropertyValue>,
    pub line_number: usize,
    pub external_implementation: bool,
}

impl Function {
    pub fn new(
        name: &str,
        return_kind: fastn_type::KindData,
        arguments: Vec<fastn_type::Argument>,
        expression: Vec<fastn_type::FunctionExpression>,
        js: Option<fastn_type::PropertyValue>,
        line_number: usize,
    ) -> Function {
        Function {
            name: name.to_string(),
            return_kind,
            arguments,
            expression,
            js,
            line_number,
            external_implementation: false,
        }
    }
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

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct FunctionExpression {
    pub expression: String,
    pub line_number: usize,
}
