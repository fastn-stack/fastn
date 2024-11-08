extern crate self as fastn_type;

mod kind;
pub use kind::{Kind, KindData};

mod value;
pub use value::{PropertyValue, PropertyValueSource, Value};

mod function;
pub use function::FunctionCall;

mod component;
pub use component::{
    Argument, Component, ComponentSource, Event, EventName, Loop, Property, PropertySource,
};

mod expression;
pub use expression::Expression;

mod module;
pub use module::ModuleThing;

mod record;
pub use record::{AccessModifier, Field, Record};

pub type Map<T> = std::collections::BTreeMap<String, T>;
