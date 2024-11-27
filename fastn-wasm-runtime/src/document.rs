pub struct Document {
    store: wasmtime::Store<fastn_runtime::Dom>,
    pub instance: wasmtime::Instance,
}

impl Document {
    pub fn new(wat: impl AsRef<[u8]>) -> Document {
        let (store, instance) = fastn_runtime::Dom::create_instance(wat);

        Document {
            store,
            instance,
        }
    }

    #[cfg(feature = "render")]
    pub fn handle_event(&mut self, event: fastn_runtime::ExternalEvent) {
        let node = self.get_node_for_event(event);
        self.handle_event_with_target(event, node);
    }

    #[cfg(feature = "render")]
    pub fn get_node_for_event(&self, _event: fastn_runtime::ExternalEvent) -> fastn_runtime::NodeKey {
        // TODO
        self.store.data().root
    }

    pub fn handle_event_with_target(&mut self, event: fastn_runtime::ExternalEvent, _node: fastn_runtime::NodeKey) {
        match event {
            fastn_runtime::ExternalEvent::CursorMoved { x, y } => self.cursor_moved(x, y),
            fastn_runtime::ExternalEvent::Focused(f) => self.store.data_mut().has_focus = f,
            fastn_runtime::ExternalEvent::ModifierChanged(m) => self.store.data_mut().modifiers = m,
            fastn_runtime::ExternalEvent::Key { code, pressed } => self.handle_key(code, pressed),
            _ => todo!(),
        }
    }

    fn handle_key(&mut self, code: fastn_runtime::event::VirtualKeyCode, _pressed: bool) {
        dbg!(&code);

        let memory = &self.store.data().memory;
        let closures = memory.closure.clone();

        if let Some(events) = memory
            .get_event_handlers(fastn_runtime::DomEventKind::OnGlobalKey, None)
            .map(|v| v.to_vec())
        {
            for event in events {
                let closure = closures.get(event.closure).unwrap();

                // Create a temporary variable to hold the export
                let void_by_index = self
                    .instance
                    .get_export(&mut self.store, "void_by_index")
                    .expect("void_by_index is not defined");

                // Make the call using the temporary variable
                void_by_index
                    .into_func()
                    .expect("void_by_index not a func")
                    .call(
                        &mut self.store,
                        &[
                            wasmtime::Val::I32(closure.function),
                            wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                                closure.captured_variables.pointer,
                            ))),
                        ],
                        &mut [],
                    )
                    .expect("void_by_index failed");
            }
        }
    }

    fn cursor_moved(&self, _pos_x: f64, _pos_y: f64) {
        // let _nodes = self.nodes_under_mouse(self.root, pos_x, pos_y);
        // todo!()
    }

    // initial_html() -> server side HTML
    pub fn initial_html(&self) -> String {
        fastn_runtime::server::html::initial(self.store.data())
    }
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

    // if not wasm
    pub async fn event(
        &mut self,
        _e: fastn_runtime::ExternalEvent,
    ) -> (fastn_runtime::ControlFlow, Vec<fastn_runtime::Operation>) {
        // find the event target based on current layout and event coordinates
        // handle event, which will update the dom tree
        // compute layout
        (fastn_runtime::ControlFlow::WaitForEvent, vec![])
    }
}
