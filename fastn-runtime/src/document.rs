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
                /*let pointer_vector = store
                    .memory
                    .vec
                    .get(closure.captured_variables.pointer)
                    .unwrap().value.value();
                for pointer in pointer_vector {
                    match pointer.kind {
                        fastn_runtime::PointerKind::Boolean => store
                            .memory.boolean.get(pointer.pointer)
                        fastn_runtime::PointerKind::Integer => {}
                        fastn_runtime::PointerKind::Record => {}
                        fastn_runtime::PointerKind::OrType => {}
                        fastn_runtime::PointerKind::Decimal => {}
                        fastn_runtime::PointerKind::List => {}
                        fastn_runtime::PointerKind::String => {}
                    }
                }*/
            }
        }

        // self.memory.handle_event(2.into(), None)
    }

    fn resolve_pointer(&mut self, _pointer: &fastn_runtime::Pointer) {
        /*let store = self.store.data_mut();

        match pointer.kind {
            fastn_runtime::PointerKind::Boolean => store
                .memory.boolean.get(pointer.pointer).unwrap().value
            fastn_runtime::PointerKind::Integer => {}
            fastn_runtime::PointerKind::Record => {}
            fastn_runtime::PointerKind::OrType => {}
            fastn_runtime::PointerKind::Decimal => {}
            fastn_runtime::PointerKind::List => {}
            fastn_runtime::PointerKind::String => {}
        }*/
    }

    fn resolve_heap_value<T>(
        &mut self,
        _kind: fastn_runtime::PointerKind,
        _data: &mut fastn_runtime::HeapValue<T>,
    ) {
        /*use fastn_runtime::wasm_helpers::Params;
        use wasmtime::AsContextMut;

        let context = self.store.as_context_mut();
        let closures = self.store.data().memory.closure.clone();

        if let fastn_runtime::HeapValue::Formula {
            cached_value,
            closure,
        } = data
        {
            let closure = closures.get(*closure).unwrap();
            let mut values = vec![wasmtime::Val::I32(0)];
            self.instance
                .get_export(self.store.as_context_mut(), "call_by_index")
                .expect("call_by_index is not defined")
                .into_func()
                .expect("call_by_index not a func")
                .call(
                    context,
                    &[
                        wasmtime::Val::I32(closure.function),
                        wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                            closure.captured_variables,
                        ))),
                    ],
                    &mut values,
                )
                .expect("call failed");

            /*match kind {
                fastn_runtime::PointerKind::Boolean => {
                    *cached_value = store.memory.get_boolean(values.ptr(0));
                }
                fastn_runtime::PointerKind::Integer => {
                    *cached_value = store.memory.get_i32(values.ptr(0));
                }
                fastn_runtime::PointerKind::Record => {
                    *cached_value = store.memory.get_vec(values.ptr(0));
                }
                fastn_runtime::PointerKind::Decimal => {
                    *cached_value = store.memory.get_f32(values.ptr(0));
                }
                fastn_runtime::PointerKind::List => {
                    *cached_value = store.memory.get_vec(values.ptr(0));
                }
                fastn_runtime::PointerKind::String => {
                    *cached_value = store.memory.get_string(values.ptr(0));
                }
                fastn_runtime::PointerKind::OrType => todo!(),
            }*/
        }*/
    }

    fn cursor_moved(&self, _pos_x: f64, _pos_y: f64) {
        // let _nodes = self.nodes_under_mouse(self.root, pos_x, pos_y);
        // todo!()
    }

    // initial_html() -> server side HTML
    pub fn initial_html(&self) -> String {
        fastn_runtime::html::initial(self.store.data())
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
