// Functions from host
extern "C" {
    fn from_host(ptr: u32, len: u32) -> u32;
}

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

#[no_mangle]
pub fn call_guest(ptr: u32, len: u32) -> u32 {
    unsafe {
        // read the pointer memory which is of [u8; 8] 8 bytes long array
        let data_bytes = Vec::from_raw_parts(ptr as *mut u8, len as usize, len as usize);
        let mut data = String::from_utf8(data_bytes).unwrap();
        let data_for_host = data + "\nHey This hello From Guest\n How are you doing?";
        let data_len = data_for_host.as_bytes().len();
        let data_pointer_for_host = utils::SizedData::from_string(data_for_host).to_bytes();
        let from_host = from_host(data_pointer_for_host, data_len as u32);
        return from_host;
        // let data_pointer: utils::SizedData = utils::PointerData(memory_pointer).to_sized_data();
        // return data_pointer.len;

        // call the host function by passing the new memory address
        // append new string in the host response and then return the memory address
    }
}

// This utility can be shared at the both end
mod utils {
    pub struct SizedData {
        pub len: u32,
        pub data: u32,
    }

    impl SizedData {
        pub fn from_string(s: String) -> Self {
            let mut data: Vec<u8> = s.into_bytes();
            let len = data.len() as u32;
            let data_ptr = data.as_mut_ptr() as u32;
            // While returning from here rust will not free the memory
            // Otherwise rust will free the memory while returning because of the ownership
            // of the variable
            std::mem::forget(data);
            return SizedData {
                data: data_ptr,
                len,
            };
        }
        pub fn to_bytes(self) -> u32 {
            // pointer array which contains 64 bits
            // in this array we will store the data pointer and len value
            // both are 32 bit
            let mut pointer: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

            // Return the memory representation of this integer as a byte array in native byte order.
            // Note: Wasm follows little endian architecture
            let data_pointer_bytes = self.data.to_ne_bytes();
            let len_value_bytes = self.len.to_ne_bytes();

            // Now we will store both the values in the pointer array and
            // will return that array pointer to rust

            // storing data pointer
            pointer[0] = data_pointer_bytes[0];
            pointer[1] = data_pointer_bytes[1];
            pointer[2] = data_pointer_bytes[2];
            pointer[3] = data_pointer_bytes[3];

            // storing length of the data
            pointer[4] = len_value_bytes[0];
            pointer[5] = len_value_bytes[1];
            pointer[6] = len_value_bytes[2];
            pointer[7] = len_value_bytes[3];

            // To tell rust, forget this memory and
            // do not clean it while returning from the function
            std::mem::forget(pointer);

            pointer.as_ptr() as u32
        }
    }

    pub struct PointerData(pub Vec<u8>);

    impl PointerData {
        pub fn to_sized_data(self) -> SizedData {
            fn u32_le(array: [u8; 4]) -> u32 {
                // First method
                ((array[0] as u32) << 0)
                    + ((array[0] as u32) << 8)
                    + ((array[0] as u32) << 16)
                    + ((array[0] as u32) << 24)

                // Second method
                // unsafe {
                //     std::mem::transmute::<&[u8; 4], u32>(array)
                // }.to_le()

                // Third Method
                // u32::from_le_bytes(array)
            }

            let pointer_bytes = self.0.as_slice();

            // let data_pointer: [u8; 4] = [pointer_bytes[0],pointer_bytes[1],pointer_bytes[2],pointer_bytes[3]];
            // let len_pointer: [u8; 4] = [pointer_bytes[4],pointer_bytes[5],pointer_bytes[6],pointer_bytes[7]];

            SizedData {
                // data: u32::from_le_bytes(pointer_bytes[0..3]);// u32_le(&pointer_bytes[0..4]),
                // len: u32_le(&len_pointer)
                data: 0,
                len: 0,
            }
        }
    }
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
