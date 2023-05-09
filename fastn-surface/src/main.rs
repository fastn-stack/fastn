fn main() {
    let doc = fastn_surface::Document::default();

    #[cfg(feature = "native")]
    {
        fastn_surface::wgpu::render(doc);
    }
}
