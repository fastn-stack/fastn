fn main() {
    let window = fastn_surface::Window::new();

    #[cfg(feature = "native")]
    if true {
        fastn_surface::native::render(window);
        return;
    }

}