#[derive(Debug, Default)]
pub struct Memory {
    stack: Vec<Frame>,

    boolean: slotmap::SlotMap<fastn_runtime::PointerKey, bool>,
    i32: slotmap::SlotMap<fastn_runtime::PointerKey, i32>,
    f32: slotmap::SlotMap<fastn_runtime::PointerKey, f32>,
    // vec can store both vecs, and structs
    vec: slotmap::SlotMap<fastn_runtime::PointerKey, Vec<Pointer>>,
    r#enum: slotmap::SlotMap<fastn_runtime::PointerKey, (u8, Vec<Pointer>)>,

    attachment: std::collections::HashMap<Pointer, Vec<SDep>>,
    ui_deps: std::collections::HashMap<fastn_runtime::NodeKey, Vec<Pointer>>,
}

#[derive(Debug)]
pub struct SDep {
    // this is the dom element we are directly or indirectly connected with
    element: fastn_runtime::NodeKey,
    // who told we us about this connection
    source: Pointer,
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct Pointer {
    key: fastn_runtime::PointerKey,
    kind: Kind,
}

#[derive(Debug, Default)]
pub struct Frame {
    pointers: Vec<Pointer>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum Kind {
    Boolean,
    BooleanVec,
    Integer,
    Record,
    OrType2,
    Decimal,
}

impl Memory {
    pub fn detach_dom(&mut self, dom: fastn_runtime::NodeKey) {
        for pointer in self.ui_deps.remove(&dom).unwrap_or_default() {
            self.drop_pointer(&pointer);
        }
    }

    pub fn attach_to_dom(
        &mut self,
        _dom: fastn_runtime::NodeKey,
        _ptr: Pointer,
    ) {
        // add a new dependency to ptr, and recursively add it to all its dependencies
        todo!()
    }

    pub fn attach(&mut self, parent: Pointer, child: Pointer) {
        let parent_attachments = self.attachment.get(&parent).or_default();
        let child_attachments = self.attachment.entry(child).or_default();
        for parent_attachment in parent_attachments.iter() {
            // if parent has not already given the attachment to the child, add it
            child_attachments.push(parent_attachment.clone());
        }
        // TODO: pass all attachments from parent to child
        self.drop_from_frame(&child.key);
    }

    fn insert_in_frame(&mut self, pointer: fastn_runtime::PointerKey, kind: Kind) {
        // using .unwrap() so we crash on a bug instead of silently ignoring it
        self.stack
            .last_mut()
            .unwrap()
            .pointers
            .push(Pointer { key: pointer, kind });
    }

    pub fn create_frame(&mut self) {
        self.stack.push(Frame::default());
    }

    fn drop_pointer(&mut self, _pointer: &Pointer) {
        todo!()
    }

    pub fn end_frame(&mut self) {
        // using .unwrap() so we crash on a bug instead of silently ignoring it
        for pointer in self.stack.pop().unwrap().pointers.iter() {
            self.drop_pointer(pointer);
        }
    }

    pub fn create_boolean(&mut self, value: bool) -> fastn_runtime::PointerKey {
        let pointer = self.boolean.insert(value);
        self.insert_in_frame(pointer, Kind::Boolean);
        pointer
    }

    pub fn create_rgba(&mut self, r: i32, g: i32, b: i32, a: f32) -> fastn_runtime::PointerKey {
        let r_pointer = self.i32.insert(r);
        let g_pointer = self.i32.insert(g);
        let b_pointer = self.i32.insert(b);
        let a_pointer = self.f32.insert(a);

        let vec = self.vec.insert(vec![
            Pointer {
                key: r_pointer,
                kind: Kind::Integer,
            },
            Pointer {
                key: g_pointer,
                kind: Kind::Integer,
            },
            Pointer {
                key: b_pointer,
                kind: Kind::Integer,
            },
            Pointer {
                key: a_pointer,
                kind: Kind::Decimal,
            },
        ]);

        self.insert_in_frame(vec, Kind::Record);
        vec
    }

    pub fn register(&self, linker: &mut wasmtime::Linker<fastn_runtime::Dom>) {
        use fastn_runtime::dom::Params;

        linker
            .func_new(
                "fastn",
                "create_boolean",
                wasmtime::FuncType::new(
                    [wasmtime::ValType::I32].iter().cloned(),
                    [wasmtime::ValType::ExternRef].iter().cloned(),
                ),
                |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                    // ExternRef is a reference-counted pointer to a host-defined object. We mut not
                    // deallocate it on Rust side unless it's .strong_count() is 0. Not sure how it
                    // affects us yet.
                    results[0] = wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                        caller.data_mut().store.create_boolean(params.boolean(0)),
                    )));

                    Ok(())
                },
            )
            .unwrap();

        linker
            .func_new(
                "fastn",
                "create_rgba",
                wasmtime::FuncType::new(
                    [
                        wasmtime::ValType::I32,
                        wasmtime::ValType::I32,
                        wasmtime::ValType::I32,
                        wasmtime::ValType::F32,
                    ]
                    .iter()
                    .cloned(),
                    [wasmtime::ValType::ExternRef].iter().cloned(),
                ),
                |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                    results[0] = wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                        caller.data_mut().store.create_rgba(
                            params.i32(0),
                            params.i32(1),
                            params.i32(2),
                            params.f32(3),
                        ),
                    )));
                    Ok(())
                },
            )
            .unwrap();

        linker
            .func_new(
                "fastn",
                "create_frame",
                wasmtime::FuncType::new([].iter().cloned(), [].iter().cloned()),
                |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, _results| {
                    caller.data_mut().store.create_frame();
                    Ok(())
                },
            )
            .unwrap();

        linker
            .func_new(
                "fastn",
                "end_frame",
                wasmtime::FuncType::new([].iter().cloned(), [].iter().cloned()),
                |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, _results| {
                    caller.data_mut().store.end_frame();
                    Ok(())
                },
            )
            .unwrap();

        linker
            .func_new(
                "fastn",
                "get_boolean",
                wasmtime::FuncType::new(
                    [wasmtime::ValType::ExternRef].iter().cloned(),
                    [wasmtime::ValType::I32].iter().cloned(),
                ),
                |caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                    let s = &caller.data().store;

                    results[0] = wasmtime::Val::I32(s.boolean[params.ptr(0)] as i32);
                    Ok(())
                },
            )
            .unwrap();
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        fastn_runtime::assert_import("create_boolean", "(param i32) (result externref)");
        fastn_runtime::assert_import("create_frame", "");
        fastn_runtime::assert_import("end_frame", "");
        fastn_runtime::assert_import("create_rgba", "(param i32 i32 i32 f32) (result externref)");
    }

    #[test]
    fn gc() {
        let mut m = super::Memory::default();
        println!("{:#?}", m);
        m.create_frame();
        m.create_boolean(true);
        m.end_frame();
        println!("{:#?}", m);
        // panic!("yo");
    }
}
