#[tokio::main]
async fn main() {
    // check if --wasm is passed on cli
    let wat = if std::env::args().any(|arg| arg == "--stdin") {
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

    let document = fastn_runtime::Document::new(wat);

    #[cfg(feature = "native")]
    fastn_runtime::wgpu::render_document(document).await;

    // #[cfg(feature = "terminal")]
    // fastn_runtime::terminal::draw(doc).await;
}
