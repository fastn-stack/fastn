extern crate self as fastn_wasm;

mod ast;
// mod encoder;
mod export;
mod expression;
mod func;
mod import;
mod memory;
mod pl;
mod table;
mod ty;

pub use ast::*;
pub use export::{Export, ExportDesc};
pub use expression::{Expression, Index};
pub use func::{Func, FuncDecl};
pub use import::{Import, ImportDesc};
pub use memory::Memory;
pub use pl::PL;
pub use table::{Limits, RefType, Table};
pub use ty::Type;

pub fn encode(module: &[fastn_wasm::Ast]) -> String {
    let mut s = String::new();
    s.push_str("(module\n");
    for node in module {
        s.push_str(&node.to_wat());
        s.push_str("\n");
    }
    s.push_str(")");
    s
}

pub fn local_named_get(name: &str) -> fastn_wasm::Expression {
    fastn_wasm::Expression::LocalGet {
        index: name.into(),
    }
}

pub fn local_named_set(name: &str, e: fastn_wasm::Expression) -> fastn_wasm::Expression {
    fastn_wasm::Expression::LocalSet {
        index: name.into(),
        value: Box::new(e),
    }
}
