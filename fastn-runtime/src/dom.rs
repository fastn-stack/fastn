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

    pub fn register_funcs(
        mut store: wasmtime::Store<fastn_runtime::Dom>,
        module: &wasmtime::Module,
    ) -> (wasmtime::Store<fastn_runtime::Dom>, wasmtime::Instance) {
        // this is quite tedious boilerplate, maybe we can write some macro to generate it
        let create_column = wasmtime::Func::new(
            &mut store,
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
        );

        let instance = wasmtime::Instance::new(&mut store, module, &[create_column.into()])
            .expect("cant create instance");

        (store, instance)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_create_column() {
        let test_wat = r#"
        (module
            (import "fastn" "create_column" (func $create_column (result externref)))
        )
        "#;
        let engine = wasmtime::Engine::new(wasmtime::Config::new().async_support(false))
            .expect("cant create engine");
        let module = wasmtime::Module::new(
            &engine,
            test_wat,
        )
        .expect("cant parse module");

        fastn_runtime::Dom::register_funcs(
            wasmtime::Store::new(&engine, fastn_runtime::Dom::new()),
            &module,
        );
    }
}
