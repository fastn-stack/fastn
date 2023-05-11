#[tokio::main]
async fn main() {
    let doc = fastn_surface::Document::default();

    #[cfg(feature = "native")]
    {
        fastn_surface::wgpu::render(doc).await
    }
}


