extern crate self as fastn_js;

mod func;
mod instruction;
mod ssr;

pub use func::{func0, Func};
pub use instruction::{Instruction, StaticVariable};
pub use ssr::{ssr, ssr_str};

pub fn encode(js: &[fastn_js::Func]) -> String {
    let mut w = Vec::new();
    let o = pretty::RcDoc::intersperse(js.iter().map(|f| f.to_js()), pretty::RcDoc::space());
    o.render(80, &mut w).unwrap();
    String::from_utf8(w).unwrap()
}
