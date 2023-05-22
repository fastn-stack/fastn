#[derive(Debug)]
pub struct Func {
    pub name: Option<String>,
    pub export: Option<String>,
    pub params: Vec<fastn_wasm::PL>,
    pub locals: Vec<fastn_wasm::PL>,
    pub result: Option<fastn_wasm::Type>,
    pub body: Vec<fastn_wasm::Expression>,
}


impl Func {
    pub fn to_wat(&self) -> String {
        let mut s = String::new();
        s.push_str("(func");
        if let Some(name) = &self.name {
            s.push_str(" $");
            s.push_str(name);
        }
        if let Some(export) = &self.export {
            s.push_str(" (export \"");
            s.push_str(export);
            s.push_str("\")");
        }
        if !self.params.is_empty() {
            for param in self.params.iter() {
                s.push_str(" ");
                s.push_str(param.to_wat(true).as_str());
            }
        }
        if !self.locals.is_empty() {
            s.push_str(" (local");
            for local in &self.locals {
                s.push_str(" ");
                s.push_str(&local.ty.to_wat());
            }
            s.push_str(")");
        }
        if let Some(result) = &self.result {
            s.push_str(" (result ");
            s.push_str(&result.to_wat());
            s.push_str(")");
        }
        for ast in &self.body {
            s.push_str(" ");
            s.push_str(&ast.to_wat());
        }
        s.push_str(")");
        s
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(
            fastn_wasm::Func {
                name: None,
                export: None,
                params: vec![],
                locals: vec![],
                result: None,
                body: vec![],
            }.to_wat(),
            "(func)"
        );
        assert_eq!(
            fastn_wasm::Func {
                name: Some("foo".to_string()),
                export: None,
                params: vec![],
                locals: vec![],
                result: None,
                body: vec![],
            }.to_wat(),
            "(func $foo)"
        );
        assert_eq!(
            fastn_wasm::Func {
                name: None,
                export: Some("foo".to_string()),
                params: vec![],
                locals: vec![],
                result: None,
                body: vec![],
            }.to_wat(),
            r#"(func (export "foo"))"#
        );
        assert_eq!(
            fastn_wasm::Func {
                name: Some("foo".to_string()),
                export: Some("foo".to_string()),
                params: vec![],
                locals: vec![],
                result: None,
                body: vec![],
            }.to_wat(),
            r#"(func $foo (export "foo"))"#
        );
        assert_eq!(
            fastn_wasm::Func {
                name: None,
                export: None,
                params: vec![fastn_wasm::Type::I32.into()],
                locals: vec![],
                result: None,
                body: vec![],
            }.to_wat(),
            "(func (param i32))"
        );
        assert_eq!(
            fastn_wasm::Func {
                name: None,
                export: None,
                params: vec![fastn_wasm::Type::I32.into(), fastn_wasm::Type::I64.into()],
                locals: vec![],
                result: None,
                body: vec![],
            }.to_wat(),
            "(func (param i32) (param i64))"
        );
        assert_eq!(
            fastn_wasm::Func {
                name: None,
                export: None,
                params: vec![fastn_wasm::PL {
                    name: Some("foo".to_string()),
                    ty: fastn_wasm::Type::I32,
                }, fastn_wasm::PL {
                    name: Some("bar".to_string()),
                    ty: fastn_wasm::Type::F32,
                }],
                locals: vec![],
                result: None,
                body: vec![],
            }.to_wat(),
            "(func (param $foo i32) (param $bar f32))"
        );
    }
}