extern crate self as fastn_wasm;

mod ast;
// mod encoder;
mod elem;
pub mod export;
pub mod expression;
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
