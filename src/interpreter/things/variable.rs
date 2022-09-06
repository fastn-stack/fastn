#[derive(Debug, PartialEq, Clone)]
pub struct Variable {
    pub name: String,
    pub value: ftd::interpreter::PropertyValue,
    pub conditions: Vec<ConditionalValue>,
    pub flags: VariableFlags,
    pub source: TextSource,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ConditionalValue {
    pub expression: ftd::interpreter::Boolean,
    pub value: ftd::interpreter::PropertyValue,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct VariableFlags {
    pub always_include: Option<bool>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TextSource {
    Header,
    Caption,
    Body,
    Default,
}
