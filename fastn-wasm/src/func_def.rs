#[derive(Debug)]
pub struct FuncDef {
    name: String,
    decl: fastn_wasm::FuncDecl,
}

pub fn func_def(
    name: &str,
    params: Vec<fastn_wasm::PL>,
    result: Option<fastn_wasm::Type>,
) -> fastn_wasm::Ast {
    fastn_wasm::Ast::FuncDef(FuncDef {
        name: name.to_string(),
        decl: fastn_wasm::FuncDecl {
            name: None,
            params,
            result,
        },
    })
}

pub fn func1(name: &str, arg1: fastn_wasm::PL) -> fastn_wasm::Ast {
    func_def(name, vec![arg1], None)
}

pub fn func1ret(name: &str, arg1: fastn_wasm::PL, ret: fastn_wasm::Type) -> fastn_wasm::Ast {
    func_def(name, vec![arg1], Some(ret))
}

pub fn func2ret(
    name: &str,
    arg1: fastn_wasm::PL,
    arg2: fastn_wasm::PL,
    ret: fastn_wasm::Type,
) -> fastn_wasm::Ast {
    func_def(name, vec![arg1, arg2], Some(ret))
}

impl FuncDef {
    pub fn to_doc(&self) -> pretty::RcDoc<'static> {
        fastn_wasm::group(
            "type".to_string(),
            Some(pretty::RcDoc::text(format!("${}", self.name))),
            self.decl.to_doc(),
        )
    }

    pub fn to_wat(&self) -> String {
        let mut s = String::new();

        s.push_str("(type $");
        s.push_str(self.name.as_str());
        s.push(' ');
        s.push_str(self.decl.to_wat().as_str());
        s.push(')');

        s
    }
}

#[cfg(test)]
mod test {
    #[track_caller]
    fn e(f: fastn_wasm::Ast, s: &str) {
        let g = fastn_wasm::encode_new(&vec![f]);
        println!("got: {}", g);
        println!("expected: {}", s);
        assert_eq!(g, s);
    }

    #[test]
    fn test() {
        e(
            super::func1ret(
                "return_externref",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::ExternRef,
            ),
            "(module (type $return_externref (func (param externref) (result externref))))",
        );
    }
}
