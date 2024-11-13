#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Variable {
    pub name: String,
    pub kind: fastn_type::KindData,
    pub mutable: bool,
    pub value: fastn_type::PropertyValue,
    pub conditional_value: Vec<ConditionalValue>,
    pub line_number: usize,
    pub is_static: bool,
}

impl Variable {
    pub fn is_static(&self) -> bool {
        !self.mutable && self.is_static
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ConditionalValue {
    pub condition: fastn_type::Expression,
    pub value: fastn_type::PropertyValue,
    pub line_number: usize,
}

impl ConditionalValue {
    pub fn new(
        condition: fastn_type::Expression,
        value: fastn_type::PropertyValue,
        line_number: usize,
    ) -> ConditionalValue {
        ConditionalValue {
            condition,
            value,
            line_number,
        }
    }
}
