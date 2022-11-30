/// Allocate memory into the wasm linear memory
/// and return the offset to the start of the block.
#[no_mangle]
pub fn alloc(len: u32) -> u32 {
    // create a new mutable buffer with capacity len
    let mut buf: Vec<u8> = Vec::with_capacity(len as usize);
    unsafe {
        buf.set_len(len as usize);
    }
    // take mutable pointer to the buffer
    let ptr = buf.as_mut_ptr();
    // take ownership of the memory block and
    // ensure that its destructor is not
    // called when the object goes out of scope
    // at the end of the function
    std::mem::forget(buf);
    return ptr as u32;
}

/// de-allocating the memory
#[no_mangle]
pub unsafe fn dealloc(ptr: u32, size: u32) {
    let data = Vec::from_raw_parts(ptr as *mut u8, size as usize, size as usize);
    drop(data);
}

#[no_mangle]
pub fn array_sum(ptr: u32, len: u32) -> u32 {
    unsafe {
        let data = Vec::from_raw_parts(ptr as *mut u8, len as usize, len as usize);
        let sum: u8 = data.iter().sum();
        // std::mem::forget(data);
        return sum as u32;
    }
}
