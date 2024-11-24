#![deny(unused_crate_dependencies)]

extern crate self as fastn_runtime;

#[cfg(feature = "browser")]
fn main() {}

#[cfg(not(feature = "browser"))]
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

    // let document = fastn_runtime::Document::new(create_columns());

    #[cfg(feature = "native")]
    fastn_runtime::wgpu::render_document(document).await;

    // #[cfg(feature = "terminal")]
    // fastn_runtime::terminal::draw(doc).await;
}

#[cfg(not(feature = "browser"))]
pub fn create_module() -> Vec<u8> {
    let m: Vec<fastn_wasm::Ast> = vec![
        fastn_wasm::import::func0("create_column", fastn_wasm::Type::ExternRef),
        fastn_wasm::import::func2(
            "add_child",
            fastn_wasm::Type::ExternRef.into(),
            fastn_wasm::Type::ExternRef.into(),
        ),
        fastn_wasm::import::func2(
            "set_column_width_px",
            fastn_wasm::Type::ExternRef.into(),
            fastn_wasm::Type::I32.into(),
        ),
        fastn_wasm::import::func2(
            "set_column_height_px",
            fastn_wasm::Type::ExternRef.into(),
            fastn_wasm::Type::I32.into(),
        ),
        fastn_wasm::export::func1(
            "main",
            fastn_wasm::Type::ExternRef.to_pl("root"),
            vec![
                fastn_wasm::expression::call3(
                    "foo",
                    fastn_wasm::expression::local("root"),
                    fastn_wasm::expression::i32(100),
                    fastn_wasm::expression::i32(200),
                ),
                fastn_wasm::expression::call3(
                    "foo",
                    fastn_wasm::expression::local("root"),
                    fastn_wasm::expression::i32(400),
                    fastn_wasm::expression::i32(600),
                ),
            ],
        ),
        fastn_wasm::Ast::Func(fastn_wasm::Func {
            name: Some("foo".to_string()),
            params: vec![
                fastn_wasm::Type::ExternRef.to_pl("root"),
                fastn_wasm::Type::I32.to_pl("width"),
                fastn_wasm::Type::I32.to_pl("height"),
            ],
            locals: vec![fastn_wasm::Type::ExternRef.to_pl("column")],
            body: vec![
                fastn_wasm::expression::local_set(
                    "column",
                    fastn_wasm::expression::call("create_column"),
                ),
                fastn_wasm::Expression::Call {
                    name: "add_child".to_string(),
                    params: vec![
                        fastn_wasm::expression::local("root"),
                        fastn_wasm::expression::local("column"),
                    ],
                },
                fastn_wasm::Expression::Call {
                    name: "set_column_width_px".to_string(),
                    params: vec![
                        fastn_wasm::expression::local("column"),
                        fastn_wasm::expression::local("width"),
                    ],
                },
                fastn_wasm::Expression::Call {
                    name: "set_column_height_px".to_string(),
                    params: vec![
                        fastn_wasm::expression::local("column"),
                        fastn_wasm::expression::local("height"),
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

// source: columns.clj (derived from columns.ftd)
#[cfg(not(feature = "browser"))]
fn create_columns() -> Vec<u8> {
    let mut m: Vec<fastn_wasm::Ast> = fastn_runtime::Dom::imports();

    // Note: can not add these till the functions are defined
    m.extend(fastn_wasm::table_2(
        fastn_wasm::RefType::Func,
        "product",
        "foo#on_mouse_enter",
        // "foo#on_mouse_leave",
        // "foo#background",
    ));

    m.push(fastn_wasm::func_def::func1ret(
        "return_externref",
        fastn_wasm::Type::ExternRef.into(),
        fastn_wasm::Type::ExternRef,
    ));

    m.push(fastn_wasm::func_def::func1(
        "no_return",
        fastn_wasm::Type::ExternRef.into(),
    ));

    // (func (export "call_by_index") (param $idx i32) (param $arr externref) (result externref)
    //    call_indirect (type $return_externref) (local.get 0) (local.get 1)
    // )

    // (type $return_externref (func (param externref) (result externref)))
    // (func (export "call_by_index")
    //      (param $idx i32)
    //      (param $arr externref)
    //      (result externref)
    //
    //      (call_indirect (type $return_externref) (local.get $idx) (local.get $arr))
    // )

    m.push(
        fastn_wasm::Func {
            name: None,
            export: Some("call_by_index".to_string()),
            params: vec![
                fastn_wasm::Type::I32.to_pl("fn_idx"),
                fastn_wasm::Type::ExternRef.to_pl("arr"),
            ],
            locals: vec![],
            result: Some(fastn_wasm::Type::ExternRef),
            body: vec![fastn_wasm::expression::call_indirect2(
                "return_externref",
                fastn_wasm::expression::local("arr"),
                fastn_wasm::expression::local("fn_idx"),
            )],
        }
        .to_ast(),
    );

    m.push(
        fastn_wasm::Func {
            name: None,
            export: Some("void_by_index".to_string()),
            params: vec![
                fastn_wasm::Type::I32.to_pl("fn_idx"),
                fastn_wasm::Type::ExternRef.to_pl("arr"),
            ],
            locals: vec![],
            result: None,
            body: vec![fastn_wasm::expression::call_indirect2(
                "no_return",
                fastn_wasm::expression::local("arr"),
                fastn_wasm::expression::local("fn_idx"),
            )],
        }
        .to_ast(),
    );

    m.push(
        fastn_wasm::Func {
            name: Some("product".to_string()),
            export: None,
            params: vec![fastn_wasm::Type::ExternRef.to_pl("func-data")],
            locals: vec![],
            result: Some(fastn_wasm::Type::ExternRef),
            body: vec![
                fastn_wasm::expression::call("create_frame"),
                fastn_wasm::expression::call1(
                    "return_frame",
                    fastn_wasm::expression::call3(
                        "multiply_i32",
                        fastn_wasm::expression::local("func-data"),
                        fastn_wasm::expression::i32(0),
                        fastn_wasm::expression::i32(1),
                    ),
                ),
            ],
        }
        .to_ast(),
    );

    m.push(
        fastn_wasm::Func {
            name: None,
            export: Some("main".to_string()),
            params: vec![fastn_wasm::Type::ExternRef.to_pl("root")],
            locals: vec![fastn_wasm::Type::ExternRef.to_pl("column")],
            result: None,
            body: vec![
                fastn_wasm::expression::call("create_frame"),
                fastn_wasm::expression::call2(
                    "set_global",
                    fastn_wasm::expression::i32(0),
                    fastn_wasm::expression::call1("create_boolean", fastn_wasm::expression::i32(0)),
                ),
                fastn_wasm::expression::call2(
                    "set_global",
                    fastn_wasm::expression::i32(1),
                    fastn_wasm::expression::call1("create_i32", fastn_wasm::expression::i32(42)),
                ),
                fastn_wasm::expression::local_set(
                    "column",
                    fastn_wasm::expression::call2(
                        "create_kernel",
                        fastn_wasm::expression::local("root"),
                        fastn_wasm::expression::i32(fastn_runtime::ElementKind::Column.into()),
                    ),
                ),
                /* fastn_wasm::expression::call4(
                    "set_dynamic_property_i32",
                    fastn_wasm::expression::local("column"),
                    fastn_wasm::expression::i32(fastn_runtime::UIProperty::WidthFixedPx.into()),
                    fastn_wasm::expression::i32(0), // table_index
                    fastn_wasm::expression::call2(
                        "array_i32_2",
                        fastn_wasm::expression::call1(
                            "create_i32",
                            fastn_wasm::expression::i32(10),
                        ),
                        fastn_wasm::expression::call1("get_global", fastn_wasm::expression::i32(1)),
                    ),
                ),*/
                fastn_wasm::expression::call3(
                    "set_property_i32",
                    fastn_wasm::expression::local("column"),
                    fastn_wasm::expression::i32(fastn_runtime::UIProperty::HeightFixedPx.into()),
                    fastn_wasm::expression::i32(500),
                ),
                fastn_wasm::expression::call3(
                    "set_property_i32",
                    fastn_wasm::expression::local("column"),
                    fastn_wasm::expression::i32(fastn_runtime::UIProperty::SpacingFixedPx.into()),
                    fastn_wasm::expression::i32(100),
                ),
                fastn_wasm::expression::call3(
                    "set_property_i32",
                    fastn_wasm::expression::local("column"),
                    fastn_wasm::expression::i32(fastn_runtime::UIProperty::MarginFixedPx.into()),
                    fastn_wasm::expression::i32(140),
                ),
                fastn_wasm::expression::call1("foo", fastn_wasm::expression::local("column")),
                fastn_wasm::expression::call1("foo", fastn_wasm::expression::local("column")),
                fastn_wasm::expression::call("end_frame"),
            ],
        }
        .to_ast(),
    );

    m.push(
        fastn_wasm::Func {
            name: Some("foo".to_string()),
            export: None,
            params: vec![fastn_wasm::Type::ExternRef.to_pl("parent")],
            locals: vec![
                fastn_wasm::Type::ExternRef.to_pl("column"),
                fastn_wasm::Type::ExternRef.to_pl("on-hover"),
            ],
            result: None,
            body: vec![
                fastn_wasm::expression::call("create_frame"),
                fastn_wasm::expression::local_set(
                    "on-hover",
                    fastn_wasm::expression::call1("create_i32", fastn_wasm::expression::i32(42)),
                ),
                fastn_wasm::expression::local_set(
                    "column",
                    fastn_wasm::expression::call2(
                        "create_kernel",
                        fastn_wasm::expression::local("parent"),
                        fastn_wasm::expression::i32(fastn_runtime::ElementKind::Column.into()),
                    ),
                ),
                fastn_wasm::expression::call4(
                    "attach_event_handler",
                    fastn_wasm::expression::local("column"),
                    fastn_wasm::expression::i32(fastn_runtime::DomEventKind::OnGlobalKey.into()),
                    fastn_wasm::expression::i32(1), // table index (on-mouse-enter)
                    fastn_wasm::expression::call4(
                        "create_list_2",
                        fastn_wasm::expression::i32(fastn_runtime::PointerKind::Integer.into()),
                        fastn_wasm::expression::call1("get_global", fastn_wasm::expression::i32(1)),
                        fastn_wasm::expression::i32(fastn_runtime::PointerKind::Integer.into()),
                        fastn_wasm::expression::local("on-hover"),
                    ),
                ),
                fastn_wasm::expression::call3(
                    "set_property_i32",
                    fastn_wasm::expression::local("column"),
                    fastn_wasm::expression::i32(fastn_runtime::UIProperty::HeightFixedPx.into()),
                    fastn_wasm::expression::i32(80),
                ),
                fastn_wasm::expression::call4(
                    "set_dynamic_property_i32",
                    fastn_wasm::expression::local("column"),
                    fastn_wasm::expression::i32(fastn_runtime::UIProperty::WidthFixedPx.into()),
                    fastn_wasm::expression::i32(0), // table_index
                    fastn_wasm::expression::call2(
                        "array_i32_2",
                        fastn_wasm::expression::call1("create_i32", fastn_wasm::expression::i32(2)),
                        fastn_wasm::expression::local("on-hover"),
                    ),
                ),
                fastn_wasm::expression::call("end_frame"),
            ],
        }
        .to_ast(),
    );

    m.push(
        fastn_wasm::Func {
            name: Some("foo#on_mouse_enter".to_string()),
            export: None,
            params: vec![fastn_wasm::Type::ExternRef.to_pl("func-data")],
            locals: vec![],
            result: None,
            body: vec![
                fastn_wasm::expression::call("create_frame"),
                fastn_wasm::expression::call2(
                    "set_i32",
                    fastn_wasm::expression::call2(
                        "get_func_arg_ref",
                        fastn_wasm::expression::local("func-data"),
                        fastn_wasm::expression::i32(1),
                    ),
                    fastn_wasm::expression::i32(80),
                ),
                fastn_wasm::expression::call("end_frame"),
            ],
        }
        .to_ast(),
    );

    m.push(
        fastn_wasm::Func {
            name: Some("foo#on_mouse_leave".to_string()),
            export: None,
            params: vec![fastn_wasm::Type::ExternRef.to_pl("func-data")],
            locals: vec![],
            result: None,
            body: vec![
                fastn_wasm::expression::call("create_frame"),
                // fastn_wasm::expression::call2(
                //     "set_boolean",
                // ),
                fastn_wasm::expression::call("end_frame"),
            ],
        }
        .to_ast(),
    );

    let wat = fastn_wasm::encode(&m);
    println!("{}", wat);
    wat.into_bytes()
}
