#[tokio::main]
async fn main() {
    let document = fastn_runtime::Document::new(
        r#"
        (module
            (import "fastn" "create_column" (func $create_column (result externref)))
            (import "fastn" "root_container" (func $root_container (result externref)))
            (import "fastn" "set_column_width_px" (func $set_column_width_px (param externref i32)))
            (import "fastn" "set_column_height_px" (func $set_column_height_px (param externref i32)))

            ;; fastn.add_child(parent: NodeKey, child: NodeKey)
            (import "fastn" "add_child" (func $add_child (param externref externref)))

            (func (export "main") (local $column externref)
                ;; -- ftd.column:
                (local.set $column (call $create_column))
                (call $add_child (call $root_container) (local.get $column))
                (call $set_column_width_px (local.get $column) (i32.const 600))
                (call $set_column_height_px (local.get $column) (i32.const 400))

                ;; -- ftd.column:
                (local.set $column (call $create_column))
                (call $add_child (call $root_container) (local.get $column))
                (call $set_column_width_px (local.get $column) (i32.const 300))
                (call $set_column_height_px (local.get $column) (i32.const 700))
            )
        )
    "#,
    );

    #[cfg(feature = "native")]
    fastn_runtime::wgpu::render_document(document).await;

    // #[cfg(feature = "terminal")]
    // fastn_runtime::terminal::draw(doc).await;
}
