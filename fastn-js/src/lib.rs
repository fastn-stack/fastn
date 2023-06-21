extern crate self as fastn_js;

mod ast;
mod component;
mod component_statement;
mod mutable_variable;
mod ssr;
mod static_variable;
mod udf;

pub use ast::Ast;
pub use component::{component0, component1, component2, Component};
pub use component_statement::ComponentStatement;
pub use mutable_variable::{mutable_quoted, mutable_unquoted, MutableVariable};
pub use ssr::{ssr, ssr_str};
pub use static_variable::{static_quoted, static_unquoted, StaticVariable};
pub use udf::UDF;

pub fn to_js(ast: &[fastn_js::Ast]) -> String {
    let mut w = Vec::new();
    let o = pretty::RcDoc::intersperse(ast.iter().map(|f| f.to_js()), pretty::RcDoc::space());
    o.render(80, &mut w).unwrap();
    String::from_utf8(w).unwrap()
}
