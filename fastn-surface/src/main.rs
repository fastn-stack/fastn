use fastn_surface::Pencil;

#[tokio::main]
async fn main() {
    let doc = fastn_surface::Document::default();
    doc.render().await;
    // w.layout(window_size.width, window_size.height);

    let d = pencil();
    d.init().await.unwrap();

}

#[cfg(feature = "native")]
fn pencil() -> impl fastn_surface::Pencil {
    fastn_surface::wgpu::Pencil {}
}