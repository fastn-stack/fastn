#[tokio::main]
async fn main() {
    // check if --wasm is passed on cli
    let _wat = if std::env::args().any(|arg| arg == "--stdin") {
        use std::io::Read;

        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer).unwrap();
        buffer
    } else {
        r#"
        (module
            (import "fastn" "create_column" (func $create_column (result externref)))
            (import "fastn" "root_container" (func $root_container (result externref)))
            (import "fastn" "set_column_width_px" (func $set_column_width_px (param externref i32)))
            (import "fastn" "set_column_height_px" (func $set_column_height_px (param externref i32)))

            ;; fastn.add_child(parent: NodeKey, child: NodeKey)
            (import "fastn" "add_child" (func $add_child (param externref externref)))

            (func (export "main") (local $column externref) (local $root_container_ externref)
                (local.set $root_container_ (call $root_container))

                ;; -- ftd.column:
                ;; width.fixed.px: 100
                ;; height.fixed.px: 100
                (call $foo (local.get $root_container_) (i32.const 100) (i32.const 100))
                drop

                ;; -- ftd.column:
                (call $foo (local.get $root_container_) (i32.const 200) (i32.const 300))
                drop
            )

            (func $foo
                (param $root externref)
                (param $width i32)
                (param $height i32)

                (result externref)

                (local $column externref)

                ;; body

                (local.set $column (call $create_column))

                (call $add_child (local.get $root) (local.get $column))
                (call $set_column_width_px (local.get $column) (local.get $width))
                (call $set_column_height_px (local.get $column) (local.get $height))

                (local.get $column)
            )
        )
    "#.to_string()
    };

    // let document = fastn_runtime::Document::new(wat);

    let document = fastn_runtime::Document::new(create_module());

    #[cfg(feature = "native")]
    fastn_runtime::wgpu::render_document(document).await;

    // #[cfg(feature = "terminal")]
    // fastn_runtime::terminal::draw(doc).await;
}

fn create_module() -> Vec<u8> {
    let m: Vec<fastn_wasm::Ast> = vec![
        fastn_wasm::Ast::Import(fastn_wasm::Import {
            module: "fastn".to_string(),
            name: "create_column".to_string(),
            desc: fastn_wasm::ImportDesc::Func(fastn_wasm::FuncDecl {
                name: Some("create_column".to_string()),
                params: vec![],
                result: Some(fastn_wasm::Type::ExternRef),
            }),
        }),
        fastn_wasm::Ast::Import(fastn_wasm::Import {
            module: "fastn".to_string(),
            name: "add_child".to_string(),
            desc: fastn_wasm::ImportDesc::Func(fastn_wasm::FuncDecl {
                name: Some("add_child".to_string()),
                params: vec![fastn_wasm::Type::ExternRef.into(), fastn_wasm::Type::ExternRef.into()],
                result: None,
            }),
        }),
        fastn_wasm::Ast::Import(fastn_wasm::Import {
            module: "fastn".to_string(),
            name: "set_column_width_px".to_string(),
            desc: fastn_wasm::ImportDesc::Func(fastn_wasm::FuncDecl {
                name: Some("set_column_width_px".to_string()),
                params: vec![fastn_wasm::Type::ExternRef.into(), fastn_wasm::Type::I32.into()],
                result: None,
            }),
        }),
        fastn_wasm::Ast::Import(fastn_wasm::Import {
            module: "fastn".to_string(),
            name: "set_column_height_px".to_string(),
            desc: fastn_wasm::ImportDesc::Func(fastn_wasm::FuncDecl {
                name: Some("set_column_height_px".to_string()),
                params: vec![fastn_wasm::Type::ExternRef.into(), fastn_wasm::Type::I32.into()],
                result: None,
            }),
        }),
        fastn_wasm::Ast::Func(fastn_wasm::Func {
            export: Some("main".to_string()),
            params: vec![fastn_wasm::Type::ExternRef.to_pl("root")],
            body: vec![
                fastn_wasm::Expression::Call {
                    name: "foo".to_string(),
                    params: vec![
                        fastn_wasm::local_named_get("root"),
                        fastn_wasm::Expression::I32Const(200),
                        fastn_wasm::Expression::I32Const(300),
                    ],
                },
                fastn_wasm::Expression::Call {
                    name: "foo".to_string(),
                    params: vec![
                        fastn_wasm::local_named_get("root"),
                        fastn_wasm::Expression::I32Const(400),
                        fastn_wasm::Expression::I32Const(600),
                    ],
                },
            ],
            ..Default::default()
        }),
        fastn_wasm::Ast::Func(fastn_wasm::Func {
            name: Some("foo".to_string()),
            params: vec![
                fastn_wasm::Type::ExternRef.to_pl("root"),
                fastn_wasm::Type::I32.to_pl("width"),
                fastn_wasm::Type::I32.to_pl("height"),
            ],
            locals: vec![fastn_wasm::Type::ExternRef.to_pl("column")],
            body: vec![
                fastn_wasm::local_named_set(
                    "column",
                    fastn_wasm::Expression::Call {
                        name: "create_column".to_string(),
                        params: vec![],
                    },
                ),
                fastn_wasm::Expression::Call {
                    name: "add_child".to_string(),
                    params: vec![
                        fastn_wasm::local_named_get("root"),
                        fastn_wasm::local_named_get("column"),
                    ],
                },
                fastn_wasm::Expression::Call {
                    name: "set_column_width_px".to_string(),
                    params: vec![
                        fastn_wasm::local_named_get("column"),
                        fastn_wasm::local_named_get("width"),
                    ],
                },
                fastn_wasm::Expression::Call {
                    name: "set_column_height_px".to_string(),
                    params: vec![
                        fastn_wasm::local_named_get("column"),
                        fastn_wasm::local_named_get("height"),
                    ],
                },
            ],
            ..Default::default()
        }),
    ];
    let wat = fastn_wasm::encode(&m);
    println!("{}", wat);
    wat.into_bytes()
}
