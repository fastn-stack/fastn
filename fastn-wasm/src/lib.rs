extern crate self as fastn_wasm;

mod ast;
// mod encoder;
mod elem;
mod export;
mod expression;
mod func;
pub mod func_def;
mod global;
mod helpers;
pub mod import;
mod memory;
mod pl;
mod table;
mod ty;

pub use ast::*;
pub use elem::Elem;
pub use export::{Export, ExportDesc};
pub use expression::{Expression, Index};
pub use func::{Func, FuncDecl};
pub use func_def::FuncDef;
pub use global::{global, Global};
pub use helpers::{LinkerExt, StoreExtractor, WasmType};
pub use import::{Import, ImportDesc};
pub use memory::Memory;
pub use pl::PL;
pub use table::{table, table_1, table_2, table_3, table_4, Limits, RefType, Table};
pub use ty::Type;

pub fn encode(module: &[fastn_wasm::Ast]) -> String {
    let mut s = String::new();
    s.push_str("(module\n");
    for node in module {
        s.push_str(&node.to_wat());
        s.push('\n');
    }
    s.push(')');
    s
}

pub fn local(name: &str) -> fastn_wasm::Expression {
    fastn_wasm::Expression::LocalGet { index: name.into() }
}

pub fn local_set(name: &str, e: fastn_wasm::Expression) -> fastn_wasm::Expression {
    fastn_wasm::Expression::LocalSet {
        index: name.into(),
        value: Box::new(e),
    }
}

pub fn i32(i: i32) -> fastn_wasm::Expression {
    fastn_wasm::Expression::I32Const(i)
}

pub fn call3(
    name: &str,
    e0: fastn_wasm::Expression,
    e1: fastn_wasm::Expression,
    e2: fastn_wasm::Expression,
) -> fastn_wasm::Expression {
    fastn_wasm::Expression::Call {
        name: name.into(),
        params: vec![e0, e1, e2],
    }
}

pub fn call(name: &str) -> fastn_wasm::Expression {
    fastn_wasm::Expression::Call {
        name: name.into(),
        params: vec![],
    }
}

pub fn exported_func1(
    name: &str,
    arg0: fastn_wasm::PL,
    body: Vec<fastn_wasm::Expression>,
) -> fastn_wasm::Ast {
    fastn_wasm::Ast::Func(fastn_wasm::Func {
        export: Some(name.to_string()),
        params: vec![arg0],
        body,
        ..Default::default()
    })
}
