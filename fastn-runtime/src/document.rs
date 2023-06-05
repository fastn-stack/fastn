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

    pub fn handle_event(&mut self, event: fastn_runtime::ExternalEvent) {
        match evt {
            fastn_runtime::ExternalEvent::CursorMoved { x, y } => self.cursor_moved(x, y),
            fastn_runtime::ExternalEvent::Focused(f) => self.has_focus = f,
            fastn_runtime::ExternalEvent::ModifierChanged(m) => self.modifiers = m,
            fastn_runtime::ExternalEvent::Key { code, pressed } => self.handle_key(code, pressed),
            _ => todo!(),
        }
    }

    fn handle_key(&mut self, _code: fastn_runtime::event::VirtualKeyCode, _pressed: bool) {
        self.store
            .get_export("call_by_index")
            .expect("call_by_index is not defined")
            .into_func()
            .expect("call_by_index not a func")
            .call(
                caller.as_context_mut(),
                &[
                    wasmtime::Val::I32(table_index),
                    wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(func_arg))),
                ],
                &mut values,
            )
            .expect("call failed");


        self.memory.handle_event(2.into(), None)
    }

    fn cursor_moved(&self, pos_x: f64, pos_y: f64) {
        // let _nodes = self.nodes_under_mouse(self.root, pos_x, pos_y);
        // todo!()
    }


    // initial_html() -> server side HTML
    pub fn initial_html(&self) -> String {
        todo!()
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
