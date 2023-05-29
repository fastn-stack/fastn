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
        use fastn_runtime::LinkerExt;
        use fastn_runtime::Params;

        self.register_memory_functions(linker);

        // this is quite tedious boilerplate, maybe we can write some macro to generate it
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
    }
}

impl fastn_runtime::Memory {
    pub fn register(&self, linker: &mut wasmtime::Linker<fastn_runtime::Dom>) {
        use fastn_runtime::LinkerExt;

        linker.func0("create_frame", |mem: &mut fastn_runtime::Memory| {
            mem.create_frame()
        });
        linker.func0("end_frame", |mem: &mut fastn_runtime::Memory| {
            mem.end_frame()
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
