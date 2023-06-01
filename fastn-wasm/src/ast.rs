#[derive(Debug)]
pub enum Ast {
    Func(fastn_wasm::Func),
    Global(fastn_wasm::Global),
    Import(fastn_wasm::Import),
    Export(fastn_wasm::Export),
    Table(fastn_wasm::Table),
    Memory(fastn_wasm::Memory),
    Elem(fastn_wasm::Elem),
    FuncDef(fastn_wasm::FuncDef),
}

impl Ast {
    pub fn to_wat(&self) -> String {
        match self {
            Ast::Func(f) => f.to_wat(),
            Ast::Import(i) => i.to_wat(),
            Ast::Export(e) => e.to_wat(),
            Ast::Table(t) => t.to_wat(),
            Ast::Memory(m) => m.to_wat(),
            Ast::Global(g) => g.to_wat(),
            Ast::Elem(g) => g.to_wat(),
            Ast::FuncDef(g) => g.to_wat(),
        }
    }
}
