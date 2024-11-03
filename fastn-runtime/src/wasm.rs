impl fastn_runtime::Dom {
    pub fn create_instance(
        wat: impl AsRef<[u8]>,
    ) -> (wasmtime::Store<fastn_runtime::Dom>, wasmtime::Instance) {
        let engine = wasmtime::Engine::new(wasmtime::Config::new().async_support(false))
            .expect("cant create engine");
        let module = wasmtime::Module::new(&engine, wat).expect("cant unresolved module");
        let dom = fastn_runtime::Dom::new(0, 0);

        let mut linker = wasmtime::Linker::new(&engine);

        dom.register_functions(&mut linker);

        let mut store = wasmtime::Store::new(&engine, dom);
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
                "set_property_i32",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::I32.into(),
            ),
            fastn_wasm::import::func3(
                "set_property_f32",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::F32.into(),
            ),
            fastn_wasm::import::func4(
                "set_dynamic_property_i32",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::ExternRef.into(),
            ),
            fastn_wasm::import::func4(
                "set_dynamic_property_color",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::ExternRef.into(),
            ),
        ]);
        e
    }

    fn register_functions(&self, linker: &mut wasmtime::Linker<fastn_runtime::Dom>) {
        use fastn_runtime::wasm_helpers::Params;
        use fastn_wasm::LinkerExt;

        self.register_memory_functions(linker);

        linker.func2ret(
            "create_kernel",
            |dom: &mut fastn_runtime::Dom, parent, kind| dom.create_kernel(parent, kind),
        );
        linker.func3(
            "set_property_i32",
            |dom: &mut fastn_runtime::Dom, key, property_kind, value| {
                dom.set_property(key, property_kind, fastn_runtime::dom::Value::I32(value))
            },
        );
        linker.func3(
            "set_property_f32",
            |dom: &mut fastn_runtime::Dom, key, property_kind, value| {
                dom.set_property(key, property_kind, fastn_runtime::dom::Value::F32(value))
            },
        );

        linker.func4_caller(
            "set_dynamic_property_i32",
            |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>,
             node_key,
             ui_property,
             table_index,
             func_arg| {
                // TODO: refactor this into a generic helper
                let current_value_of_dynamic_property = {
                    let mut values = vec![wasmtime::Val::I32(0)];
                    caller
                        .get_export("call_by_index")
                        .expect("call_by_index is not defined")
                        .into_func()
                        .expect("call_by_index not a func")
                        .call(
                            &mut caller,
                            &[
                                wasmtime::Val::I32(table_index),
                                wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(func_arg))),
                            ],
                            &mut values,
                        )
                        .expect("call failed");

                    caller.data().memory().get_i32(values.ptr(0))
                };

                caller.data_mut().set_dynamic_property(
                    node_key,
                    ui_property,
                    table_index,
                    func_arg,
                    current_value_of_dynamic_property.into(),
                )
            },
        );

        linker.func4_caller(
            "set_dynamic_property_color",
            |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>,
             node_key,
             ui_property,
             table_index,
             func_arg| {
                // TODO: refactor this into a generic helper
                let current_value_of_dynamic_property = {
                    let mut values = vec![wasmtime::Val::I32(0)];
                    caller
                        .get_export("call_by_index")
                        .expect("call_by_index is not defined")
                        .into_func()
                        .expect("call_by_index not a func")
                        .call(
                            &mut caller,
                            &[
                                wasmtime::Val::I32(table_index),
                                wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(func_arg))),
                            ],
                            &mut values,
                        )
                        .expect("call failed");

                    caller.data().memory().get_colors(values.ptr(0))
                };

                caller.data_mut().set_dynamic_property(
                    node_key,
                    ui_property,
                    table_index,
                    func_arg,
                    current_value_of_dynamic_property.into(),
                )
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
                "get_global",
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::ExternRef,
            ),
            fastn_wasm::import::func2(
                "set_global",
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::ExternRef.into(),
            ),
            fastn_wasm::import::func1ret(
                "create_boolean",
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::ExternRef,
            ),
            fastn_wasm::import::func0("create_list", fastn_wasm::Type::ExternRef),
            fastn_wasm::import::func2ret(
                "create_list_1",
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::ExternRef,
            ),
            fastn_wasm::import::func4ret(
                "create_list_2",
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::ExternRef,
            ),
            fastn_wasm::import::func1ret(
                "get_boolean",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::I32,
            ),
            fastn_wasm::import::func2(
                "set_boolean",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::I32.into(),
            ),
            fastn_wasm::import::func2ret(
                "get_func_arg_ref",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::ExternRef,
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
            fastn_wasm::import::func2(
                "set_i32",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::I32.into(),
            ),
            fastn_wasm::import::func1ret(
                "create_f32",
                fastn_wasm::Type::F32.into(),
                fastn_wasm::Type::ExternRef,
            ),
            fastn_wasm::import::func3ret(
                "multiply_i32",
                /* func-data */ fastn_wasm::Type::ExternRef.into(),
                /* idx_1 */ fastn_wasm::Type::I32.into(),
                /* idx_2 */ fastn_wasm::Type::I32.into(),
                /* func-data[idx_1] * func-data[idx_2] */ fastn_wasm::Type::ExternRef,
            ),
            fastn_wasm::import::func1ret(
                "get_f32",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::F32,
            ),
            fastn_wasm::import::func2(
                "set_f32",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::F32.into(),
            ),
            fastn_wasm::import::func2ret(
                "create_string_constant",
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::ExternRef,
            ),
            fastn_wasm::import::func2ret(
                "array_i32_2",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::ExternRef,
            ),
            fastn_wasm::import::func4(
                "attach_event_handler",
                fastn_wasm::Type::ExternRef.into(),
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::I32.into(),
                fastn_wasm::Type::ExternRef.into(),
            ),
        ]
    }

    pub fn register(&self, linker: &mut wasmtime::Linker<fastn_runtime::Dom>) {
        use fastn_runtime::wasm_helpers::Params;
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
        linker.func1ret("get_global", |mem: &mut fastn_runtime::Memory, idx| {
            mem.get_global(idx)
        });
        linker.func2("set_global", |mem: &mut fastn_runtime::Memory, idx, ptr| {
            mem.set_global(idx, ptr)
        });
        linker.func0ret("create_list", |mem: &mut fastn_runtime::Memory| {
            mem.create_list()
        });
        linker.func2ret(
            "create_list_1",
            |mem: &mut fastn_runtime::Memory, v1_kind, v1_ptr| mem.create_list_1(v1_kind, v1_ptr),
        );
        linker.func4ret(
            "create_list_2",
            |mem: &mut fastn_runtime::Memory, v1_kind, v1_ptr, v2_kind, v2_ptr| {
                mem.create_list_2(v1_kind, v1_ptr, v2_kind, v2_ptr)
            },
        );
        linker.func2ret_caller(
            "create_string_constant",
            |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, start: i32, length: i32| {
                let mut buffer = Vec::with_capacity(length as usize);
                caller
                    .get_export("memory")
                    .unwrap()
                    .into_memory()
                    .unwrap()
                    .read(&caller, start as usize, &mut buffer)
                    .unwrap();
                caller.data_mut().memory.create_string_constant(buffer)
            },
        );
        linker.func1ret("create_boolean", |mem: &mut fastn_runtime::Memory, v| {
            mem.create_boolean(v)
        });
        linker.func1ret("get_boolean", |mem: &mut fastn_runtime::Memory, ptr| {
            mem.get_boolean(ptr)
        });
        linker.func2_caller(
            "set_boolean",
            |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, ptr, value| {
                caller.data_mut().memory.boolean[ptr].value.set_value(value);

                for ui_property in caller.data().memory.boolean[ptr].ui_properties.clone() {
                    let closure_pointer = caller
                        .data()
                        .memory
                        .closure
                        .get(ui_property.closure)
                        .unwrap()
                        .clone();
                    let current_value_of_dynamic_property = {
                        let mut values = vec![wasmtime::Val::I32(0)];
                        caller
                            .get_export("call_by_index")
                            .expect("call_by_index is not defined")
                            .into_func()
                            .expect("call_by_index not a func")
                            .call(
                                &mut caller,
                                &[
                                    wasmtime::Val::I32(closure_pointer.function),
                                    wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                                        closure_pointer.captured_variables.pointer,
                                    ))),
                                ],
                                &mut values,
                            )
                            .expect("call failed");

                        // Todo: check ui_property.property
                        caller.data().memory.get_i32(values.ptr(0))
                    };

                    dbg!("set_boolean***", &current_value_of_dynamic_property);

                    caller.data_mut().set_property(
                        ui_property.node,
                        ui_property.property,
                        current_value_of_dynamic_property.into(),
                    )
                }
            },
        );
        linker.func1ret("create_i32", |mem: &mut fastn_runtime::Memory, v| {
            mem.create_i32(v)
        });
        linker.func1ret("get_i32", |mem: &mut fastn_runtime::Memory, ptr| {
            mem.get_i32(ptr)
        });
        linker.func2_caller(
            "set_i32",
            |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, ptr, value| {
                dbg!("set_i32", &ptr);
                caller.data_mut().memory.i32[ptr].value.set_value(value);
                dbg!(&caller.data().memory.i32[ptr]);

                for dependent in caller.data().memory.i32[ptr].parents.clone() {
                    for ui_property in caller.data().memory.vec[dependent.pointer]
                        .ui_properties
                        .clone()
                    {
                        let closure_pointer = caller
                            .data()
                            .memory
                            .closure
                            .get(ui_property.closure)
                            .unwrap()
                            .clone();
                        dbg!(&closure_pointer);
                        let current_value_of_dynamic_property = {
                            let mut values = vec![wasmtime::Val::I32(0)];
                            caller
                                .get_export("call_by_index")
                                .expect("call_by_index is not defined")
                                .into_func()
                                .expect("call_by_index not a func")
                                .call(
                                    &mut caller,
                                    &[
                                        wasmtime::Val::I32(0), // TODO: arpita: closure_pointer.function
                                        wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                                            closure_pointer.captured_variables.pointer,
                                        ))),
                                    ],
                                    &mut values,
                                )
                                .expect("call failed");

                            // Todo: check ui_property.property
                            caller.data().memory.get_i32(values.ptr(0))
                        };

                        dbg!("set_i32***", &current_value_of_dynamic_property);

                        caller.data_mut().set_property(
                            ui_property.node,
                            ui_property.property,
                            current_value_of_dynamic_property.into(),
                        )
                    }
                }
            },
        );
        linker.func3ret(
            "multiply_i32",
            |mem: &mut fastn_runtime::Memory, arr, idx_1, idx_2| {
                mem.multiply_i32(arr, idx_1, idx_2)
            },
        );
        linker.func1ret("create_f32", |mem: &mut fastn_runtime::Memory, v| {
            mem.create_f32(v)
        });
        linker.func1ret("get_f32", |mem: &mut fastn_runtime::Memory, ptr| {
            mem.get_f32(ptr)
        });
        linker.func2("set_f32", |mem: &mut fastn_runtime::Memory, ptr, v| {
            mem.set_f32(ptr, v)
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
        linker.func2ret(
            "get_func_arg_ref",
            |mem: &mut fastn_runtime::Memory, ptr, idx| mem.get_func_arg_ref(ptr, idx),
        );
        linker.func4(
            "attach_event_handler",
            |mem: &mut fastn_runtime::Memory, node_key, event, table_index, func_arg| {
                mem.attach_event_handler(node_key, event, table_index, func_arg)
            },
        );
    }
}

#[cfg(test)]
mod test {
    pub fn assert_import(name: &str, type_: &str) {
        fastn_runtime::Dom::create_instance(format!(
            r#"
                (module (import "fastn" "{}" (func {}))
                    (func (export "main") (param externref))
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
        assert_import("set_property_i32", "(param externref i32 i32)");
        assert_import("set_property_f32", "(param externref i32 f32)");
        assert_import(
            "set_dynamic_property_i32",
            "(param externref i32 i32 externref)",
        );
        assert_import(
            "set_dynamic_property_color",
            "(param externref i32 i32 externref)",
        );
        assert_import(
            "attach_event_handler",
            "(param externref i32 i32 externref)",
        );
    }

    #[test]
    fn memory() {
        assert_import0("create_frame");
        assert_import0("end_frame");
        assert_import("return_frame", "(param externref) (result externref)");
        assert_import("set_global", "(param i32 externref)");
        assert_import("get_global", "(param i32) (result externref)");
        assert_import("create_list", "(result externref)");
        assert_import("create_list_1", "(param i32 externref) (result externref)");
        assert_import(
            "create_list_2",
            "(param i32 externref i32 externref) (result externref)",
        );
        assert_import("create_boolean", "(param i32) (result externref)");
        assert_import("get_boolean", "(param externref) (result i32)");
        assert_import("set_boolean", "(param externref i32)");
        assert_import("create_i32", "(param i32) (result externref)");
        assert_import("get_i32", "(param externref) (result i32)");
        assert_import("set_i32", "(param externref i32)");
        assert_import(
            "multiply_i32",
            "(param externref i32 i32) (result externref)",
        );
        assert_import("create_f32", "(param f32) (result externref)");
        assert_import("get_f32", "(param externref) (result f32)");
        assert_import("set_f32", "(param externref f32)");
        assert_import(
            "array_i32_2",
            "(param externref externref) (result externref)",
        );
        assert_import("create_rgba", "(param i32 i32 i32 f32) (result externref)");
        assert_import(
            "array_i32_2",
            "(param externref externref) (result externref)",
        )
    }
}
