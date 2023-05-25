/// Memory contains all the data created by our runtime.
///
/// When say a boolean is created in ftd world, we add an entry in the `.boolean` here, and return
/// the "pointer" to this to wasm world as `externref` type. Similarly we have `.i32`, and `.f32`.
///
/// Currently we store all integers (`i8`, `u8` etc) as `i32` and all floats as `f32`. These are
/// the types in wasm and we are designed to be used with wasm only.
///
/// For vectors and structs, we use a memory sub-optimal solution of storing each data as a vector,
/// so a vector containing two booleans will be a vector containing two pointers pointing to each
/// boolean, instead of storing the booleans themselves.
///
/// we store enums in `.or_type`. The `u8` is for storing the variant of the enum that this
/// value represents. The data for the variant is stored in the Vec.
///
/// We maintain stack of function calls in a `.stack`. We do not store any data on stack, the
/// purpose of stack is to assist in garbage collection. When a value is created it's pointer is
/// stored on the top frame of the stack. When we attach any value to dom using `.attach_to_dom()`
/// we remove the pointer and all the descendants of the pointer from the frame they were created
/// in. This was at the end of the frame, whatever is left is safe to de-allocate.
///
/// The real magic happens when `.attach_to_dom()` is called on any pointer. We call this the
/// "pointer getting attached to the UI". Any pointer that is not attached to UI gets de-allocated
/// at first opportunity.
///
/// When a pointer is created, we also create a `Vec<Attachment>`, and store it next to it. So if
/// a boolean is created we create a store both the boolean and `Vec<Attachment>` for that boolean
/// in the `.boolean`. We have a type `PointerData<T>` which keeps track of the value and the
/// attachments.
///
/// When `.attach_to_dom()` is called, we find all the dependencies.
#[derive(Debug, Default)]
pub struct Memory {
    /// when a function starts in wasm side, a new `Frame` is created and added here. Each new
    /// pointer we create, we add it to the `Frame`. When a new pointer is created, it is
    /// considered "owned" by the `Frame`. Once we attach to dom node using `Memory.attach_to_dom()`,
    /// we remove the link to pointer from the frame. This way at the end of the frame we see if
    /// anything is still attached to the frame, and which means that pointer is not attached to
    /// anything else, we clear it up cleanly.
    stack: Vec<Frame>,

    boolean: Heap<bool>,
    i32: Heap<i32>,
    f32: Heap<f32>,
    /// `.vec` can store both `vec`s, `tuple`s, and `struct`s using these. For struct the fields
    /// are stored in the order they are defined.
    vec: Heap<Vec<KindPointer>>,
    or_type: Heap<(u8, Vec<KindPointer>)>,

    closures: slotmap::SlotMap<fastn_runtime::ClosureKey, Closure>,

    /// if we have:
    /// -- ftd.text: hello
    ///
    /// a string containing hello will be created, and then passed to Rust as text properties, and
    /// original wasm value would get dropped.
}

type Heap<T> = slotmap::SlotMap<fastn_runtime::PointerKey, HeapData<T>>;

/// For every ftd value we have one such entry
struct HeapData<T> {
    /// The inner value being stored in ftd
    value: HeapValue<T>,
    /// the list of values that depend on this, eg if we add x to a list l, we also do a
    /// x.dependents.add(l)
    dependents: Vec<KindPointer>,
    /// whenever a dom node is added or deleted, it is added or removed from this list.
    ui_properties: Vec<UIDependendent>,
}

/// This is the data we store in the heap for any value.
enum HeapValue<T> {
    Value(T),

    /// If a value is defined in terms of a function, we store the last computed value and the
    /// closure. We cached the last computed value so if the data is not changing we do not have
    /// to re-compute the closure.
    ///
    /// -- integer x: 10 (stored as HeapValue::Value(10))
    /// -- integer y: 20 (stored as HeapValue::Value(10))
    /// -- integer z = { x + y } (stored as HeapValue::Formula { cached_value: 30, closure: 1v2 }
    Formula { cached_value: T, closure: fastn_runtime::ClosureKey },
}

#[derive(Debug)]
struct UIDependent {
    property: UIProperty,
    node: fastn_runtime::NodeKey,
    closure: Option<fastn_runtime::ClosureKey>,
}

#[derive(Debug)]
struct Closure {
    /// functions are defined in wasm, and this is the index in the function table.
    function: i32, // entry in the function table
    /// function_data is the data passed to the function.
    function_data: KindPointer,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Attachment {
    /// this is the dom element we are directly or indirectly connected with
    element: fastn_runtime::NodeKey,
    /// who told we us about this element
    source: KindPointer,
}

/// Since a pointer can be present in any of the slotmaps on Memory, .boolean, .i32 etc, we need
/// to keep track of Kind so we know where this pointer came from
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KindPointer {
    key: fastn_runtime::PointerKey,
    kind: Kind,
}

#[derive(Debug, Default)]
pub struct Frame {
    pointers: Vec<KindPointer>,
}

#[derive(Debug)]
enum UIProperty {
    WidthPx,
    WidthPercent,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum Kind {
    Boolean,
    Integer,
    Record,
    OrType,
    Decimal,
}

impl Memory {
    pub fn detach_dom(&mut self, dom: fastn_runtime::NodeKey) {
        for pointer in self.ui_deps.remove(&dom).unwrap_or_default() {
            self.drop_pointer(&pointer);
        }
    }

    pub fn attach_to_dom(&mut self, _dom: fastn_runtime::NodeKey, _ptr: KindPointer) {
        // add a new dependency to ptr, and recursively add it to all its dependencies
        todo!()
    }

    fn get_pointer_dep_children(&self, pointer: &KindPointer) -> Option<Vec<KindPointer>> {
        match &pointer.kind {
            Kind::Boolean => self.boolean.get(pointer.key).map(|v| v.1.clone()),
            Kind::Integer => self.boolean.get(pointer.key).map(|v| v.1.clone()),
            Kind::Record => self
                .vec
                .get(pointer.key)
                .map(|v| [v.0.clone(), v.1.clone()].concat().into_iter().collect()),
            Kind::OrType => self.or_type.get(pointer.key).map(|v| v.1.clone()),
            Kind::Decimal => self.f32.get(pointer.key).map(|v| v.1.clone()),
        }
    }

    fn add_dep_child(&mut self, pointer: &KindPointer, child: KindPointer) {
        if let Some(dep_children) = match &pointer.kind {
            Kind::Boolean => self.boolean.get_mut(pointer.key).map(|v| &mut v.1),
            Kind::Integer => self.boolean.get_mut(pointer.key).map(|v| &mut v.1),
            Kind::Record => self.vec.get_mut(pointer.key).map(|v| &mut v.1),
            Kind::OrType => self.or_type.get_mut(pointer.key).map(|v| &mut v.1),
            Kind::Decimal => self.f32.get_mut(pointer.key).map(|v| &mut v.1),
        } {
            dep_children.push(child);
        }
    }

    pub fn attach(&mut self, parent: KindPointer, child: KindPointer) {
        let parent_attachments = if let Some(attachment) = self.attachment.get(&parent) {
            attachment.clone()
        } else {
            return;
        };
        let mut child_attachments = self.attachment.entry(child.clone()).or_default().clone();
        for parent_attachment in parent_attachments {
            // if parent has not already given the attachment to the child, add it
            let attachment = Attachment {
                element: parent_attachment.element,
                source: parent.clone(),
            };
            let is_attached = child_attachments.insert(attachment);
            if is_attached {
                let dep_children = self.get_pointer_dep_children(&child).unwrap();
                for dep in dep_children {
                    self.attach(child.clone(), dep)
                }
            }
        }

        *self.attachment.get_mut(&child).unwrap() = child_attachments;
        self.add_dep_child(&parent, child.clone());
        // TODO: pass all attachments from parent to child
        self.drop_from_frame(&child);
    }

    fn insert_in_frame(&mut self, pointer: fastn_runtime::PointerKey, kind: Kind) {
        // using .unwrap() so we crash on a bug instead of silently ignoring it
        self.stack
            .last_mut()
            .unwrap()
            .pointers
            .push(KindPointer { key: pointer, kind });
    }

    pub fn create_frame(&mut self) {
        self.stack.push(Frame::default());
    }

    fn drop_from_frame(&mut self, _pointer: &KindPointer) {
        todo!()
    }

    fn drop_pointer(&mut self, _pointer: &KindPointer) {
        todo!()
    }

    pub fn end_frame(&mut self) {
        // using .unwrap() so we crash on a bug instead of silently ignoring it
        for pointer in self.stack.pop().unwrap().pointers.iter() {
            self.drop_pointer(pointer);
        }
    }

    pub fn create_boolean(&mut self, value: bool) -> fastn_runtime::PointerKey {
        let pointer = self.boolean.insert((value, vec![]));
        self.insert_in_frame(pointer, Kind::Boolean);
        pointer
    }

    pub fn create_i32(&mut self, value: i32) -> fastn_runtime::PointerKey {
        let pointer = self.i32.insert((value, vec![]));
        self.insert_in_frame(pointer, Kind::Integer);
        pointer
    }

    pub fn create_rgba(&mut self, r: i32, g: i32, b: i32, a: f32) -> fastn_runtime::PointerKey {
        let r_pointer = self.i32.insert((r, vec![]));
        let g_pointer = self.i32.insert((g, vec![]));
        let b_pointer = self.i32.insert((b, vec![]));
        let a_pointer = self.f32.insert((a, vec![]));

        let vec = self.vec.insert((
            vec![
                KindPointer {
                    key: r_pointer,
                    kind: Kind::Integer,
                },
                KindPointer {
                    key: g_pointer,
                    kind: Kind::Integer,
                },
                KindPointer {
                    key: b_pointer,
                    kind: Kind::Integer,
                },
                KindPointer {
                    key: a_pointer,
                    kind: Kind::Decimal,
                },
            ],
            vec![],
        ));

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
                "create_i32",
                wasmtime::FuncType::new(
                    [wasmtime::ValType::I32].iter().cloned(),
                    [wasmtime::ValType::ExternRef].iter().cloned(),
                ),
                |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                    results[0] = wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                        caller.data_mut().store.create_i32(params.i32(0)),
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

                    results[0] = wasmtime::Val::I32(s.boolean[params.ptr(0)].0 as i32);
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


// -- record x:
// y list y:
//
// -- record y:
// string z:
//
// -- x $x:
// -- x.y:
// z: hello

// -- foo: $x.y
// -- ftd.text: $x.y.z

// -- ftd.text: yo
// $on-click$: $x = new_x(x, "bye")
// $on-click$: $x.y = new_y("bye")


// -- l: $o
// $loop$: $x.y



// x.y.z = "hello"
// x.y.z changed





// (attach_dom (create_l) $x [0, 0])


// (attach_dom (create_l) $x [0, 0])


// x.y.insert_at(0, new_y)

// (attach_dom (create_text) $x [0, 0])



// -- foo:
// person: $person


// -- foo:
// $person: $person



// -- show-student: $student
// $loop$: $students as $student
// rank: calculate_rank($students, idx)


// -- ftd.text:
// $on-click$: $x = new_x(x, "bye")
// $on-click$: $x.y = new_y("bye")
//
// x new_x(v):
// string v:
//
// {
//    y: {
//        z: v
//    }
// }



