pub struct Document {
    pub width: u32,
    pub height: u32,
    store: wasmtime::Store<fastn_runtime::Dom>,
    pub instance: wasmtime::Instance,
}

impl Document {
    pub fn new(wat: impl AsRef<[u8]>) -> Document {
        let engine = wasmtime::Engine::new(wasmtime::Config::new().async_support(false))
            .expect("cant create engine");
        let module = wasmtime::Module::new(&engine, wat).expect("cant parse module");
        let (store, instance) = fastn_runtime::Dom::register_funcs(
            wasmtime::Store::new(&engine, fastn_runtime::Dom::new()),
            &module,
        );

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
        _width: u32,
        _height: u32,
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
