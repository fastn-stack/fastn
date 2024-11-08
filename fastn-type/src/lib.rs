extern crate self as fastn_type;

mod kind;
pub use kind::{Kind, KindData};

mod value;
pub use value::{PropertyValue, PropertyValueSource, Value};

mod function;
pub use function::FunctionCall;

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

pub type Map<T> = std::collections::BTreeMap<String, T>;
