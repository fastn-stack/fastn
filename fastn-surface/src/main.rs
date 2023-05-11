#[tokio::main]
async fn main() {
    let doc = fastn_surface::Document::default();
    let ops = doc.render().await;

    #[cfg(feature = "native")]
    fastn_surface::wgpu::draw(&ops).await;
}
