fn main() {
    let window = fastn_surface::Window::default();

    #[cfg(feature = "native")]
    {
        fastn_surface::native::render(window);
    }
}
