#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Variable {
    pub name: String,
    pub kind: fastn_type::KindData,
    pub mutable: bool,
    pub value: fastn_type::PropertyValue,
    pub conditional_value: Vec<fastn_type::ConditionalValue>,
    pub line_number: usize,
    pub is_static: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ConditionalValue {
    pub condition: fastn_type::Expression,
    pub value: fastn_type::PropertyValue,
    pub line_number: usize,
}
