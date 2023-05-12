#[tokio::main]
async fn main() {
    let document = fastn_runtime::Document::default();

    #[cfg(feature = "native")]
    fastn_runtime::wgpu::render_document(document).await;

    // #[cfg(feature = "terminal")]
    // fastn_runtime::terminal::draw(doc).await;
}
