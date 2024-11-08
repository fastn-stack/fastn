extern crate self as fastn_type;

mod thing;
pub use thing::component::{
    Argument, Component, ComponentDefinition, ComponentSource, Event, EventName, Loop, Property,
    PropertySource,
};
pub use thing::default;
pub use thing::expression::Expression;
pub use thing::function::{Function, FunctionCall};
pub use thing::kind::{Kind, KindData};
pub use thing::module::ModuleThing;
pub use thing::or_type::{OrType, OrTypeVariant};
pub use thing::record::{AccessModifier, Field, Record};
pub use thing::value::{PropertyValue, PropertyValueSource, Value};
pub use thing::variable::{ConditionalValue, Variable};
pub use thing::web_component::WebComponentDefinition;
pub use thing::Thing;

mod tdoc;
pub use tdoc::TDoc;

mod data;
pub use data::Data;
mod js_ast;
mod resolver;
pub use resolver::ResolverData;
mod ftd_to_js_variant;
pub(crate) use ftd_to_js_variant::ftd_to_js_variant;

mod utils;

pub type Map<T> = std::collections::BTreeMap<String, T>;

pub const FTD_SPECIAL_VALUE: &str = "$VALUE";
pub const FTD_SPECIAL_CHECKED: &str = "$CHECKED";
pub const FTD_INHERITED: &str = "inherited";
pub const FTD_LOOP_COUNTER: &str = "LOOP.COUNTER";
pub const CLONE: &str = "*$";
pub const REFERENCE: &str = "$";
