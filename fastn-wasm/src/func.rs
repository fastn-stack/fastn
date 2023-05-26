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

    #[cfg(test)]
    pub fn to_wat_formatted(&self) -> String {
        wasmfmt::fmt(
            &self.to_wat(),
            wasmfmt::Options {
                resolve_names: false,
            },
        )
        .replace("\t", "    ")
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
    #[test]
    fn test() {
        assert_eq!(
            fastn_wasm::Func::default().to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (func)
                )
            "#
            )
        );
        assert_eq!(
            fastn_wasm::Func {
                name: Some("foo".to_string()),
                ..Default::default()
            }
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (func $foo)
                )
            "#
            )
        );
        assert_eq!(
            fastn_wasm::Func {
                export: Some("foo".to_string()),
                ..Default::default()
            }
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (func (export "foo"))
                )
            "#
            )
        );
        assert_eq!(
            fastn_wasm::Func {
                name: Some("foo".to_string()),
                export: Some("foo".to_string()),
                ..Default::default()
            }
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (func $foo (export "foo"))
                )
            "#
            )
        );
        assert_eq!(
            fastn_wasm::Func {
                params: vec![fastn_wasm::Type::I32.into()],
                ..Default::default()
            }
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (func (param i32))
                )
            "#
            )
        );
        assert_eq!(
            fastn_wasm::Func {
                params: vec![fastn_wasm::Type::I32.into(), fastn_wasm::Type::I64.into()],
                ..Default::default()
            }
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (func (param i32 i64))
                )
            "#
            )
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
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (func (param $foo i32) (param $bar f32))
                )
            "#
            )
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
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (func
                        (local $foo i32)
                        (local $bar f32)
                    )
                )
            "#
            )
        );
        assert_eq!(
            fastn_wasm::Func {
                locals: vec![fastn_wasm::PL {
                    name: Some("foo".to_string()),
                    ty: fastn_wasm::Type::I32,
                },],
                params: vec![fastn_wasm::PL {
                    name: Some("bar".to_string()),
                    ty: fastn_wasm::Type::F32,
                },],
                ..Default::default()
            }
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (func (param $bar f32)
                        (local $foo i32)
                    )
                )
            "#
            )
        );
        assert_eq!(
            fastn_wasm::Func {
                result: Some(fastn_wasm::Type::I32),
                ..Default::default()
            }
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (func (result i32))
                )
            "#
            )
        );
        assert_eq!(
            fastn_wasm::Func {
                name: Some("name".to_string()),
                export: Some("exp".to_string()),
                locals: vec![fastn_wasm::PL {
                    name: Some("foo".to_string()),
                    ty: fastn_wasm::Type::I32,
                },],
                params: vec![fastn_wasm::PL {
                    name: Some("bar".to_string()),
                    ty: fastn_wasm::Type::F32,
                },],
                result: Some(fastn_wasm::Type::I32),
                body: vec![],
            }
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (func $name (export "exp") (param $bar f32) (result i32)
                        (local $foo i32)
                    )
                )
            "#
            )
        );
        assert_eq!(
            fastn_wasm::Func {
                params: vec![fastn_wasm::Type::I32.into(), fastn_wasm::Type::I32.into()],
                result: Some(fastn_wasm::Type::I32),
                body: vec![fastn_wasm::Expression::Operations {
                    name: "i32.add".to_string(),
                    values: vec![
                        fastn_wasm::Expression::LocalGet { index: 0.into() },
                        fastn_wasm::Expression::LocalGet { index: 1.into() },
                    ],
                }],
                ..Default::default()
            }
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (func (param i32 i32) (result i32)
                        (local.get 0)
                        (local.get 1)
                        i32.add
                    )
                )
            "#
            )
        );
        assert_eq!(
            fastn_wasm::Func {
                params: vec![
                    fastn_wasm::PL {
                        name: Some("lhs".to_string()),
                        ty: fastn_wasm::Type::I32
                    },
                    fastn_wasm::PL {
                        name: Some("rhs".to_string()),
                        ty: fastn_wasm::Type::I32
                    },
                ],
                result: Some(fastn_wasm::Type::I32),
                body: vec![fastn_wasm::Expression::Operations {
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
            }
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (func (param $lhs i32) (param $rhs i32) (result i32)
                        (local.get $lhs)
                        (local.get $rhs)
                        i32.add
                    )
                )
            "#
            )
        );
        assert_eq!(
            fastn_wasm::Func {
                export: Some("main".to_string()),
                locals: vec![
                    fastn_wasm::PL {
                        name: Some("column".to_string()),
                        ty: fastn_wasm::Type::I32
                    },
                    fastn_wasm::PL {
                        name: Some("root".to_string()),
                        ty: fastn_wasm::Type::I32
                    },
                ],
                result: Some(fastn_wasm::Type::I32),
                body: vec![
                    fastn_wasm::Expression::LocalSet {
                        index: "root".into(),
                        value: Box::new(fastn_wasm::Expression::Call {
                            name: "root_container".to_string(),
                            params: vec![]
                        }),
                    },
                    fastn_wasm::Expression::Call {
                        name: "foo".to_string(),
                        params: vec![
                            fastn_wasm::Expression::LocalGet {
                                index: "root".into()
                            },
                            fastn_wasm::Expression::I32Const(100),
                            fastn_wasm::Expression::I32Const(100)
                        ]
                    },
                    fastn_wasm::Expression::Drop,
                    fastn_wasm::Expression::Call {
                        name: "foo".to_string(),
                        params: vec![
                            fastn_wasm::Expression::LocalGet {
                                index: "root".into()
                            },
                            fastn_wasm::Expression::I32Const(200),
                            fastn_wasm::Expression::I32Const(300)
                        ]
                    },
                    fastn_wasm::Expression::Drop,
                ],
                ..Default::default()
            }
            .to_wat_formatted(),
            indoc::indoc!(
                r#"
                (module
                    (func (export "main") (result i32)
                        (local $column i32)
                        (local $root i32)
                        (call $root_container)
                        (local.set $root)
                        (local.get $root)
                        (i32.const 100)
                        (i32.const 100)
                        (call $foo)
                        drop
                        (local.get $root)
                        (i32.const 200)
                        (i32.const 300)
                        (call $foo)
                        drop
                    )
                )
            "#
            )
        );
    }
}
