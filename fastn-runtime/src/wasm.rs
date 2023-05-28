impl fastn_runtime::Dom {
    pub fn create_instance(
        wat: impl AsRef<[u8]>,
    ) -> (wasmtime::Store<fastn_runtime::Dom>, wasmtime::Instance) {
        let engine = wasmtime::Engine::new(wasmtime::Config::new().async_support(false))
            .expect("cant create engine");
        let module = wasmtime::Module::new(&engine, wat).expect("cant parse module");
        let dom = fastn_runtime::Dom::default();

        let mut linker = wasmtime::Linker::new(&engine);

        dom.register_functions(&mut linker);

        let mut store = wasmtime::Store::new(&engine, fastn_runtime::Dom::default());
        let instance = linker
            .instantiate(&mut store, &module)
            .expect("cant create instance");

        let root = Some(wasmtime::ExternRef::new(store.data().root()));

        let wasm_main = instance
            .get_typed_func::<(Option<wasmtime::ExternRef>,), ()>(&mut store, "main")
            .unwrap();
        wasm_main.call(&mut store, (root,)).unwrap();

        (store, instance)
    }

    fn register_functions(&self, linker: &mut wasmtime::Linker<fastn_runtime::Dom>) {
        use fastn_runtime::Params;

        self.register_memory_functions(linker);

        // this is quite tedious boilerplate, maybe we can write some macro to generate it
        linker
            .func_new(
                "fastn",
                "create_kernel",
                wasmtime::FuncType::new(
                    [wasmtime::ValType::ExternRef, wasmtime::ValType::I32]
                        .iter()
                        .cloned(),
                    [wasmtime::ValType::ExternRef].iter().cloned(),
                ),
                |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                    // ExternRef is a reference-counted pointer to a host-defined object. We mut not
                    // deallocate it on Rust side unless it's .strong_count() is 0. Not sure how it
                    // affects us yet.
                    results[0] = wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                        caller
                            .data_mut()
                            .create_kernel(params.key(0), params.i32(1).into()),
                    )));
                    Ok(())
                },
            )
            .unwrap();

        linker
            .func_new(
                "fastn",
                "set_i32_prop",
                wasmtime::FuncType::new(
                    [
                        wasmtime::ValType::ExternRef,
                        wasmtime::ValType::I32,
                        wasmtime::ValType::I32,
                    ]
                    .iter()
                    .cloned(),
                    [].iter().cloned(),
                ),
                |_caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, _results| {
                    // wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                    //     caller.data_mut().set_property(
                    //         params.key(0),
                    //         params.i32(0).into(),
                    //         params.i32(0).into(),
                    //     ),
                    // )));

                    todo!()
                },
            )
            .unwrap();

        linker
            .func_new(
                "fastn",
                "set_f32_prop",
                wasmtime::FuncType::new(
                    [
                        wasmtime::ValType::ExternRef,
                        wasmtime::ValType::I32,
                        wasmtime::ValType::F32,
                    ]
                    .iter()
                    .cloned(),
                    [].iter().cloned(),
                ),
                |_caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, _results| {
                    // wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                    //     caller.data_mut().set_property(
                    //         params.key(0),
                    //         params.i32(0).into(),
                    //         params.f32(0).into(),
                    //     ),
                    // )));

                    todo!()
                },
            )
            .unwrap();

        linker
            .func_new(
                "fastn",
                "set_column_width_px",
                wasmtime::FuncType::new(
                    [wasmtime::ValType::ExternRef, wasmtime::ValType::I32]
                        .iter()
                        .cloned(),
                    [].iter().cloned(),
                ),
                |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, p, _results| {
                    // ExternRef is a reference-counted pointer to a host-defined object. We mut not
                    // deallocate it on Rust side unless it's .strong_count() is 0. Not sure how it
                    // affects us yet.
                    caller.data_mut().set_element_width_px(p.key(0), p.i32(1));
                    Ok(())
                },
            )
            .unwrap();
    }
}

impl fastn_runtime::Memory {
    pub fn register(&self, linker: &mut wasmtime::Linker<fastn_runtime::Dom>) {
        use fastn_runtime::{Params, CallerExt, LinkerExt};
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
                        caller.memory_mut().create_boolean(params.boolean(0)),
                    )));

                    Ok(())
                },
            )
            .unwrap();

        linker.func1ret("create_i32", wasmtime::ValType::I32, |mem, v| {
            mem.create_i32(v.i32().unwrap()).into()
        });

        linker
            .func_new(
                "fastn",
                "create_rgba",
                wasmtime::FuncType::new(
                    [
                        wasmtime::ValType::I32,
                        wasmtime::ValType::I32,
                        wasmtime::ValType::I32,
                        wasmtime::ValType::F32,
                    ]
                    .iter()
                    .cloned(),
                    [wasmtime::ValType::ExternRef].iter().cloned(),
                ),
                |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                    results[0] = wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                        caller.memory_mut().create_rgba(
                            params.i32(0),
                            params.i32(1),
                            params.i32(2),
                            params.f32(3),
                        ),
                    )));
                    Ok(())
                },
            )
            .unwrap();

        linker.func0("create_frame", |mem| mem.create_frame());
        linker.func0("end_frame", |mem| mem.end_frame());

        linker
            .func_new(
                "fastn",
                "get_boolean",
                wasmtime::FuncType::new(
                    [wasmtime::ValType::ExternRef].iter().cloned(),
                    [wasmtime::ValType::I32].iter().cloned(),
                ),
                |caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, _results| {
                    let _s = &caller.memory();

                    // results[0] = wasmtime::Val::I32(s.boolean[params.ptr(0)].0 as i32);
                    Ok(())
                },
            )
            .unwrap();
    }
}



#[cfg(test)]
mod test {
    pub fn assert_import(name: &str, type_: &str) {
        fastn_runtime::Dom::create_instance(format!(
            r#"
                (module (import "fastn" "{}" (func {}))
                    (func (export "main")  (param externref))
                )
            "#,
            name, type_
        ));
    }

    #[test]
    fn dom() {
        assert_import("create_kernel", "(param externref i32) (result externref)");
    }

    #[test]
    fn memory() {
        assert_import("create_boolean", "(param i32) (result externref)");
        assert_import("create_frame", "");
        assert_import("end_frame", "");
        assert_import("create_rgba", "(param i32 i32 i32 f32) (result externref)");
    }
}
