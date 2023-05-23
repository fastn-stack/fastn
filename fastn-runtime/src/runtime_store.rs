#[derive(Debug, Default)]
pub struct Memory {
    stack: Vec<Frame>,

    boolean: slotmap::SlotMap<fastn_runtime::PointerKey, bool>,
    i32: slotmap::SlotMap<fastn_runtime::PointerKey, i32>,
    f32: slotmap::SlotMap<fastn_runtime::PointerKey, f32>,
    vec: slotmap::SlotMap<fastn_runtime::PointerKey, (i32, Vec<fastn_runtime::PointerKey>)>,

    pointer_deps: std::collections::HashMap<Key, Vec<SDep>>,
    ui_deps: std::collections::HashMap<Key, Vec<fastn_runtime::PointerKey>>,
}

#[derive(Debug)]
pub struct SDep {
    // this is the dom element we are directly or indirectly connected with
    element: fastn_runtime::PointerKey,
    // who gave us this link
    source: Key,
}

#[derive(Debug)]
struct Key {
    key: fastn_runtime::PointerKey,
    kind: Kind,
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
    pointers: Vec<Key>,
}

impl Memory {
    pub fn _attach_to_dom(
        &mut self,
        _dom: fastn_runtime::PointerKey,
        _ptr: fastn_runtime::PointerKey,
    ) {
        todo!()
    }

    pub fn _attach(&mut self, _a: fastn_runtime::PointerKey, _b: fastn_runtime::PointerKey) {
        // let _a_deps = match self.pointer_deps.get(&a) {
        //     None => return,
        //     Some(v) => v,
        // };

        todo!()
    }

    pub fn register(&self, linker: &mut wasmtime::Linker<fastn_runtime::Dom>) {
        use fastn_runtime::dom::Params;

        linker
            .func_new(
                "fastn",
                "create_boolean",
                wasmtime::FuncType::new(
                    [wasmtime::ValType::I32].iter().cloned(),
                    [wasmtime::ValType::ExternRef].iter().cloned(),
                ),
                |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                    // ExternRef is a reference-counted pointer to a host-defined object. We mut not
                    // deallocate it on Rust side unless it's .strong_count() is 0. Not sure how it
                    // affects us yet.

                    results[0] = wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                        caller.data_mut().store.boolean.insert(params.boolean(0)),
                    )));
                    Ok(())
                },
            )
            .unwrap();

        linker
            .func_new(
                "fastn",
                "get_boolean",
                wasmtime::FuncType::new(
                    [wasmtime::ValType::ExternRef].iter().cloned(),
                    [wasmtime::ValType::I32].iter().cloned(),
                ),
                |caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                    let s = &caller.data().store;

                    results[0] = wasmtime::Val::I32(s.boolean[params.ptr(0)] as i32);
                    Ok(())
                },
            )
            .unwrap();
    }
}
