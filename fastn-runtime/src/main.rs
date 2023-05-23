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

            ;; ... foo definition omitted
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

// (module
//   (func (export "main"))
// )
fn create_module() -> Vec<u8> {
    let m: Vec<fastn_wasm::Ast> = vec![
        fastn_wasm::Ast::Import(fastn_wasm::Import {
            module: "fastn".to_string(),
            name: "create_column".to_string(),
            desc: fastn_wasm::ImportDesc::Func(fastn_wasm::FuncDecl {
                    name: Some("create_column".to_string()),
                    params: vec![],
                    result: Some(fastn_wasm::Type::ExternRef),
                },
            ),
        }),
        fastn_wasm::Ast::Func(fastn_wasm::Func {
        name: None,
        export: Some("main".to_string()),
        params: vec![fastn_wasm::Type::ExternRef.into()],
        locals: vec![],
        result: None,
        body: vec![
            // fastn_wasm::Instr::LocalGet(0),
        ],
    })];
    let wat = fastn_wasm::encode(&m);
    println!("{}", wat);
    wat.into_bytes()
}
