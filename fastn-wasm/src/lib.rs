extern crate self as fastn_wasm;

mod ast;
// mod encoder;
mod expression;
mod func;
mod pl;
mod ty;

pub use ast::*;
pub use expression::Expression;
pub use func::Func;
pub use pl::PL;
pub use ty::Type;

pub fn encode(module: &[fastn_wasm::Ast]) -> String {
    let mut s = String::new();
    for node in module {
        s.push_str(&node.to_wat());
    }
    s
}
