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
        fastn_wasm::import_func0("create_column", fastn_wasm::Type::ExternRef),
        fastn_wasm::import_func2(
            "add_child",
            fastn_wasm::Type::ExternRef.into(),
            fastn_wasm::Type::ExternRef.into(),
        ),
        fastn_wasm::import_func2(
            "set_column_width_px",
            fastn_wasm::Type::ExternRef.into(),
            fastn_wasm::Type::I32.into(),
        ),
        fastn_wasm::import_func2(
            "set_column_height_px",
            fastn_wasm::Type::ExternRef.into(),
            fastn_wasm::Type::I32.into(),
        ),
        fastn_wasm::exported_func1(
            "main",
            fastn_wasm::Type::ExternRef.to_pl("root"),
            vec![
                fastn_wasm::call3(
                    "foo",
                    fastn_wasm::local("root"),
                    fastn_wasm::i32(100),
                    fastn_wasm::i32(200),
                ),
                fastn_wasm::call3(
                    "foo",
                    fastn_wasm::local("root"),
                    fastn_wasm::i32(400),
                    fastn_wasm::i32(600),
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
                fastn_wasm::local_set(
                    "column",
                    fastn_wasm::Expression::Call {
                        name: "create_column".to_string(),
                        params: vec![],
                    },
                ),
                fastn_wasm::Expression::Call {
                    name: "add_child".to_string(),
                    params: vec![fastn_wasm::local("root"), fastn_wasm::local("column")],
                },
                fastn_wasm::Expression::Call {
                    name: "set_column_width_px".to_string(),
                    params: vec![fastn_wasm::local("column"), fastn_wasm::local("width")],
                },
                fastn_wasm::Expression::Call {
                    name: "set_column_height_px".to_string(),
                    params: vec![fastn_wasm::local("column"), fastn_wasm::local("height")],
                },
            ],
            ..Default::default()
        }),
    ];
    let wat = fastn_wasm::encode(&m);
    println!("{}", wat);
    wat.into_bytes()
}

// source: fastn-runtime/columns.ftd
// fn create_columns() -> Vec<u8> {
//     todo!()
// }
