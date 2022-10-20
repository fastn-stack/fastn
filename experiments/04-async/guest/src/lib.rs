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

/// lbp = length bytes pointer. length is u32, big endian.
///
/// string's length is length
fn string_to_lbp(s: String) -> u32 {}
