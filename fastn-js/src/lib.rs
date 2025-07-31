#![deny(unused_crate_dependencies)]

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
mod or_type;
mod property;
mod record;
mod ssr;
mod static_variable;
mod to_js;
mod udf;
mod udf_statement;
pub mod utils;

pub use ast::Ast;
pub use component::{Component, component_with_params, component0, component1, component2};
pub use component_invocation::{
    ElementKind, InstantiateComponent, InstantiateComponentData, Kernel,
};
pub use component_statement::ComponentStatement;
pub use conditional_component::ConditionalComponent;
pub use constants::*;
pub use device::{DeviceBlock, DeviceType};
pub use event::{Event, EventHandler, Function, FunctionData};
pub use loop_component::ForLoop;
pub use mutable_variable::{MutableList, MutableVariable, mutable_integer, mutable_string};
pub use or_type::OrType;
pub use property::{
    ConditionalValue, Formula, FormulaType, PropertyKind, SetProperty, SetPropertyValue, Value,
};
pub use record::RecordInstance;
pub use ssr::{SSRError, run_test, ssr, ssr_raw_string_without_test, ssr_str, ssr_with_js_string};
pub use static_variable::{StaticVariable, static_integer, static_string};
pub use to_js::to_js;
pub use udf::{UDF, udf_with_arguments};
pub use udf_statement::UDFStatement;

pub fn fastn_assertion_headers(http_status_code: u16, http_location: &str) -> String {
    format!(
        indoc::indoc! {"
            fastn.http_status = {http_status};
            fastn.http_location = \"{http_location}\";
        "},
        http_status = http_status_code,
        http_location = http_location
    )
}

pub fn fastn_test_js() -> &'static str {
    include_str!("../js/fastn_test.js")
}

pub fn all_js_without_test_and_ftd_langugage_js() -> String {
    let markdown_js = fastn_js::markdown_js();
    // Core JS files - order is important for dependencies
    let benchmark_utils_js = include_str_with_debug!("../js/benchmark-utils.js");
    let fastn_js = include_str_with_debug!("../js/fastn.js");
    let dom_js = include_str_with_debug!("../js/dom.js");
    let utils_js = include_str_with_debug!("../js/utils.js");
    let virtual_js = include_str_with_debug!("../js/virtual.js");
    let ftd_js = include_str_with_debug!("../js/ftd.js");
    let web_component_js = include_str_with_debug!("../js/web-component.js");
    let post_init_js = include_str_with_debug!("../js/postInit.js");

    // the order is important
    // benchmark-utils must come first to define fastn_perf
    // global variable defined in dom_js might be read in virtual_js
    format!(
        "{benchmark_utils_js}{markdown_js}{fastn_js}{dom_js}{utils_js}{virtual_js}{web_component_js}{ftd_js}{post_init_js}"
    )
}

#[macro_export]
macro_rules! include_str_with_debug {
    ($name:expr) => {{
        let default = include_str!($name);
        if std::env::var("DEBUG").is_ok() {
            std::fs::read_to_string($name).unwrap_or_else(|_| default.to_string())
        } else {
            default.to_string()
        }
    }};
}

pub fn all_js_without_test() -> String {
    let fastn_js = all_js_without_test_and_ftd_langugage_js();
    let ftd_language_js = include_str!("../js/ftd-language.js");
    format!("{ftd_language_js}{fastn_js}\nwindow.ftd = ftd;\n")
}

pub fn all_js_with_test() -> String {
    let test_js = include_str!("../js/test.js");
    let all_js = all_js_without_test_and_ftd_langugage_js();
    format!("{all_js}{test_js}")
}

pub fn markdown_js() -> &'static str {
    include_str!("../js/vendor/marked.js")
}

pub fn prism_css() -> String {
    let prism_line_highlight = include_str!("../js/vendor/prism/prism-line-highlight.css");
    let prism_line_numbers = include_str!("../js/vendor/prism/prism-line-numbers.css");
    format!("{prism_line_highlight}{prism_line_numbers}")
}

pub fn prism_js() -> String {
    let prism = include_str!("../js/vendor/prism/prism.js");
    let prism_line_highlight = include_str!("../js/vendor/prism/prism-line-highlight.js");
    let prism_line_numbers = include_str!("../js/vendor/prism/prism-line-numbers.js");

    // Languages supported
    // Rust, Json, Python, Markdown, SQL, Bash, JavaScript
    let prism_rust = include_str!("../js/vendor/prism/prism-rust.js");
    let prism_json = include_str!("../js/vendor/prism/prism-json.js");
    let prism_python = include_str!("../js/vendor/prism/prism-python.js");
    let prism_markdown = include_str!("../js/vendor/prism/prism-markdown.js");
    let prism_sql = include_str!("../js/vendor/prism/prism-sql.js");
    let prism_bash = include_str!("../js/vendor/prism/prism-bash.js");
    let prism_javascript = include_str!("../js/vendor/prism/prism-javascript.js");
    let prism_diff = include_str!("../js/vendor/prism/prism-diff.js");

    format!(
        "{prism}{prism_line_highlight}{prism_line_numbers}{prism_rust}{prism_json}{prism_python\
        }{prism_markdown}{prism_sql}{prism_bash}{prism_javascript}{prism_diff}"
    )
}

pub fn ftd_js_css() -> &'static str {
    include_str!("../ftd-js.css")
}
