#[tokio::main]
async fn main() {
    let document = fastn_surface::Document::default();

    #[cfg(feature = "native")]
    fastn_surface::wgpu::render_document(document).await;

    // #[cfg(feature = "terminal")]
    // fastn_surface::terminal::draw(doc).await;
}
