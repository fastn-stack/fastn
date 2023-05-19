fn ast_to_wat(ast: Vec<ftd::wasm::Ast>) -> String {
    let mut wat = String::new();
    for a in ast {
        wat.push_str(&a.to_wat());
    }
    wat
}

impl ftd::wasm::Ast {
    pub fn to_wat(&self) -> String {
        match self {
            ftd::wasm::Ast::Func(f) => f.to_wat(),
        }
    }
}

impl ftd::wasm::Func {
    pub fn to_wat(&self) -> String {
        let mut wat = String::from("(func ");
        if let Some(name) = &self.name {
            wat.push_str(&format!(" $\"{}\" ", name));
        }
        if let Some(export) = &self.export {
            wat.push_str(&format!(" (export \"{}\")", export));
        }
        for p in &self.params {
            wat.push_str(&format!(" {}", p.to_wat()));
        }
        for l in &self.locals {
            wat.push_str(&format!(" {}", l.to_wat()));
        }
        for b in &self.body {
            wat.push_str(&format!(" {}", b.to_wat()));
        }
        wat.push_str(")");
        wat
    }
}