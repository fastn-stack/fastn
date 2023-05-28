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

pub trait ParamExtractor<T> {
    fn extract(&self, idx: usize) -> T;
}

impl ParamExtractor<i32> for [wasmtime::Val] {
    fn extract(&self, idx: usize) -> i32 {
        self.i32(idx)
    }
}

pub trait Params {
    fn i32(&self, idx: usize) -> i32;
    fn f32(&self, idx: usize) -> f32;
    fn key(&self, idx: usize) -> fastn_runtime::NodeKey;
    fn ptr(&self, idx: usize) -> fastn_runtime::PointerKey;
    fn boolean(&self, idx: usize) -> bool;
}

impl Params for [wasmtime::Val] {
    fn i32(&self, idx: usize) -> i32 {
        self[idx].i32().unwrap()
    }

    fn f32(&self, idx: usize) -> f32 {
        self[idx].f32().unwrap()
    }

    fn key(&self, idx: usize) -> fastn_runtime::NodeKey {
        *self[idx]
            .externref()
            .unwrap()
            .expect("externref gone?")
            .data()
            .downcast_ref()
            .unwrap()
    }
    fn ptr(&self, idx: usize) -> fastn_runtime::PointerKey {
        *self[idx]
            .externref()
            .unwrap()
            .expect("externref gone?")
            .data()
            .downcast_ref()
            .unwrap()
    }

    fn boolean(&self, idx: usize) -> bool {
        self.i32(idx) != 0
    }
}

impl fastn_runtime::Memory {
    pub fn register(&self, linker: &mut wasmtime::Linker<fastn_runtime::Dom>) {
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

trait LinkerExt {
    fn func0(
        &mut self,
        name: &str,
        func: impl Fn(&mut fastn_runtime::Memory) + Send + Sync + 'static,
    );
    // fn func1<T: ParamExtractor<T>>(&mut self, name: &str, func: impl Fn(&mut fastn_runtime::Memory, T) + Send + Sync + 'static);
    fn func1(
        &mut self,
        name: &str,
        arg1: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val) + Send + Sync + 'static,
    );
    fn func2(
        &mut self,
        name: &str,
        arg1: wasmtime::ValType,
        arg2: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val, &wasmtime::Val)
            + Send
            + Sync
            + 'static,
    );
    fn func0ret(
        &mut self,
        name: &str,
        func: impl Fn(&mut fastn_runtime::Memory) -> wasmtime::Val + Send + Sync + 'static,
    );
    fn func1ret(
        &mut self,
        name: &str,
        arg: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val) -> wasmtime::Val
            + Send
            + Sync
            + 'static,
    );
    fn func2ret(
        &mut self,
        name: &str,
        arg1: wasmtime::ValType,
        arg2: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val, &wasmtime::Val) -> wasmtime::Val
            + Send
            + Sync
            + 'static,
    );
}

impl LinkerExt for wasmtime::Linker<fastn_runtime::Dom> {
    fn func0(
        &mut self,
        name: &str,
        func: impl Fn(&mut fastn_runtime::Memory) + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, _results| {
                func(caller.memory_mut());
                Ok(())
            },
        )
        .unwrap();
    }
    fn func1(
        &mut self,
        name: &str,
        arg1: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val) + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([arg1].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, _results| {
                func(caller.memory_mut(), &params[0]);
                Ok(())
            },
        )
        .unwrap();
    }
    fn func2(
        &mut self,
        name: &str,
        arg1: wasmtime::ValType,
        arg2: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val, &wasmtime::Val)
            + Send
            + Sync
            + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([arg1, arg2].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, _results| {
                func(caller.memory_mut(), &params[0], &params[1]);
                Ok(())
            },
        )
        .unwrap();
    }
    fn func0ret(
        &mut self,
        name: &str,
        func: impl Fn(&mut fastn_runtime::Memory) -> wasmtime::Val + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, results| {
                results[0] = func(caller.memory_mut());
                Ok(())
            },
        )
        .unwrap();
    }
    fn func1ret(
        &mut self,
        name: &str,
        arg: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val) -> wasmtime::Val
            + Send
            + Sync
            + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([arg].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                results[0] = func(caller.memory_mut(), &params[0]);
                Ok(())
            },
        )
        .unwrap();
    }
    fn func2ret(
        &mut self,
        name: &str,
        arg1: wasmtime::ValType,
        arg2: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val, &wasmtime::Val) -> wasmtime::Val
            + Send
            + Sync
            + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([arg1, arg2].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                results[0] = func(caller.memory_mut(), &params[0], &params[1]);
                Ok(())
            },
        )
        .unwrap();
    }
}

impl From<fastn_runtime::PointerKey> for wasmtime::Val {
    fn from(value: fastn_runtime::PointerKey) -> Self {
        wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(value)))
    }
}

trait CallerExt {
    fn memory(&self) -> &fastn_runtime::Memory;
    fn memory_mut(&mut self) -> &mut fastn_runtime::Memory;
}

impl CallerExt for wasmtime::Caller<'_, fastn_runtime::Dom> {
    fn memory(&self) -> &fastn_runtime::Memory {
        self.data().memory()
    }
    fn memory_mut(&mut self) -> &mut fastn_runtime::Memory {
        self.data_mut().memory_mut()
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
