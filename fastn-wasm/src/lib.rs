extern crate self as fastn_wasm;

mod ast;
// mod encoder;
mod export;
mod expression;
mod func;
mod import;
mod pl;
mod table;
mod ty;

pub use ast::*;
pub use export::{Export, ExportDesc};
pub use expression::{Expression, Index};
pub use func::{Func, FuncDecl};
pub use import::{Import, ImportDesc};
pub use pl::PL;
pub use table::{Limits, RefType, Table};
pub use ty::Type;

pub fn encode(module: &[fastn_wasm::Ast]) -> String {
    let mut s = String::new();
    for node in module {
        s.push_str(&node.to_wat());
    }
    s
}
