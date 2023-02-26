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
pub fn guest_only_ptr(ptr: u32) -> u32 {
    unsafe {
        // std::ptr::read()
        let memory_pointer = Vec::from_raw_parts(ptr as *mut u8, 8 as usize, 8 as usize);

        let data_pointer: utils::SizedData = utils::SizedData::from_pointer(memory_pointer);
        // This is returning wrong length of the data
        // data_pointer.iter().sum::<u8>() as u32;
        return data_pointer.len;
    }
}

// This utility can be shared at the both end
mod utils {

    pub struct SizedData {
        pub len: u32,
        pub data: u32,
    }

    impl SizedData {
        pub fn from_pointer(array: Vec<u8>) -> SizedData {
            let pointer_bytes = array.as_slice();
            SizedData {
                data: u32::from_ne_bytes(pointer_bytes[0..4].try_into().unwrap()),
                len: u32::from_ne_bytes(pointer_bytes[4..8].try_into().unwrap()),
            }
        }
    }

    // let size = u32::from_ne_bytes(mem[offset..offset + 4].try_into().unwrap()) as usize;
    // let str_offset = u32::from_ne_bytes(mem[offset + 4..offset + 8].try_into().unwrap()) as usize;
    //
    // std::str::from_utf8(&mem[str_offset..str_offset + size])
    //     .unwrap_or("oops")
    //     .to_string()
}

/*


############
Requirements
############

- Need a function append_something(string_ptr)
  From Host: Hi Guest
  From Guest: Hi Host, How are you doing?
  From Host: I am doing good, what about you?
  From Guest: I am also doing great, I would like
  tell you stop using my memory.
  call host_append_string(string)
 */
