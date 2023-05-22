#[derive(Debug, Default)]
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
        for param in self.params.iter() {
            s.push_str(" ");
            s.push_str(param.to_wat(true).as_str());
        }
        for local in self.locals.iter() {
            s.push_str(" ");
            s.push_str(local.to_wat(false).as_str());
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
        #[cfg(test)]
        {
            s = wasmfmt::fmt(&s, wasmfmt::Options {resolve_names: false}).replace("\t", "    ");
        }
        s
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(
            fastn_wasm::Func::default().to_wat(),
            indoc::indoc!(r#"
                (module
                    (func)
                )
            "#)
        );
        assert_eq!(
            fastn_wasm::Func {
                name: Some("foo".to_string()),
                ..Default::default()
            }
            .to_wat(),
            indoc::indoc!(r#"
                (module
                    (func $foo)
                )
            "#)
        );
        assert_eq!(
            fastn_wasm::Func {
                export: Some("foo".to_string()),
                ..Default::default()
            }
            .to_wat(),
            indoc::indoc!(r#"
                (module
                    (func (export "foo"))
                )
            "#)
        );
        assert_eq!(
            fastn_wasm::Func {
                name: Some("foo".to_string()),
                export: Some("foo".to_string()),
                ..Default::default()
            }
            .to_wat(),
            indoc::indoc!(r#"
                (module
                    (func $foo (export "foo"))
                )
            "#)
        );
        assert_eq!(
            fastn_wasm::Func {
                params: vec![fastn_wasm::Type::I32.into()],
                ..Default::default()
            }
            .to_wat(),
            indoc::indoc!(r#"
                (module
                    (func (param i32))
                )
            "#)
        );
        assert_eq!(
            fastn_wasm::Func {
                params: vec![fastn_wasm::Type::I32.into(), fastn_wasm::Type::I64.into()],
                ..Default::default()
            }
            .to_wat(),
            indoc::indoc!(r#"
                (module
                    (func (param i32 i64))
                )
            "#)
        );
        assert_eq!(
            fastn_wasm::Func {
                params: vec![
                    fastn_wasm::PL {
                        name: Some("foo".to_string()),
                        ty: fastn_wasm::Type::I32,
                    },
                    fastn_wasm::PL {
                        name: Some("bar".to_string()),
                        ty: fastn_wasm::Type::F32,
                    }
                ],
                ..Default::default()
            }
            .to_wat(),
            "(func (param $foo i32) (param $bar f32))"
        );
        assert_eq!(
            fastn_wasm::Func {
                locals: vec![
                    fastn_wasm::PL {
                        name: Some("foo".to_string()),
                        ty: fastn_wasm::Type::I32,
                    },
                    fastn_wasm::PL {
                        name: Some("bar".to_string()),
                        ty: fastn_wasm::Type::F32,
                    }
                ],
                ..Default::default()
            }
            .to_wat(),
            "(func (local $foo i32) (local $bar f32))"
        );
        assert_eq!(
            fastn_wasm::Func {
                locals: vec![
                    fastn_wasm::PL {
                        name: Some("foo".to_string()),
                        ty: fastn_wasm::Type::I32,
                    },
                ],
                params: vec![
                    fastn_wasm::PL {
                        name: Some("bar".to_string()),
                        ty: fastn_wasm::Type::F32,
                    },
                ],
                ..Default::default()
            }
                .to_wat(),
            "(func (param $bar f32) (local $foo i32))"
        );
        assert_eq!(
            fastn_wasm::Func {
                result: Some(fastn_wasm::Type::I32),
                ..Default::default()
            }
                .to_wat(),
            "(func (result i32))"
        );
        assert_eq!(
            fastn_wasm::Func {
                name: Some("name".to_string()),
                export: Some("exp".to_string()),
                locals: vec![
                    fastn_wasm::PL {
                        name: Some("foo".to_string()),
                        ty: fastn_wasm::Type::I32,
                    },
                ],
                params: vec![
                    fastn_wasm::PL {
                        name: Some("bar".to_string()),
                        ty: fastn_wasm::Type::F32,
                    },
                ],
                result: Some(fastn_wasm::Type::I32),
                body: vec![],
            }
                .to_wat(),
            r#"(func $name (export "exp") (param $bar f32) (local $foo i32) (result i32))"#
        );
    }
}
