pub struct Document {
    pub width: u32,
    pub height: u32,
    store: wasmtime::Store<fastn_runtime::Dom>,
    instance: wasmtime::Instance,
}

impl Document {
    pub fn new(wat: impl AsRef<[u8]>) -> Document {
        let engine = wasmtime::Engine::new(wasmtime::Config::new().async_support(false))
            .expect("cant create engine");
        let module = wasmtime::Module::new(&engine, wat).expect("cant parse module");

        let mut store = wasmtime::Store::new(&engine, fastn_runtime::Dom::new());

        let create_column_type = wasmtime::FuncType::new(
            [].iter().cloned(),
            [wasmtime::ValType::ExternRef].iter().cloned(),
        );
        let create_column = wasmtime::Func::new(&mut store, create_column_type, |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, results| {
            // wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(caller.data_mut().create_column())))
            results[0] = wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(caller.data_mut().create_column())));
            Ok(())
        });

        let instance = wasmtime::Instance::new(&mut store, &module, &[create_column.into()])
            .expect("cant create instance");

        Document {
            width: 0,
            height: 0,
            store,
            instance,
        }
    }

    // initial_html() -> server side HTML
    // hydrate() -> client side
    // event_with_target() -> Vec<DomMutation>

    // if not wasm
    pub fn initial_layout(
        &mut self,
        width: u32,
        height: u32,
    ) -> (fastn_runtime::ControlFlow, Vec<fastn_runtime::Operation>) {
        // let taffy_root = self.nodes[self.root].taffy();
        // self.taffy
        //     .compute_layout(
        //         taffy_root,
        //         taffy::prelude::Size {
        //             width: taffy::prelude::points(width as f32),
        //             height: taffy::prelude::points(height as f32),
        //         },
        //     )
        //     .unwrap();
        // self.width = width;
        // self.height = height;
        // dbg!(self.taffy.layout(taffy_root).unwrap());

        (
            fastn_runtime::ControlFlow::WaitForEvent,
            self.store.data().to_operations(),
        )
    }

    // if not wasm
    pub async fn event(
        &mut self,
        _e: fastn_runtime::Event,
    ) -> (fastn_runtime::ControlFlow, Vec<fastn_runtime::Operation>) {
        // find the event target based on current layout and event coordinates
        // handle event, which will update the dom tree
        // compute layout
        (fastn_runtime::ControlFlow::WaitForEvent, vec![])
    }
}

