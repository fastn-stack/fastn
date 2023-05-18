#[tokio::main]
async fn main() {
    let document = fastn_runtime::Document::new(
        r#"
        (module
            (import "" ""
                (func $create_column
                    (result externref)
                )
            )

            (func (export "main")
                ;; -- ftd.column:
                (call $create_column)
                (drop)

                ;; -- ftd.column:
                (call $create_column)
                (drop)
            )
        )
    "#,
    );

    #[cfg(feature = "native")]
    fastn_runtime::wgpu::render_document(document).await;

    // #[cfg(feature = "terminal")]
    // fastn_runtime::terminal::draw(doc).await;
}
