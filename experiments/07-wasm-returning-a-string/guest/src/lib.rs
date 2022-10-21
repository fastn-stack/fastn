#[no_mangle]
pub extern "C" fn hello() -> u32 {
    SizedData::from_string("wasm says hello".to_string()).to_bytes()
}

#[no_mangle]
pub extern "C" fn free(ptr: u32, size: u32) {
    let v = unsafe { Vec::from_raw_parts(ptr as *mut u8, size as usize, size as usize) };
    drop(v);
}

struct SizedData {
    len: u32,  // same as usize in wasm
    data: u32, // same as *const u8 in wasm
}

impl SizedData {
    fn from_string(s: String) -> Self {
        let mut d = s.into_bytes();
        let len = d.len() as u32;
        let data = d.as_mut_ptr() as u32;

        std::mem::forget(d);

        SizedData { data, len }
    }

    fn to_bytes(self) -> u32 {
        let mut o: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        let b = self.len.to_ne_bytes();
        o[0] = b[0];
        o[1] = b[1];
        o[2] = b[2];
        o[3] = b[3];

        let b = self.data.to_ne_bytes();
        o[4] = b[0];
        o[5] = b[1];
        o[6] = b[2];
        o[7] = b[3];

        std::mem::forget(o);

        o.as_ptr() as u32
    }
}
