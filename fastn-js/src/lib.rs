extern crate self as fastn_js;

mod ast;
mod component;
mod component_invocation;
mod component_statement;
mod conditional_component;
mod constants;
mod device;
mod event;
mod loop_component;
mod mutable_variable;
mod property;
mod record;
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
pub use constants::*;
pub use device::{DeviceBlock, DeviceType};
pub use event::{Event, EventHandler, Function};
pub use loop_component::ForLoop;
pub use mutable_variable::{mutable_integer, mutable_string, MutableList, MutableVariable};
pub use property::{ConditionalValue, Formula, PropertyKind, SetProperty, SetPropertyValue, Value};
pub use record::RecordInstance;
pub use ssr::{ssr, ssr_str};
pub use static_variable::{static_integer, static_string, StaticVariable};
pub use to_js::to_js;
pub use udf::{udf0, udf1, udf2, udf_with_params, UDF};
pub use udf_statement::UDFStatement;

pub fn all_js() -> String {
    let fastn_js = include_str!("../js/fastn.js");
    let dom_js = include_str!("../js/dom.js");
    let utils_js = include_str!("../js/utils.js");
    let virtual_js = include_str!("../js/virtual.js");
    let ftd_js = include_str!("../js/ftd.js");
    let post_init_js = include_str!("../js/postInit.js");
    let test_js = include_str!("../js/test.js");
    format!("{fastn_js}{dom_js}{utils_js}{virtual_js}{ftd_js}{post_init_js}{test_js}")
}
