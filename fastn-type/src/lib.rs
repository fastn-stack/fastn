extern crate self as fastn_type;

mod thing;
pub use thing::function::FunctionCall;
pub use thing::kind::{Kind, KindData};
pub use thing::value::{PropertyValue, PropertyValueSource, Value};

pub type Map<T> = std::collections::BTreeMap<String, T>;
