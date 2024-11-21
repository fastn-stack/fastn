#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Variable {
    pub name: String,
    pub kind: fastn_resolved::KindData,
    pub mutable: bool,
    pub value: fastn_resolved::PropertyValue,
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
    pub condition: fastn_resolved::Expression,
    pub value: fastn_resolved::PropertyValue,
    pub line_number: usize,
}

impl ConditionalValue {
    pub fn new(
        condition: fastn_resolved::Expression,
        value: fastn_resolved::PropertyValue,
        line_number: usize,
    ) -> ConditionalValue {
        ConditionalValue {
            condition,
            value,
            line_number,
        }
    }
}
