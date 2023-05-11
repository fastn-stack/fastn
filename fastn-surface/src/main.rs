#[tokio::main]
async fn main() {
    let doc = fastn_surface::Document::default();

    #[cfg(feature = "native")]
    fastn_surface::wgpu::draw(doc).await;

    // #[cfg(feature = "terminal")]
    // fastn_surface::terminal::draw(doc).await;
}
