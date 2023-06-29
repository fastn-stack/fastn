extern crate self as fastn_js;

mod ast;
mod component;
mod component_invocation;
mod component_statement;
mod conditional_component;
mod event;
mod mutable_variable;
mod property;
mod ssr;
mod static_variable;
mod to_js;
mod udf;
mod udf_statement;
pub mod utils;

pub use ast::Ast;
pub use component::{component0, component1, component2, component_with_params, Component};
pub use component_invocation::{ElementKind, InstantiateComponent, Kernel};
pub use component_statement::ComponentStatement;
pub use conditional_component::ConditionalComponent;
pub use event::{Event, EventHandler, Function};
pub use mutable_variable::{mutable_quoted, mutable_unquoted, MutableVariable};
pub use property::{ConditionalValue, Formula, PropertyKind, SetProperty, SetPropertyValue, Value};
pub use ssr::{ssr, ssr_str};
pub use static_variable::{static_quoted, static_unquoted, StaticVariable};
pub use to_js::to_js;
pub use udf::{udf0, udf1, udf2, udf_with_params, UDF};
pub use udf_statement::UDFStatement;

pub fn all_js() -> String {
    let fastn_js = include_str!("../fastn.js");
    let dom_js = include_str!("../dom.js");
    let utils_js = include_str!("../utils.js");
    let virtual_js = include_str!("../virtual.js");
    let variables_js = include_str!("../variables.js");
    format!("{fastn_js}{dom_js}{utils_js}{virtual_js}{variables_js}")
}
