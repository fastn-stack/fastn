#[derive(Debug, Default, Clone)]
pub struct Func {
    pub name: Option<String>,
    pub export: Option<String>,
    pub params: Vec<fastn_wasm::PL>,
    pub locals: Vec<fastn_wasm::PL>,
    pub result: Option<fastn_wasm::Type>,
    pub body: Vec<fastn_wasm::Expression>,
}

impl Func {
    pub fn to_doc(&self) -> pretty::RcDoc<()> {
        let mut name = self
            .name
            .clone()
            .map(|n| pretty::RcDoc::text(format!("${}", n)));

        if let Some(export) = &self.export {
            let exp = fastn_wasm::named(
                "export",
                Some(pretty::RcDoc::text(format!("\"{}\"", export))),
            );

            name = match name {
                Some(n) => Some(n.append(pretty::RcDoc::space().append(exp))),
                None => Some(exp),
            }
        };

        let mut v: Vec<pretty::RcDoc<()>> = vec![];

        if !self.params.is_empty() {
            v.push(
                pretty::RcDoc::intersperse(
                    self.params.iter().map(|x| x.to_doc(true)),
                    pretty::RcDoc::line(),
                )
                .group(),
            );
        }

        if let Some(result) = &self.result {
            v.push(fastn_wasm::group(
                "result".to_string(),
                None,
                result.to_doc(),
            ))
        };

        if !self.locals.is_empty() {
            v.push(
                pretty::RcDoc::intersperse(
                    self.locals.iter().map(|x| x.to_doc(false)),
                    pretty::RcDoc::line(),
                )
                .group(),
            );
        }

        if !self.body.is_empty() {
            v.push(
                pretty::RcDoc::intersperse(
                    self.body.iter().map(|x| x.to_doc()),
                    pretty::RcDoc::line(),
                )
                .group(),
            );
        }

        if v.is_empty() {
            fastn_wasm::named("func", name)
        } else {
            fastn_wasm::group(
                "func".to_string(),
                name,
                pretty::RcDoc::intersperse(v, pretty::Doc::line()),
            )
        }
        .group()
        .nest(1)
    }

    pub fn to_ast(self) -> fastn_wasm::Ast {
        fastn_wasm::Ast::Func(self)
    }

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
            s.push(' ');
            s.push_str(param.to_wat(true).as_str());
        }
        if let Some(result) = &self.result {
            s.push_str(" (result ");
            s.push_str(result.to_wat());
            s.push(')');
        }
        for local in self.locals.iter() {
            s.push(' ');
            s.push_str(local.to_wat(false).as_str());
        }
        for ast in &self.body {
            s.push(' ');
            s.push_str(&ast.to_wat());
        }
        s.push(')');

        s
    }
}

#[derive(Debug, Default)]
pub struct FuncDecl {
    pub name: Option<String>,
    pub params: Vec<fastn_wasm::PL>,
    pub result: Option<fastn_wasm::Type>,
}

impl FuncDecl {
    pub fn to_wat(&self) -> String {
        fastn_wasm::Func {
            name: self.name.to_owned(),
            params: self.params.to_owned(),
            result: self.result.to_owned(),
            ..Default::default()
        }
        .to_wat()
    }
}

#[cfg(test)]
mod test {
    use super::Func;

    #[track_caller]
    fn e(f: super::Func, s: &str) {
        let g = fastn_wasm::encode_new(&vec![fastn_wasm::Ast::Func(f)]);
        println!("got: {}", g);
        println!("expected: {}", s);
        assert_eq!(g, s);
    }

    #[test]
    fn test() {
        e(Func::default(), "(module (func))");
        e(
            Func {
                name: Some("foo".to_string()),
                ..Default::default()
            },
            "(module (func $foo))",
        );
        e(
            Func {
                export: Some("foo".to_string()),
                ..Default::default()
            },
            r#"(module (func (export "foo")))"#,
        );
        e(
            Func {
                name: Some("foo".to_string()),
                export: Some("foo".to_string()),
                ..Default::default()
            },
            r#"(module (func $foo (export "foo")))"#,
        );
        e(
            Func {
                params: vec![fastn_wasm::Type::I32.into()],
                ..Default::default()
            },
            "(module (func (param i32)))",
        );
        e(
            Func {
                params: vec![fastn_wasm::Type::I32.into(), fastn_wasm::Type::I64.into()],
                ..Default::default()
            },
            "(module (func (param i32) (param i64)))",
        );
        e(
            Func {
                params: vec![
                    fastn_wasm::PL {
                        name: Some("foo".to_string()),
                        ty: fastn_wasm::Type::I32,
                    },
                    fastn_wasm::PL {
                        name: Some("bar".to_string()),
                        ty: fastn_wasm::Type::F32,
                    },
                ],
                ..Default::default()
            },
            "(module (func (param $foo i32) (param $bar f32)))",
        );
        e(
            Func {
                locals: vec![
                    fastn_wasm::PL {
                        name: Some("foo".to_string()),
                        ty: fastn_wasm::Type::I32,
                    },
                    fastn_wasm::PL {
                        name: Some("bar".to_string()),
                        ty: fastn_wasm::Type::F32,
                    },
                ],
                ..Default::default()
            },
            "(module (func (local $foo i32) (local $bar f32)))",
        );
        e(
            Func {
                locals: vec![fastn_wasm::PL {
                    name: Some("foo".to_string()),
                    ty: fastn_wasm::Type::I32,
                }],
                params: vec![fastn_wasm::PL {
                    name: Some("bar".to_string()),
                    ty: fastn_wasm::Type::F32,
                }],
                ..Default::default()
            },
            "(module (func (param $bar f32) (local $foo i32)))",
        );
        e(
            Func {
                result: Some(fastn_wasm::Type::I32),
                ..Default::default()
            },
            "(module (func (result i32)))",
        );
        e(
            Func {
                name: Some("name".to_string()),
                export: Some("exp".to_string()),
                locals: vec![fastn_wasm::PL {
                    name: Some("foo".to_string()),
                    ty: fastn_wasm::Type::I32,
                }],
                params: vec![fastn_wasm::PL {
                    name: Some("bar".to_string()),
                    ty: fastn_wasm::Type::F32,
                }],
                result: Some(fastn_wasm::Type::I32),
                body: vec![],
            },
            indoc::indoc!(
                r#"
                (module (func $name (export "exp") (param $bar f32)
                  (result i32)
                  (local $foo i32)))"#
            ),
        );
        e(
            Func {
                params: vec![fastn_wasm::Type::I32.into(), fastn_wasm::Type::I32.into()],
                result: Some(fastn_wasm::Type::I32),
                body: vec![fastn_wasm::Expression::Operation {
                    name: "i32.add".to_string(),
                    values: vec![
                        fastn_wasm::Expression::LocalGet { index: 0.into() },
                        fastn_wasm::Expression::LocalGet { index: 1.into() },
                    ],
                }],
                ..Default::default()
            },
            indoc::indoc!(
                r#"
                    (module (func (param i32) (param i32)
                      (result i32)
                      (i32.add (local.get 0) (local.get 1))))"#,
            ),
        );
        e(
            Func {
                params: vec![
                    fastn_wasm::PL {
                        name: Some("lhs".to_string()),
                        ty: fastn_wasm::Type::I32,
                    },
                    fastn_wasm::PL {
                        name: Some("rhs".to_string()),
                        ty: fastn_wasm::Type::I32,
                    },
                ],
                result: Some(fastn_wasm::Type::I32),
                body: vec![fastn_wasm::Expression::Operation {
                    name: "i32.add".to_string(),
                    values: vec![
                        fastn_wasm::Expression::LocalGet {
                            index: "lhs".into(),
                        },
                        fastn_wasm::Expression::LocalGet {
                            index: "rhs".into(),
                        },
                    ],
                }],
                ..Default::default()
            },
            indoc::indoc!(
                r#"
                (module (func (param $lhs i32) (param $rhs i32)
                  (result i32)
                  (i32.add (local.get $lhs) (local.get $rhs))))"#
            ),
        );
        e(
            Func {
                export: Some("main".to_string()),
                locals: vec![
                    fastn_wasm::PL {
                        name: Some("column".to_string()),
                        ty: fastn_wasm::Type::I32,
                    },
                    fastn_wasm::PL {
                        name: Some("root".to_string()),
                        ty: fastn_wasm::Type::I32,
                    },
                ],
                result: Some(fastn_wasm::Type::I32),
                body: vec![
                    fastn_wasm::Expression::LocalSet {
                        index: "root".into(),
                        value: Box::new(fastn_wasm::Expression::Call {
                            name: "root_container".to_string(),
                            params: vec![],
                        }),
                    },
                    fastn_wasm::Expression::Call {
                        name: "foo".to_string(),
                        params: vec![
                            fastn_wasm::Expression::LocalGet {
                                index: "root".into(),
                            },
                            fastn_wasm::Expression::I32Const(100),
                            fastn_wasm::Expression::I32Const(100),
                        ],
                    },
                    fastn_wasm::Expression::Drop,
                    fastn_wasm::Expression::Call {
                        name: "foo".to_string(),
                        params: vec![
                            fastn_wasm::Expression::LocalGet {
                                index: "root".into(),
                            },
                            fastn_wasm::Expression::I32Const(200),
                            fastn_wasm::Expression::I32Const(300),
                        ],
                    },
                    fastn_wasm::Expression::Drop,
                ],
                ..Default::default()
            },
            indoc::indoc!(
                r#"
                (module (func (export "main") (result i32)
                  (local $column i32) (local $root i32)
                  (local.set $root (call $root_container ))
                  (call $foo (local.get $root)
                  (i32.const $100)
                  (i32.const $100))
                  (drop)
                  (call $foo (local.get $root)
                  (i32.const $200)
                  (i32.const $300))
                  (drop)))"#
            ),
        );
    }
}
