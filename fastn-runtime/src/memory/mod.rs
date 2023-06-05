mod heap;
mod helper;
mod pointer;
mod ui;

pub use heap::{Heap, HeapData, HeapValue};
pub use pointer::{ClosurePointer, Pointer, PointerKey, PointerKind};
pub use ui::{DynamicProperty, UIProperty};

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
///
/// if we have:
///
/// ```ftd
/// -- ftd.text: hello
/// ```
///
/// a string containing hello will be created, and then passed to Rust as text properties, and
/// original wasm value would get dropped.
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
    /// are stored in the order they are defined. We also closure captured variables here.
    pub vec: Heap<Vec<Pointer>>,
    pub string: Heap<String>,
    or_type: Heap<(u8, Vec<Pointer>)>,

    text_role: Heap<fastn_runtime::TextRole>,
    color_role: Heap<fastn_runtime::Color>,
    length_role: Heap<fastn_runtime::LengthRole>,

    closure: slotmap::SlotMap<fastn_runtime::ClosurePointer, Closure>,
    event_handler: std::collections::HashMap<fastn_runtime::DomEventKind, Vec<EventHandler>>,
    /// We need to store some global variables. For every top level variable defined in ftd files
    /// we create a global variable. Since all values are stored in `Memory`, the globals contain
    /// pointers.
    ///
    /// The number of type of global variable will depend on ftd files.
    ///
    /// Our first attempt was to use wasm global, create a wasm global for each
    /// `(global $main#x externref)` but this does not work. When declaring global like that we have
    /// to store a value in the global slot. Which is odd as `(local)` does not have this
    /// requirement.
    ///
    /// For now we are going with the `get_global(idx: i32) -> externref`,
    /// `set_global(idx: i32, value: externref)`, where each global will be identified by the
    /// index (`idx`).
    global: Vec<fastn_runtime::PointerKey>,
    // if we have:
    // -- ftd.text: hello
    //
    // a string containing hello will be created, and then passed to Rust as text properties, and
    // original wasm value would get dropped.
}

#[derive(Debug)]
pub struct EventHandler {
    node: fastn_runtime::NodeKey,
    closure: fastn_runtime::ClosurePointer,
}

#[derive(Debug)]
pub struct Closure {
    /// functions are defined in wasm, and this is the index in the function table.
    pub function: i32,
    /// function_data is the pointer to a vector that contains all the variables "captured" by this
    /// closure.
    pub captured_variables: Pointer,
    // in future we can this optimisation: Saves us from creating vectors unless needed. Most
    // closures have two pointers (if most had three we can create a v3).

    // pub v1: Pointer,
    // pub v2: Option<Pointer>,
    // pub rest: Option<Vec<Pointer>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Attachment {
    /// this is the dom element we are directly or indirectly connected with
    element: fastn_runtime::NodeKey,
    /// who told we us about this element
    source: Pointer,
}

#[derive(Debug, Default)]
pub struct Frame {
    pointers: Vec<Pointer>,
}

impl Memory {
    #[cfg(test)]
    #[track_caller]
    pub(crate) fn assert_empty(&self) {
        if !self.stack.is_empty() {
            panic!("stack is not empty");
        }
        if !self.boolean.is_empty() {
            panic!("boolean is not empty");
        }
        if !self.i32.is_empty() {
            panic!("i32 is not empty");
        }
        if !self.f32.is_empty() {
            panic!("f32 is not empty");
        }
        if !self.vec.is_empty() {
            panic!("vec is not empty");
        }
        if !self.or_type.is_empty() {
            panic!("or_type is not empty");
        }
        if !self.closure.is_empty() {
            panic!("closures is not empty");
        }
    }

    pub fn get_colors(&self, color_pointer: fastn_runtime::PointerKey) -> (i32, i32, i32, f32) {
        let vec_value = self
            .vec
            .get(color_pointer)
            .expect("Expected color vec")
            .value
            .value();
        let r_pointer = vec_value.get(0).expect("Expected r pointer");
        let r_value = self
            .i32
            .get(r_pointer.pointer)
            .expect("Expected r value")
            .value
            .value();

        let g_pointer = vec_value.get(1).expect("Expected g pointer");
        let g_value = self
            .i32
            .get(g_pointer.pointer)
            .expect("Expected g value")
            .value
            .value();

        let b_pointer = vec_value.get(2).expect("Expected b pointer");
        let b_value = self
            .i32
            .get(b_pointer.pointer)
            .expect("Expected b value")
            .value
            .value();

        let a_pointer = vec_value.get(3).expect("Expected a pointer");
        let a_value = self
            .f32
            .get(a_pointer.pointer)
            .expect("Expected a value")
            .value
            .value();

        (*r_value, *g_value, *b_value, *a_value)
    }

    fn insert_in_frame(&mut self, pointer: fastn_runtime::PointerKey, kind: PointerKind) {
        // using .unwrap() so we crash on a bug instead of silently ignoring it
        let frame = self.stack.last_mut().unwrap();
        let pointer = Pointer { pointer, kind };
        for p in frame.pointers.iter() {
            if p == &pointer {
                panic!();
            }
        }
        frame.pointers.push(pointer);
    }

    pub fn create_frame(&mut self) {
        self.stack.push(Frame::default());
    }

    fn drop_from_frame(&mut self, _pointer: &Pointer) {
        todo!()
    }

    pub fn add_dependent(&mut self, target: Pointer, dependent: Pointer) {
        let dependents = match target.kind {
            PointerKind::Integer => &mut self.i32.get_mut(target.pointer).unwrap().dependents,
            PointerKind::Boolean => &mut self.boolean.get_mut(target.pointer).unwrap().dependents,
            PointerKind::Decimal => &mut self.f32.get_mut(target.pointer).unwrap().dependents,
            PointerKind::String => &mut self.string.get_mut(target.pointer).unwrap().dependents,
            PointerKind::List | PointerKind::Record | PointerKind::OrType => {
                &mut self.vec.get_mut(target.pointer).unwrap().dependents
            }
        };

        println!(
            "add_dependent, target: {:?}, dependent: {:?}",
            target, &dependent
        );
        dependents.push(dependent);
    }

    fn drop_pointer(
        &mut self,
        pointer: Pointer,
        dropped_so_far: &mut Vec<fastn_runtime::Pointer>,
    ) -> bool {
        println!("dropping {:?}", pointer);
        if dropped_so_far.contains(&pointer) {
            println!("pointer already dropped, ignoring: {:?}", pointer);
            return true;
        }
        let (dependents, ui_properties) = match pointer.kind {
            PointerKind::Boolean => {
                let b = self.boolean.get(pointer.pointer).unwrap();
                (&b.dependents, &b.ui_properties)
            }
            PointerKind::Integer => {
                let b = self.i32.get(pointer.pointer).unwrap();
                (&b.dependents, &b.ui_properties)
            }
            PointerKind::Record | PointerKind::List => {
                let b = self.vec.get(pointer.pointer).unwrap();
                (&b.dependents, &b.ui_properties)
            }
            PointerKind::OrType => {
                let b = self.or_type.get(pointer.pointer).unwrap();
                (&b.dependents, &b.ui_properties)
            }
            PointerKind::Decimal => {
                let b = self.f32.get(pointer.pointer).unwrap();
                (&b.dependents, &b.ui_properties)
            }
            PointerKind::String => {
                let b = self.string.get(pointer.pointer).unwrap();
                (&b.dependents, &b.ui_properties)
            }
        };

        if !ui_properties.is_empty() {
            return false;
        }

        let mut drop = true;
        for d in dependents.clone() {
            if !self.drop_pointer(d, dropped_so_far) {
                drop = false;
                break;
            }
        }

        if drop {
            dropped_so_far.push(pointer);
            self.delete_pointer(pointer);
        }

        drop
    }

    fn delete_pointer(&mut self, pointer: Pointer) {
        match pointer.kind {
            PointerKind::Boolean => {
                self.boolean.remove(pointer.pointer);
            }
            PointerKind::Integer => {
                self.i32.remove(pointer.pointer);
            }
            PointerKind::Record | PointerKind::List => {
                self.vec.remove(pointer.pointer);
            }
            PointerKind::OrType => {
                self.or_type.remove(pointer.pointer);
            }
            PointerKind::Decimal => {
                self.f32.remove(pointer.pointer);
            }
            PointerKind::String => {
                self.string.remove(pointer.pointer);
            }
        };
    }

    pub fn end_frame(&mut self) {
        // using .unwrap() so we crash on a bug instead of silently ignoring it
        println!("end_frame called");
        let mut v = vec![];
        for pointer in self.stack.pop().unwrap().pointers.iter() {
            self.drop_pointer(*pointer, &mut v);
        }
        println!("end_frame ended");
    }

    pub fn return_frame(&mut self, keep: fastn_runtime::PointerKey) -> fastn_runtime::PointerKey {
        let mut k: Option<fastn_runtime::Pointer> = None;
        let mut v = vec![];

        for pointer in self.stack.pop().unwrap().pointers.iter() {
            if pointer.pointer == keep {
                k = Some(pointer.to_owned());
            } else {
                self.drop_pointer(*pointer, &mut v);
            }
        }

        let k = k.unwrap();
        self.insert_in_frame(k.pointer, k.kind);
        keep
    }

    pub fn get_global(&self, idx: i32) -> fastn_runtime::PointerKey {
        self.global[idx as usize]
    }

    pub fn set_global(&mut self, idx: i32, ptr: fastn_runtime::PointerKey) {
        let idx = idx as usize;

        if idx < self.global.len() {
            println!("updated global: idx={}, ptr={:?}", idx, ptr);
            self.global[idx] = ptr;
            return;
        }

        if idx == self.global.len() {
            println!("created global: idx={}, ptr={:?}", idx, ptr);
            self.global.push(ptr);
            return;
        }

        // the way things are either this global variables are sequentially initialised at the start
        // of the program. If a jump happens it means our generated wasm file is incorrect.
        unreachable!()
    }

    pub(crate) fn create_closure(&mut self, closure: Closure) -> fastn_runtime::ClosurePointer {
        let ptr = self.closure.insert(closure);
        println!("{:?}", ptr);
        ptr
    }

    pub fn attach_event_handler(
        &mut self,
        node: fastn_runtime::NodeKey,
        event_kind: fastn_runtime::DomEventKind,
        table_index: i32,
        func_arg: fastn_runtime::PointerKey,
    ) {
        let eh = fastn_runtime::EventHandler {
            node,
            closure: self.create_closure(fastn_runtime::Closure {
                function: table_index,
                captured_variables: func_arg.into_list_pointer(),
            }),
        };
        match self.event_handler.get_mut(&event_kind) {
            Some(v) => v.push(eh),
            None => {
                self.event_handler.insert(event_kind, vec![eh]);
            }
        }
    }

    pub fn is_pointer_valid(&self, ptr: fastn_runtime::Pointer) -> bool {
        match ptr.kind {
            PointerKind::Boolean => self.boolean.contains_key(ptr.pointer),
            PointerKind::Integer => self.i32.contains_key(ptr.pointer),
            PointerKind::Record => self.vec.contains_key(ptr.pointer),
            PointerKind::OrType => self.or_type.contains_key(ptr.pointer),
            PointerKind::Decimal => self.f32.contains_key(ptr.pointer),
            PointerKind::List => self.vec.contains_key(ptr.pointer),
            PointerKind::String => self.string.contains_key(ptr.pointer),
        }
    }

    pub fn create_string_constant(&mut self, buffer: Vec<u8>) -> fastn_runtime::PointerKey {
        let s = String::from_utf8(buffer).unwrap();
        let pointer = self.string.insert(HeapValue::new(s).into_heap_data());
        // Note: intentionally not adding to the frame as constant strings are not to be GCed
        // self.insert_in_frame(pointer, PointerKind::String);
        println!("{:?}", pointer);
        pointer
    }

    pub fn create_list(&mut self) -> fastn_runtime::PointerKey {
        let pointer = self.vec.insert(HeapValue::new(vec![]).into_heap_data());
        self.insert_in_frame(pointer, PointerKind::List);
        println!("{:?}", pointer);
        pointer
    }

    pub fn create_list_1(
        &mut self,
        v1_kind: fastn_runtime::PointerKind,
        v1_ptr: fastn_runtime::PointerKey,
    ) -> fastn_runtime::PointerKey {
        let ptr1 = fastn_runtime::Pointer {
            pointer: v1_ptr,
            kind: v1_kind,
        };

        let pointer = self.vec.insert(HeapValue::new(vec![ptr1]).into_heap_data());

        let list_pointer = pointer.into_list_pointer();
        self.add_dependent(ptr1, list_pointer);

        self.insert_in_frame(pointer, PointerKind::List);
        pointer
    }

    pub fn create_list_2(
        &mut self,
        v1_kind: fastn_runtime::PointerKind,
        v1_ptr: fastn_runtime::PointerKey,
        v2_kind: fastn_runtime::PointerKind,
        v2_ptr: fastn_runtime::PointerKey,
    ) -> fastn_runtime::PointerKey {
        let ptr1 = fastn_runtime::Pointer {
            pointer: v1_ptr,
            kind: v1_kind,
        };
        let ptr2 = fastn_runtime::Pointer {
            pointer: v2_ptr,
            kind: v2_kind,
        };

        let pointer = self
            .vec
            .insert(HeapValue::new(vec![ptr1, ptr2]).into_heap_data());

        let list_pointer = pointer.into_list_pointer();
        self.add_dependent(ptr1, list_pointer);
        self.add_dependent(ptr2, list_pointer);

        self.insert_in_frame(pointer, PointerKind::List);
        println!("{:?}", pointer);
        pointer
    }

    pub fn create_boolean(&mut self, value: bool) -> fastn_runtime::PointerKey {
        let pointer = self.boolean.insert(HeapValue::new(value).into_heap_data());
        self.insert_in_frame(pointer, PointerKind::Boolean);
        println!("{:?}", pointer);
        pointer
    }

    pub fn get_boolean(&self, ptr: fastn_runtime::PointerKey) -> bool {
        *self.boolean[ptr].value.value()
    }

    pub fn set_boolean(&mut self, ptr: fastn_runtime::PointerKey, value: bool) {
        self.boolean[ptr].value.set_value(value)
    }

    pub fn create_i32(&mut self, value: i32) -> fastn_runtime::PointerKey {
        let pointer = self.i32.insert(HeapValue::new(value).into_heap_data());
        self.insert_in_frame(pointer, PointerKind::Integer);
        println!("{:?}", pointer);
        pointer
    }

    pub fn get_i32(&self, ptr: fastn_runtime::PointerKey) -> i32 {
        *self.i32[ptr].value.value()
    }

    pub fn set_i32(&mut self, ptr: fastn_runtime::PointerKey, value: i32) {
        self.i32[ptr].value.set_value(value)
    }

    pub fn multiply_i32(
        &mut self,
        arr: fastn_runtime::PointerKey,
        idx_1: i32,
        idx_2: i32,
    ) -> fastn_runtime::PointerKey {
        let idx_1 = idx_1 as usize;
        let idx_2 = idx_2 as usize;

        let arr = self.vec[arr].value.mut_value();

        let v1 = *self.i32[arr[idx_1].pointer].value.value();
        let v2 = *self.i32[arr[idx_2].pointer].value.value();

        self.create_i32(v1 * v2)
    }

    pub fn create_f32(&mut self, value: f32) -> fastn_runtime::PointerKey {
        let pointer = self.f32.insert(HeapValue::new(value).into_heap_data());
        self.insert_in_frame(pointer, PointerKind::Integer);
        println!("{:?}", pointer);
        pointer
    }

    pub fn get_f32(&self, ptr: fastn_runtime::PointerKey) -> f32 {
        *self.f32[ptr].value.value()
    }

    pub fn set_f32(&mut self, ptr: fastn_runtime::PointerKey, value: f32) {
        self.f32[ptr].value.set_value(value)
    }

    pub fn create_i32_func(
        &mut self,
        cached_value: i32,
        closure: Closure,
    ) -> fastn_runtime::PointerKey {
        let closure_key = self.create_closure(closure);
        let pointer = self
            .i32
            .insert(HeapValue::new_with_formula(cached_value, closure_key).into_heap_data());
        self.insert_in_frame(pointer, PointerKind::Integer);
        println!("{:?}", pointer);
        pointer
    }

    pub fn get_func_arg_i32(&self, ptr: fastn_runtime::PointerKey, idx: i32) -> i32 {
        let ptr = self
            .vec
            .get(ptr)
            .unwrap()
            .value
            .value()
            .get(idx as usize)
            .unwrap();
        *self.i32.get(ptr.pointer).unwrap().value.value()
    }

    pub fn array_i32_2(
        &mut self,
        ptr1: fastn_runtime::PointerKey,
        ptr2: fastn_runtime::PointerKey,
    ) -> fastn_runtime::PointerKey {
        let vec = self.vec.insert(
            HeapValue::new(vec![
                Pointer {
                    pointer: ptr1,
                    kind: PointerKind::Integer,
                },
                Pointer {
                    pointer: ptr2,
                    kind: PointerKind::Integer,
                },
            ])
            .into_heap_data(),
        );
        self.add_dependent(ptr1.into_integer_pointer(), vec.into_list_pointer());
        self.add_dependent(ptr2.into_integer_pointer(), vec.into_list_pointer());

        self.insert_in_frame(vec, PointerKind::List);
        println!("{:?}", vec);

        vec
    }

    pub fn add_dynamic_property_dependency(
        &mut self,
        target: Pointer,
        dependency: DynamicProperty,
    ) {
        let dependents = match target.kind {
            PointerKind::Integer => &mut self.i32.get_mut(target.pointer).unwrap().ui_properties,
            PointerKind::String => &mut self.string.get_mut(target.pointer).unwrap().ui_properties,
            PointerKind::Boolean => {
                &mut self.boolean.get_mut(target.pointer).unwrap().ui_properties
            }
            PointerKind::Decimal => &mut self.f32.get_mut(target.pointer).unwrap().ui_properties,
            PointerKind::List | PointerKind::Record | PointerKind::OrType => {
                &mut self.vec.get_mut(target.pointer).unwrap().ui_properties
            }
        };

        dependents.push(dependency);
    }

    pub fn create_rgba(&mut self, r: i32, g: i32, b: i32, a: f32) -> fastn_runtime::PointerKey {
        let r_pointer = self.create_i32(r);
        let g_pointer = self.i32.insert(HeapValue::new(g).into_heap_data());
        let b_pointer = self.i32.insert(HeapValue::new(b).into_heap_data());
        let a_pointer = self.f32.insert(HeapValue::new(a).into_heap_data());

        let vec = self.vec.insert(
            HeapValue::new(vec![
                Pointer {
                    pointer: r_pointer,
                    kind: PointerKind::Integer,
                },
                Pointer {
                    pointer: g_pointer,
                    kind: PointerKind::Integer,
                },
                Pointer {
                    pointer: b_pointer,
                    kind: PointerKind::Integer,
                },
                Pointer {
                    pointer: a_pointer,
                    kind: PointerKind::Decimal,
                },
            ])
            .into_heap_data(),
        );

        self.add_dependent(r_pointer.into_integer_pointer(), vec.into_record_pointer());
        self.add_dependent(g_pointer.into_integer_pointer(), vec.into_record_pointer());
        self.add_dependent(b_pointer.into_integer_pointer(), vec.into_record_pointer());
        self.add_dependent(a_pointer.into_integer_pointer(), vec.into_record_pointer());

        self.insert_in_frame(vec, PointerKind::Record);
        println!("{:?}", vec);
        vec
    }

    pub(crate) fn handle_event(
        &mut self,
        event_kind: fastn_runtime::DomEventKind,
        node: Option<fastn_runtime::NodeKey>,
    ) {
        if let Some(events) = self.get_event_handlers(event_kind, node) {
            for event in events {
                let _closure = self.closure.get(event.closure).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn create_get_and_set() {
        let mut m = super::Memory::default();
        println!("{:#?}", m);
        m.assert_empty();
        m.create_frame();

        let p = m.create_boolean(true);
        assert!(m.get_boolean(p));

        m.set_boolean(p, false);
        assert!(!m.get_boolean(p));

        let p = m.create_boolean(false);
        assert!(!m.get_boolean(p));

        let p = m.create_i32(20);
        assert_eq!(m.get_i32(p), 20);

        m.set_i32(p, 30);
        assert_eq!(m.get_i32(p), 30);

        println!("{:#?}", m);
        m.end_frame();
        m.assert_empty();
        println!("{:#?}", m);
    }

    #[test]
    fn stack() {
        let mut m = super::Memory::default();
        println!("{:#?}", m);
        m.assert_empty();

        {
            m.create_frame();

            let p = m.create_boolean(true).into_boolean_pointer();
            assert!(m.get_boolean(p.pointer));

            {
                m.create_frame();
                assert!(m.get_boolean(p.pointer));

                let p2 = m.create_boolean(false).into_boolean_pointer();
                assert!(!m.get_boolean(p2.pointer));

                m.end_frame();
                assert!(m.is_pointer_valid(p));
                assert!(!m.is_pointer_valid(p2));
            }

            assert!(m.get_boolean(p.pointer));
            m.end_frame();
            assert!(!m.is_pointer_valid(p));
        }

        m.assert_empty();
    }

    #[test]
    #[should_panic]
    fn cleaned_up_pointer_access_should_panic() {
        let mut m = super::Memory::default();

        m.create_frame();

        let p = m.create_boolean(true).into_boolean_pointer();
        assert!(m.get_boolean(p.pointer));

        m.end_frame();
        m.get_boolean(p.pointer);
    }

    #[test]
    fn return_frame() {
        let mut m = super::Memory::default();
        println!("{:#?}", m);
        m.assert_empty();

        {
            m.create_frame();

            let p = m.create_boolean(true).into_boolean_pointer();
            assert!(m.get_boolean(p.pointer));

            let p2 = {
                m.create_frame();
                assert!(m.get_boolean(p.pointer));

                let p2 = m.create_boolean(false).into_boolean_pointer();
                assert!(!m.get_boolean(p2.pointer));

                m.return_frame(p2.pointer);

                assert!(m.is_pointer_valid(p));
                assert!(m.is_pointer_valid(p2));

                p2
            };

            assert!(m.get_boolean(p.pointer));
            assert!(!m.get_boolean(p2.pointer));

            m.end_frame();
            assert!(!m.is_pointer_valid(p));
            assert!(!m.is_pointer_valid(p2));
        }

        m.assert_empty();
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
