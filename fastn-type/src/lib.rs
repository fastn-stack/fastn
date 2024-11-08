extern crate self as fastn_type;

mod kind;
pub use kind::{Kind, KindData};

mod value;
pub use value::{PropertyValue, PropertyValueSource, Value};

mod function;
pub use function::{Function, FunctionCall};

mod component;
pub use component::{
    Argument, Component, ComponentDefinition, ComponentSource, Event, EventName, Loop, Property,
    PropertySource,
};

mod expression;
pub use expression::Expression;

mod module;
pub use module::ModuleThing;

mod record;
pub use record::{AccessModifier, Field, Record};

mod variable;
pub use variable::{ConditionalValue, Variable};

mod web_component;
pub use web_component::WebComponentDefinition;

mod or_type;
pub use or_type::{OrType, OrTypeVariant};

pub type Map<T> = std::collections::BTreeMap<String, T>;

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
