#[derive(Debug)]
pub enum Ast {
    Func(fastn_wasm::Func),
}

impl Ast {
    pub fn to_wat(&self) -> String {
        match self {
            Ast::Func(f) => f.to_wat(),
        }
    }
}


