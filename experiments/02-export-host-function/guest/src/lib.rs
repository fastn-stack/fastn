extern "C" {
    fn from_host();
}

#[no_mangle]
pub extern "C" fn sum(x: i32) -> i32 {
    x + unsafe {
        from_host();
        10
    }
}
