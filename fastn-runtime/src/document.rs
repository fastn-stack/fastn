pub struct Document {
    pub width: u32,
    pub height: u32,
    store: wasmtime::Store<fastn_runtime::Dom>,
    pub instance: wasmtime::Instance,
}

impl Document {
    pub fn new(wat: impl AsRef<[u8]>) -> Document {
        let (store, instance) = fastn_runtime::Dom::create_instance(wat);

        Document {
            width: 0,
            height: 0,
            store,
            instance,
        }
    }

    pub fn handle_event(&mut self, event: fastn_runtime::Event) {
        self.store.data_mut().handle_event(event)
    }

    // initial_html() -> server side HTML
    // hydrate() -> client side
    // event_with_target() -> Vec<DomMutation>

    // if not wasm
    pub fn compute_layout(
        &mut self,
        width: u32,
        height: u32,
    ) -> (fastn_runtime::ControlFlow, Vec<fastn_runtime::Operation>) {
        (
            fastn_runtime::ControlFlow::WaitForEvent,
            self.store.data_mut().compute_layout(width, height),
        )
    }
    pub fn cursor_moved(&mut self, pos_x: f64, pos_y: f64) {
        self.store.data_mut().cursor_moved(pos_x, pos_y)
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
