fn main() {
    #[cfg(feature = "native")]
    if true {
        fastn_surface::native::run();
        return;
    }

}