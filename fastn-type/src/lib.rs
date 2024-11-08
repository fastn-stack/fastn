extern crate self as fastn_type;

mod kind;
pub use kind::{Kind, KindData};

mod value;
pub use value::{PropertyValue, PropertyValueSource, Value};

mod function;
pub use function::FunctionCall;

pub type Map<T> = std::collections::BTreeMap<String, T>;
