extern crate self as fastn_wasm;

mod ast;
// mod encoder;
mod elem;
pub mod export;
pub mod expression;
mod func;
pub mod func_def;
mod helpers;
pub mod import;
mod memory;
mod pl;
mod table;
mod ty;

pub use ast::Ast;
pub use elem::Elem;
pub use export::{Export, ExportDesc};
pub use expression::{Expression, Index};
pub use func::{Func, FuncDecl};
pub use func_def::FuncDef;
pub use helpers::{FromToI32, LinkerExt, StoreExtractor, WasmType};
pub use import::{Import, ImportDesc};
pub use memory::Memory;
pub use pl::PL;
pub use table::{table, table_1, table_2, table_3, table_4, Limits, RefType, Table};
pub use ty::Type;

pub fn group<'a>(name: &'static str, body: pretty::RcDoc<'a, ()>) -> pretty::RcDoc<'a, ()> {
    pretty::RcDoc::intersperse(
        vec![
            pretty::RcDoc::text("(").append(name),
            body,
            pretty::RcDoc::text(")"),
        ],
        pretty::Doc::line(),
    )
}
pub fn encode(module: &[fastn_wasm::Ast]) -> String {
    let mut w = Vec::new();
    let o = group(
        "module",
        pretty::RcDoc::intersperse(module.into_iter().map(|x| x.to_doc()), pretty::Doc::line())
            .nest(1)
            .group(),
    );
    o.render(50, &mut w).unwrap();
    let o = String::from_utf8(w).unwrap();
    println!("{}", o);
    o
}
