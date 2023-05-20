pub struct Store {
    booleans: Vec<bool>,

}

impl Store {
    pub fn new() -> Store {
        Store {
            booleans: Vec::new(),
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
