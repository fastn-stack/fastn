#[tokio::main]
async fn main() {
    let document = fastn_runtime::Document::new(
        r#"
        (module
            (import "" ""
                (func $create_column
                    (param $t i32)
                    (param $l i32)
                    (param $w i32)
                    (param $h i32)
                    (param $r i32)
                    (param $g i32)
                    (param $b i32)
                    (result i32)
                )
            )

            (func (export "main") (result i32)
                (call $create_column
                    (i32.const 100)
                    (i32.const 100)
                    (i32.const 200)
                    (i32.const 300)
                    (i32.const 200)
                    (i32.const 0)
                    (i32.const 0)
                )
                drop
                (call $create_column
                    (i32.const 500)
                    (i32.const 100)
                    (i32.const 200)
                    (i32.const 300)
                    (i32.const 00)
                    (i32.const 200)
                    (i32.const 0)
                )

            )
        )
    "#,
    );

    #[cfg(feature = "native")]
    fastn_runtime::wgpu::render_document(document).await;

    // #[cfg(feature = "terminal")]
    // fastn_runtime::terminal::draw(doc).await;
}
