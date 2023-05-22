#[derive(Debug)]
pub enum Ast {
    Func(fastn_wasm::Func),
    Import(fastn_wasm::Import),
    Export(fastn_wasm::Export),
    // Table(fastn_wasm::Table),
    // Memory(fastn_wasm::Memory),
    // Global(fastn_wasm::Global),
    // Type(fastn_wasm::Type),
    // Start(fastn_wasm::Start),
}

impl Ast {
    pub fn to_wat(&self) -> String {
        match self {
            Ast::Func(f) => f.to_wat(),
            Ast::Import(i) => i.to_wat(),
            Ast::Export(e) => e.to_wat(),
        }
    }
}
