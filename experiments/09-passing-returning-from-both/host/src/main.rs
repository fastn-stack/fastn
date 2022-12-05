/*

Problem statement:
- Call a guest function by passing a string
  - This guest function will then read this string
  - And call a host function again by passing some string
  - Return a string back to guest
  - Return a string back to host
*/

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = wasmtime::Engine::new(wasmtime::Config::new().async_support(true))?;
    let module = wasmtime::Module::from_file(
        &engine,
        "../guest/target/wasm32-unknown-unknown/debug/guest.wasm",
    )?;
    let mut store = wasmtime::Store::new(&engine, ());

    // TODO: Can not use async can only pass the I32 but need to pass the u32
    let _from_host = wasmtime::Func::new_async(
        &mut store,
        wasmtime::FuncType::new(
            vec![wasmtime::ValType::I32, wasmtime::ValType::I32],
            Some(wasmtime::ValType::I32),
        ),
        |_caller, params, results| {
            // TODO:How to pass a pointer of u32, memory address to read the data
            // Not able to access the memory of the wasm
            // println!(
            //     "wasm => pointer: {}, value: {}",
            //     r,
            //     utils::read_string(
            //         r as usize, // upward cast never panics
            //         instance.get_memory(&mut store, "memory").unwrap(),
            //         &mut store
            //     )
            // );

            // Garbage
            Box::new(async move {
                println!("called from wasm: {:?}", params);
                results[0] = wasmtime::Val::I32(
                    tokio::fs::read_to_string("Cargo.toml").await.unwrap().len() as i32,
                );
                Ok(())
            })
        },
    );

    let from_host = wasmtime::Func::wrap(&mut store, |ptr: u32, len: u32| {
        println!("wasm sent: {}, {}", ptr, len);
        let mut buf = Vec::with_capacity(len as usize);
        // No way to access memory
        // wasmtime::Memory::read(&store, ptr as usize, )
        10 as u32
    });

    let instance = wasmtime::Instance::new_async(&mut store, &module, &[from_host.into()]).await?;

    // Calling Guest Function
    call_guest(&mut store, &instance, "Hello From Host").await?;

    // let alloc = instance.get_typed_func::<u32, u32, _>(&mut store, "alloc")?;
    // let size = 10;
    // let memory_address = alloc.call_async(&mut store, size as u32).await?;
    // println!("Wasm Memory address: {}", memory_address);
    // let input = vec![1 as u8, 2, 3, 4, 5, 6, 7, 8 ,9, 10];
    // println!("Coping the data into the wasm memory");
    // memory.write(&mut store, memory_address as usize, input.as_ref()).unwrap();
    // println!("data copied successfully");
    //
    // let array_sum = instance.get_typed_func::<(u32, u32), u32, _>(&mut store, "array_sum")?;
    // let sum_of_array = array_sum.call_async(&mut store, (memory_address, size)).await?;
    // println!("Array Sum: {}", sum_of_array);
    //
    // // println!("Deallocating from the wasm memory");
    // // println!("Coping the data into the wasm memory");
    //
    // // let sum_of_array = array_sum.call_async(&mut store, (memory_address, size)).await?;
    // // println!("Array Sum: {}", sum_of_array);
    //
    // let dealloc = instance.get_typed_func::<(u32, u32), (), _>(&mut store, "dealloc")?;
    // dealloc.call_async(&mut store, (memory_address, size)).await?;
    // println!("Memory deallocated");

    Ok(())
}

// Sending data to guest
// let's say we are sending a string
// write that data to wasm memory using alloc function
// and pass the pointer

async fn call_guest(
    mut store: &mut wasmtime::Store<()>,
    instance: &wasmtime::Instance,
    guest_data: &str,
) -> anyhow::Result<()> {
    let memory: wasmtime::Memory = instance.get_memory(&mut store, "memory").unwrap();

    println!("Allocation memory to Wasm Memory");
    let alloc = instance.get_typed_func::<u32, u32, _>(&mut store, "alloc")?;
    let guest_data_bytes = guest_data.as_bytes();
    let memory_address = alloc
        .call_async(&mut store, guest_data_bytes.len() as u32)
        .await?;

    // Panicking here
    // println!("{}", guest_data_bytes.iter().sum::<u8>());

    println!("Writing Rust Data to WASM memory: {}", guest_data);
    memory.write(&mut store, memory_address as usize, guest_data_bytes)?;
    println!(
        "Calling guest function with memory pointer: {}",
        memory_address
    );

    // Here Guest will return the Wasm memory address
    let call_guest = instance.get_typed_func::<(u32, u32), u32, _>(&mut store, "call_guest")?;
    let guest_memory_offset = call_guest
        .call_async(&mut store, (memory_address, guest_data_bytes.len() as u32))
        .await?;
    println!("Guest Memory Address: {}", guest_memory_offset);

    // let data_pointer = utils::SizedData {
    //     len: guest_data_bytes.len() as u32,
    //     data: memory_address
    // };
    //
    // println!("Should be equal to {}", &data_pointer.len);
    // println!("Should be equal to {}", &data_pointer.data);
    //
    //
    // println!("Convert memory address and data len to pointer array");
    // let data_pointer_bytes = data_pointer.to_bytes();
    //
    // println!("{:?}", &data_pointer_bytes);
    //
    // // This pointer is 8 bytes long and we can pass only 4 bytes pointer to wasm
    // // So we have to also write this pointer to wasm memory
    // let alloc = instance.get_typed_func::<u32, u32, _>(&mut store, "alloc")?;
    // let data_pointer_address = alloc.call_async(&mut store, data_pointer_bytes.len() as u32).await?;
    //
    // // Write Pointer data to memory
    // memory.write(&mut store, data_pointer_address as usize, data_pointer_bytes.as_ref())?;

    // TODO: Read the data from memory address which is returned from wasm
    // let guest_response  = utils::read_string(guest_memory_offset as usize, &memory, &mut store);
    // println!("Guest Response: {}", guest_response);

    // TODO: Need to de-allocate the memory address of wasm
    // Which is allocated twice in the above case

    Ok(())
}

// This utility can be shared at the both end
mod utils {

    pub fn read_string(
        offset: usize,
        mem: &wasmtime::Memory,
        store: &mut wasmtime::Store<()>,
    ) -> String {
        let mem = mem.data(&store);

        // in this function we have used unchecked indexing into mem, so if the offset is out of bounds
        // say because wasm is trying to be funny, we will panic. This is fine for now, but in the future
        // we will want to handle this more gracefully.

        // experiment 06 was to prove that using 4 here is fine
        let size = u32::from_ne_bytes(mem[offset..offset + 4].try_into().unwrap()) as usize;
        let str_offset =
            u32::from_ne_bytes(mem[offset + 4..offset + 8].try_into().unwrap()) as usize;

        std::str::from_utf8(&mem[str_offset..str_offset + size])
            .unwrap_or("oops")
            .to_string()
    }

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
        pub fn to_bytes(self) -> [u8; 8] {
            // pointer array which contains 64 bits
            // in this array we will store the data pointer and len value
            // both are 32 bit
            let mut pointer: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

            // Return the memory representation of this integer as a byte array in native byte order.
            // Note: Wasm follows little endian architecture
            let data_pointer_bytes = self.data.to_le_bytes();
            let len_value_bytes = self.len.to_le_bytes();

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
            // std::mem::forget(pointer);
            // pointer.as_ptr() as u32
            pointer
        }
    }
}
