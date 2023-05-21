pub struct Memory {
    stack: Vec<Frame>,
    booleans: slotmap::SlotMap<fastn_runtime::PointerKey, bool>,
    boolean_vec: slotmap::SlotMap<fastn_runtime::PointerKey, Vec<fastn_runtime::PointerKey>>,
    pointer_deps: slotmap::SecondaryMap<fastn_runtime::PointerKey, Vec<SDep>>,
    dom_pointers: slotmap::SecondaryMap<fastn_runtime::PointerKey, bool>,
}

pub struct SDep {
    stable: fastn_runtime::PointerKey,
    first_link: fastn_runtime::PointerKey,
    source: fastn_runtime::PointerKey,
}


struct S {}

pub struct Frame {
    booleans: Vec<fastn_runtime::PointerKey>,
    boolean_vec: Vec<fastn_runtime::PointerKey>,
}

impl Memory {
    pub fn attach_to_dom(&mut self, _dom: fastn_runtime::PointerKey, _ptr: fastn_runtime::PointerKey) {
        todo!()
    }

    pub fn attach(&mut self, _a: fastn_runtime::PointerKey, _b: fastn_runtime::PointerKey) {
        let _a_deps = match self.s_deps.get(a) {
            None => return,
            Some(v) => v,
        };

        todo!()
    }

    pub fn new() -> Memory {
        Memory {
            booleans: slotmap::SlotMap::with_key(),
            stack: Vec::new(),
            boolean_vec: slotmap::SlotMap::with_key(),
        }
    }

    pub fn register(&self, linker: &mut wasmtime::Linker<fastn_runtime::Dom>) {
        use fastn_runtime::dom::Params;

        linker
            .func_new(
                "runtime_store",
                "create_boolean",
                wasmtime::FuncType::new(
                    [wasmtime::ValType::I32].iter().cloned(),
                    [wasmtime::ValType::ExternRef].iter().cloned(),
                ),
                |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                    // ExternRef is a reference-counted pointer to a host-defined object. We mut not
                    // deallocate it on Rust side unless it's .strong_count() is 0. Not sure how it
                    // affects us yet.

                    let s = &mut caller.data_mut().store;
                    s.booleans.push(params.boolean(0));

                    results[0] = wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                        s.booleans.len() - 1,
                    )));
                    Ok(())
                },
            )
            .unwrap();

        linker
            .func_new(
                "runtime_store",
                "get_boolean",
                wasmtime::FuncType::new(
                    [wasmtime::ValType::ExternRef].iter().cloned(),
                    [wasmtime::ValType::I32].iter().cloned(),
                ),
                |caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                    let s = &caller.data().store;

                    results[0] = wasmtime::Val::I32(
                        s.booleans[params.ptr(0)] as i32,
                    );
                    Ok(())
                },
            )
            .unwrap();
    }
}
