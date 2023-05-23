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

pub fn local(name: &str) -> fastn_wasm::Expression {
    fastn_wasm::Expression::LocalGet {
        index: name.into(),
    }
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

pub fn import_func00(name: &str) -> fastn_wasm::Ast {
    import_func(name, vec![], None)
}

pub fn call3(name: &str, e0: fastn_wasm::Expression, e1: fastn_wasm::Expression, e2: fastn_wasm::Expression) -> fastn_wasm::Expression {
    fastn_wasm::Expression::Call {
        name: name.into(),
        params: vec![e0, e1, e2],
    }
}

pub fn exported_func1(name: &str, arg0: fastn_wasm::PL, body: Vec<fastn_wasm::Expression>) -> fastn_wasm::Ast {
    fastn_wasm::Ast::Func(fastn_wasm::Func {
        export: Some(name.to_string()),
        params: vec![arg0],
        body,
        ..Default::default()
    })
}

pub fn import_func0(name: &str, result: fastn_wasm::Type) -> fastn_wasm::Ast {
    import_func(name, vec![], Some(result))
}

pub fn import_func1(name: &str, arg0: fastn_wasm::PL) -> fastn_wasm::Ast {
    import_func(name, vec![arg0], None)
}

pub fn import_func2(name: &str, arg0: fastn_wasm::PL, arg1: fastn_wasm::PL) -> fastn_wasm::Ast {
    import_func(name, vec![arg0, arg1], None)
}

pub fn import_func(name: &str, params: Vec<fastn_wasm::PL>, result: Option<fastn_wasm::Type>) -> fastn_wasm::Ast {
    fastn_wasm::Ast::Import(fastn_wasm::Import {
        module: "fastn".to_string(),
        name: name.to_string(),
        desc: fastn_wasm::ImportDesc::Func(fastn_wasm::FuncDecl {
            name: Some(name.to_string()),
            params,
            result,
        }),
    })
}
