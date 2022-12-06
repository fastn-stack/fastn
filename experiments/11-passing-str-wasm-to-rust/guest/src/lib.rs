// Functions from host
extern "C" {
    fn from_host(ptr: u32, len: u32) -> u32;
}

#[no_mangle]
pub fn call_guest(_ptr: u32) -> u32 {
    unsafe {
        let data = "Call From Guest".to_string();
        let buf = data.into_bytes();
        let buf_pointer = buf.as_ptr() as u32;
        let buf_len = buf.len();
        // std::mem::forget(buf);
        let from_host = from_host(buf_pointer, buf_len as u32);
        return from_host;
    }
}
