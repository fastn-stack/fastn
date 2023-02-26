/*
Problem statement:
- Call the guest function call_guest
- wasm::call_guest function will call then rust::call_host by passing a string pointer
- rust::call_host function will read this string and print it simply
*/

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

impl State {
    fn to_pointer(self) -> Pointer {
        match self {
            State::AllocateMemory(len) => Pointer {
                addr: 0,
                len,
                state: 1,
            },
            State::MemoryAllocated(addr, len) => Pointer {
                addr,
                len,
                state: 2,
            },
            State::Done(addr, len) => Pointer {
                addr,
                len,
                state: 3,
            },
        }
    }
}

impl Pointer {
    fn to_bytes(self) -> [u8; 12] {
        let mut buffer = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let addr_bytes = self.addr.to_ne_bytes();
        let len_bytes = self.len.to_ne_bytes();
        let state_bytes = self.state.to_ne_bytes();
        let mut buffer_pointer = 0;

        // Putting address bytes
        for byte in addr_bytes {
            buffer[buffer_pointer] = byte;
            buffer_pointer += 1;
        }

        // Putting len bytes
        for byte in len_bytes {
            buffer[buffer_pointer] = byte;
            buffer_pointer += 1;
        }

        // Putting len bytes
        for byte in state_bytes {
            buffer[buffer_pointer] = byte;
            buffer_pointer += 1;
        }

        return buffer;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::from_file(
        &engine,
        "../guest/target/wasm32-unknown-unknown/debug/guest.wasm",
    )?;
    let mut store = wasmtime::Store::new(&engine, ());

    // How to access the memory in host function
    let from_host = wasmtime::Func::wrap(
        &mut store,
        |mut caller: wasmtime::Caller<'_, ()>, ptr: u32| {
            // ptr: u32, this always giving the memory of 12 bytes
            let mem = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => anyhow::bail!("failed to find host memory"),
            };

            // TODO: In second it will return the pointer

            // In first it will return for asking the memory
            let host_data = "hello from host".to_string().into_bytes();

            // Allocate Memory to write the data
            let state = State::AllocateMemory(host_data.len() as u32);
            let state_pointer = state.to_pointer().to_bytes();

            // Writing state pointer to the WASM Memory
            match mem.write(&mut caller, ptr as usize, state_pointer.as_slice()) {
                Ok(_) => {
                    println!("Asked to allocate the memory");
                }
                Err(err) => {
                    println!("Memory Access Error: {}", err)
                }
            };

            Ok(ptr)
            // let data = mem
            //     .data(&caller)
            //     .get(ptr as u32 as usize..)
            //     .and_then(|arr| arr.get(..len as u32 as usize));
            // let string = match data {
            //     Some(data) => match std::str::from_utf8(data) {
            //         Ok(s) => s,
            //         Err(_) => anyhow::bail!("invalid utf-8"),
            //     },
            //     None => anyhow::bail!("pointer/length out of bounds"),
            // };
            // // assert_eq!(string, "Hello, world!");
            // println!("Guest Passed: {}", string);
        },
    );

    let instance = wasmtime::Instance::new(&mut store, &module, &[from_host.into()])?;
    let call_guest = instance.get_typed_func::<(u32,), u32, _>(&mut store, "call_guest")?;
    // And finally we can call the wasm!
    println!("wasm said: {}", call_guest.call(&mut store, (1,))?);

    Ok(())
}
