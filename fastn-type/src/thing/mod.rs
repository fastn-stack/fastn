pub mod component;
pub mod default;
pub mod expression;
pub mod function;
pub mod kind;
pub mod module;
pub mod or_type;
pub mod record;
pub mod value;
pub mod variable;
pub mod web_component;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Thing {
    Record(fastn_type::Record),
    OrType(fastn_type::OrType),
    OrTypeWithVariant {
        or_type: String,
        variant: fastn_type::OrTypeVariant,
    },
    Variable(fastn_type::Variable),
    Component(fastn_type::ComponentDefinition),
    WebComponent(fastn_type::WebComponentDefinition),
    Function(fastn_type::Function),
    Export {
        from: String,
        to: String,
        line_number: usize,
    },
}
