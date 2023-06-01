#[derive(Debug)]
pub struct Global {
    pub name: String,
    pub ty: fastn_wasm::Type,
}

pub fn global(name: &str, ty: fastn_wasm::Type) -> fastn_wasm::Ast {
    fastn_wasm::Ast::Global(Global::new(name, ty))
}

impl Global {
    pub fn new(name: &str, ty: fastn_wasm::Type) -> Global {
        Global {
            name: name.to_string(),
            ty,
        }
    }
    pub fn to_wat(&self) -> String {
        format!("(global ${} {})", self.name, self.ty.to_wat())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(
            super::Global::new("foo", fastn_wasm::Type::ExternRef).to_wat(),
            "(global $foo externref)"
        );
    }
}
