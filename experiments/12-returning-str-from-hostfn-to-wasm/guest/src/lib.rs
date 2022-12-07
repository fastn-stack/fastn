// Functions from host
extern "C" {
    // pass the 12 bytes allocated memory address as argument
    // and it will return the same address back
    fn from_host(ptr: u32) -> u32;
}

#[no_mangle]
pub fn call_guest(_ptr: u32) -> u32 {
    unsafe {
        // Allocate wasm memory for the pointer which is returned from the host
        let pointer_ptr = alloc(12);
        let pointer_ptr = from_host(pointer_ptr);
        let pointer_data = Pointer::to_pointer(pointer_ptr);
        // dealloc(pointer_ptr, 12);
        return pointer_data.len;
    }
}

pub enum State {
    AllocateMemory(u32),       // in that case u32 is len
    MemoryAllocated(u32, u32), // data pointer and len of the memory
    Done(u32, u32),            // data pointer and len of the memory
}

pub struct Pointer {
    addr: u32,
    len: u32,
    state: u32,
}

impl Pointer {
    fn to_pointer(ptr: u32) -> Pointer {
        // Always assuming to read the 12 bytes of the memory
        unsafe {
            let bytes_from_pointer = Vec::from_raw_parts(ptr as *mut u8, 12 as usize, 12 as usize);
            Pointer {
                addr: u32::from_ne_bytes(bytes_from_pointer[0..4].try_into().unwrap()),
                len: u32::from_ne_bytes(bytes_from_pointer[4..8].try_into().unwrap()),
                state: u32::from_ne_bytes(bytes_from_pointer[8..12].try_into().unwrap()),
            }
        }
    }
}

// This function takes the len as input argument
// and create the buffer of provided length
// and return the pointer of that buffer
pub fn alloc(len: u32) -> u32 {
    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize);
    // set the length of the buffer
    unsafe {
        buffer.set_len(len as usize);
    }

    let buffer_ptr = buffer.as_mut_ptr();
    // To tell the rust runtime, not to flush this memory while function is complete
    std::mem::forget(buffer);
    // returning the buffer pointer
    return buffer_ptr as u32;
}

pub fn dealloc(ptr: u32, size: u32) {
    unsafe {
        let data = Vec::from_raw_parts(ptr as *mut u8, size as usize, size as usize);
        drop(data);
    }
}
