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

    pub fn imports() -> Vec<fastn_wasm::Ast> {
        let mut e = fastn_runtime::Memory::exports();
        e.extend([
            fastn_wasm::import::func2ret(
                "create_kernel",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::ExternRef,
            ),
            fastn_wasm::import::func3(
                "set_i32_prop",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::I32.into(),
            ),
            fastn_wasm::import::func3(
                "set_f32_prop",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::F32.into(),
            ),
        ]);
        e
    }

    fn register_functions(&self, linker: &mut wasmtime::Linker<fastn_runtime::Dom>) {
        use fastn_runtime::wasm_helpers::Params;
        use fastn_wasm::LinkerExt;
        use wasmtime::AsContextMut;

        self.register_memory_functions(linker);

        linker.func2ret(
            "create_kernel",
            |dom: &mut fastn_runtime::Dom, parent, kind| dom.create_kernel(parent, kind),
        );
        linker.func3(
            "set_i32_prop",
            |dom: &mut fastn_runtime::Dom, key, property_kind, value| {
                dom.set_property(key, property_kind, fastn_runtime::dom::Value::I32(value))
            },
        );
        linker.func3(
            "set_f32_prop",
            |dom: &mut fastn_runtime::Dom, key, property_kind, value| {
                dom.set_property(key, property_kind, fastn_runtime::dom::Value::F32(value))
            },
        );

        linker.func4caller(
            "set_i32_prop_func",
            |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>,
             node_key,
             ui_property,
             table_index,
             func_arg: fastn_runtime::PointerKey| {
                let mut values = vec![];
                caller
                    .get_export("callByIndex")
                    .unwrap()
                    .into_func()
                    .expect("callByIndex not a func")
                    .call(
                        caller.as_context_mut(),
                        &[
                            wasmtime::Val::I32(table_index),
                            wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(func_arg))),
                        ],
                        &mut values,
                    )
                    .expect("call failed");

                let value = values.i32(0);
                let dom = caller.data_mut();
                dom.set_property(node_key, ui_property, value.into());

                let mem = dom.memory_mut();
                let closure_key = mem.create_closure(fastn_runtime::Closure {
                    function: table_index,
                    function_data: func_arg.into_list_pointer(),
                });
                mem.add_ui_dependent(
                    func_arg.into_list_pointer(),
                    ui_property.into_ui_dependent(node_key).closure(closure_key),
                );
            },
        );

        linker.func4caller(
            "set_i32_3_prop_func",
            |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>,
             node_key,
             ui_property,
             table_index,
             func_arg| {
                let mut value = vec![];
                caller
                    .get_export("callByIndex")
                    .unwrap()
                    .into_func()
                    .expect("callByIndex not a func")
                    .call(
                        caller.as_context_mut(),
                        &[
                            wasmtime::Val::I32(table_index),
                            wasmtime::Val::ExternRef(Some(func_arg)),
                        ],
                        &mut value,
                    )
                    .expect("call failed");
                caller
                    .data_mut()
                    .set_property(node_key, ui_property, value.ptr(0).into());
            },
        );
    }
}

impl fastn_runtime::Memory {
    pub fn exports() -> Vec<fastn_wasm::Ast> {
        vec![
            fastn_wasm::import::func00("create_frame"),
            fastn_wasm::import::func00("end_frame"),
            fastn_wasm::import::func1ret(
                "return_frame",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::ExternRef,
            ),
            fastn_wasm::import::func1ret(
                "create_boolean",
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::ExternRef,
            ),
            fastn_wasm::import::func1ret(
                "get_boolean",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::I32,
            ),
            fastn_wasm::import::func1ret(
                "create_i32",
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::ExternRef,
            ),
            fastn_wasm::import::func1ret(
                "get_i32",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::I32,
            ),
        ]
    }

    pub fn register(&self, linker: &mut wasmtime::Linker<fastn_runtime::Dom>) {
        use fastn_wasm::LinkerExt;

        linker.func0("create_frame", |mem: &mut fastn_runtime::Memory| {
            mem.create_frame()
        });
        linker.func0("end_frame", |mem: &mut fastn_runtime::Memory| {
            mem.end_frame()
        });
        linker.func1ret("return_frame", |mem: &mut fastn_runtime::Memory, ret| {
            mem.return_frame(ret)
        });
        linker.func1ret("create_boolean", |mem: &mut fastn_runtime::Memory, v| {
            mem.create_boolean(v)
        });
        linker.func1ret("get_boolean", |mem: &mut fastn_runtime::Memory, ptr| {
            mem.get_boolean(ptr)
        });
        linker.func1ret("create_i32", |mem: &mut fastn_runtime::Memory, v| {
            mem.create_i32(v)
        });
        linker.func1ret("get_i32", |mem: &mut fastn_runtime::Memory, v| {
            mem.get_i32(v)
        });
        linker.func4ret(
            "create_rgba",
            |mem: &mut fastn_runtime::Memory, r, g, b, a| mem.create_rgba(r, g, b, a),
        );
        linker.func2ret(
            "array_i32_2",
            |mem: &mut fastn_runtime::Memory, ptr1, ptr2| mem.array_i32_2(ptr1, ptr2),
        );
        linker.func2ret(
            "get_func_arg_i32",
            |mem: &mut fastn_runtime::Memory, ptr, idx| mem.get_func_arg_i32(ptr, idx),
        );
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
    pub fn assert_import0(name: &str) {
        assert_import(name, "")
    }

    #[test]
    fn dom() {
        assert_import("create_kernel", "(param externref i32) (result externref)");
    }

    #[test]
    fn memory() {
        assert_import0("create_frame");
        assert_import0("end_frame");
        assert_import("create_boolean", "(param i32) (result externref)");
        assert_import("get_boolean", "(param externref) (result i32)");
        assert_import("create_i32", "(param i32) (result externref)");
        assert_import("get_i32", "(param externref) (result i32)");
        assert_import("create_rgba", "(param i32 i32 i32 f32) (result externref)");
        assert_import(
            "array_i32_2",
            "(param externref externref) (result externref)",
        )
    }
}
