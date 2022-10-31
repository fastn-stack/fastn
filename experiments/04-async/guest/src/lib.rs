extern "C" {
    fn from_host(a: i32, b: i32) -> i32;
}

#[no_mangle]
pub extern "C" fn sum(x: i32) -> i32 {
    x + unsafe {
        from_host(10, 20);
        10
    }
}
