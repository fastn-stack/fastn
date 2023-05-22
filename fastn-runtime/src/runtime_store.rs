#[derive(Debug, Default)]
pub struct Memory {
    stack: Vec<Frame>,
    r_2: slotmap::SlotMap<fastn_runtime::PointerKey, [fastn_runtime::PointerKey; 2]>,
    o_2: slotmap::SlotMap<fastn_runtime::PointerKey, (u8, [fastn_runtime::PointerKey; 2])>,
    a_3: slotmap::SlotMap<fastn_runtime::PointerKey, [fastn_runtime::PointerKey; 3]>,
    a_20: slotmap::SlotMap<fastn_runtime::PointerKey, [fastn_runtime::PointerKey; 20]>,
    booleans: slotmap::SlotMap<fastn_runtime::PointerKey, bool>,
    boolean_vec: slotmap::SlotMap<fastn_runtime::PointerKey, Vec<fastn_runtime::PointerKey>>,
    pointer_deps: std::collections::HashMap<fastn_runtime::PointerKey, Vec<SDep>>,
}

#[derive(Debug, Default)]
pub struct SDep {
    // this is the dom element we are directly or indirectly connected with
    element: fastn_runtime::PointerKey,
    // who gave us this link
    source: fastn_runtime::PointerKey,
}

#[derive(Debug)]
enum Kind {
    Boolean,
    BooleanVec,
    Integer,
    Record2,
    Record3,
    OrType2,
}

#[derive(Debug, Default)]
pub struct Frame {
    pointers: Vec<(Kind, fastn_runtime::PointerKey)>,
}

impl Memory {
    pub fn _attach_to_dom(
        &mut self,
        _dom: fastn_runtime::PointerKey,
        _ptr: fastn_runtime::PointerKey,
    ) {
        todo!()
    }

    pub fn _attach(&mut self, a: fastn_runtime::PointerKey, _b: fastn_runtime::PointerKey) {
        let _a_deps = match self.pointer_deps.get(&a) {
            None => return,
            Some(v) => v,
        };

        todo!()
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
                    s.booleans.insert(params.boolean(0));

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

                    results[0] = wasmtime::Val::I32(s.booleans[params.ptr(0)] as i32);
                    Ok(())
                },
            )
            .unwrap();
    }
}
