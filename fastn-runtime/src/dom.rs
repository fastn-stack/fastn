pub struct Dom {
    pub taffy: taffy::Taffy,
    pub nodes: slotmap::SlotMap<fastn_runtime::NodeKey, fastn_runtime::Element>,
    pub root: fastn_runtime::NodeKey,
}

impl Dom {
    pub fn new() -> Dom {
        let mut nodes = slotmap::SlotMap::with_key();
        let mut taffy = taffy::Taffy::new();
        let root = nodes.insert(fastn_runtime::Container::outer_column(&mut taffy));

        Dom { taffy, nodes, root }
    }

    pub fn to_operations(&self) -> Vec<fastn_runtime::Operation> {
        vec![]
    }

    pub fn create_column(&mut self) -> fastn_runtime::NodeKey {
        self.nodes
            .insert(fastn_runtime::Container::outer_column(&mut self.taffy))
    }

    pub fn add_child(&mut self, parent: fastn_runtime::NodeKey, child: fastn_runtime::NodeKey) {
        self.taffy.add_child(
            self.nodes[parent].taffy(),
            self.nodes[child].taffy(),
        ).unwrap();
    }

    pub fn create_instance(
        wat: impl AsRef<[u8]>,
    ) -> (wasmtime::Store<fastn_runtime::Dom>, wasmtime::Instance) {
        let engine = wasmtime::Engine::new(wasmtime::Config::new().async_support(false))
            .expect("cant create engine");
        let module = wasmtime::Module::new(&engine, wat).expect("cant parse module");

        let mut linker = wasmtime::Linker::new(&engine);

        // this is quite tedious boilerplate, maybe we can write some macro to generate it
        linker.func_new(
            "fastn", "create_column",
            wasmtime::FuncType::new(
                [].iter().cloned(),
                [wasmtime::ValType::ExternRef].iter().cloned(),
            ),
            |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, results| {
                // ExternRef is a reference-counted pointer to a host-defined object. We mut not
                // deallocate it on Rust side unless it's .strong_count() is 0. Not sure how it
                // affects us yet.
                results[0] = wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                    caller.data_mut().create_column(),
                )));
                Ok(())
            },
        ).unwrap();
        linker.func_new(
            "fastn", "root_container",
            wasmtime::FuncType::new(
                [].iter().cloned(),
                [wasmtime::ValType::ExternRef].iter().cloned(),
            ),
            |caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, results| {
                // ExternRef is a reference-counted pointer to a host-defined object. We mut not
                // deallocate it on Rust side unless it's .strong_count() is 0. Not sure how it
                // affects us yet.
                results[0] = wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                    caller.data().root,
                )));
                Ok(())
            },
        ).unwrap();
        linker.func_new(
            "fastn", "add_child",
            wasmtime::FuncType::new(
                [wasmtime::ValType::ExternRef, wasmtime::ValType::ExternRef].iter().cloned(),
                [].iter().cloned(),
            ),
            |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, _results| {
                // ExternRef is a reference-counted pointer to a host-defined object. We mut not
                // deallocate it on Rust side unless it's .strong_count() is 0. Not sure how it
                // affects us yet.
                caller.data_mut().add_child(
                    *params[0].externref().unwrap().expect("externref gone?").data().downcast_ref().unwrap(),
                    *params[1].externref().unwrap().expect("externref gone?").data().downcast_ref().unwrap(),
                );
                Ok(())
            },
        ).unwrap();

        let mut store = wasmtime::Store::new(&engine, fastn_runtime::Dom::new());
        let instance = linker.instantiate(&mut store, &module).expect("cant create instance");

        (store, instance)
    }
}

#[cfg(test)]
mod test {
    fn assert_import(name: &str, type_: &str) {
        fastn_runtime::Dom::create_instance(format!(
            r#"
                (module (import "fastn" "{}" (func {})))
            "#,
            name, type_
        ));
    }

    #[test]
    fn test() {
        assert_import("create_column", "(result externref)");
        assert_import("root_container", "(result externref)");
        assert_import("add_child", "(param externref externref)");
    }
}
