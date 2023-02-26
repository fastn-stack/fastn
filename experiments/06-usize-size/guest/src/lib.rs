#[no_mangle]
pub extern "C" fn usize_size() -> i32 {
    1usize.to_ne_bytes().len() as i32
}
