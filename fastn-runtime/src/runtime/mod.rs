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
/// -- ftd.text: hello
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
    /// are stored in the order they are defined.
    vec: Heap<Vec<KindPointer>>,
    or_type: Heap<(u8, Vec<KindPointer>)>,

    closures: slotmap::SlotMap<fastn_runtime::ClosureKey, Closure>,
    // if we have:
    // -- ftd.text: hello
    //
    // a string containing hello will be created, and then passed to Rust as text properties, and
    // original wasm value would get dropped.
}

type Heap<T> = slotmap::SlotMap<fastn_runtime::PointerKey, HeapData<T>>;

/// For every ftd value we have one such entry
#[derive(Debug)]
struct HeapData<T> {
    /// The inner value being stored in ftd
    value: HeapValue<T>,
    /// the list of values that depend on this, eg if we add x to a list l, we also do a
    /// x.dependents.add(l)
    dependents: Vec<KindPointer>,
    /// whenever a dom node is added or deleted, it is added or removed from this list.
    ui_properties: Vec<UIDependent>,
}

/// This is the data we store in the heap for any value.
#[derive(Debug, Eq, PartialEq)]
enum HeapValue<T> {
    Value(T),

    /// If a value is defined in terms of a function, we store the last computed value and the
    /// closure. We cached the last computed value so if the data is not changing we do not have
    /// to re-compute the closure.
    ///
    /// -- integer x: 10 (stored as HeapValue::Value(10))
    /// -- integer y: 20 (stored as HeapValue::Value(10))
    /// -- integer z = { x + y } (stored as HeapValue::Formula { cached_value: 30, closure: 1v2 }
    Formula {
        cached_value: T,
        closure: fastn_runtime::ClosureKey,
    },
}

impl<T> HeapValue<T> {
    pub(crate) fn value(&self) -> &T {
        match self {
            HeapValue::Value(v) => v,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub struct UIDependent {
    property: fastn_runtime::UIProperty,
    node: fastn_runtime::NodeKey,
    closure: Option<fastn_runtime::ClosureKey>,
}

impl UIDependent {
    pub(crate) fn closure(self, closure: fastn_runtime::ClosureKey) -> Self {
        let mut ui_dependent = self;
        ui_dependent.closure = Some(closure);
        ui_dependent
    }
}

#[derive(Debug)]
pub struct Closure {
    /// functions are defined in wasm, and this is the index in the function table.
    pub function: i32, // entry in the function table
    /// function_data is the data passed to the function.
    pub function_data: KindPointer,
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
    pub key: fastn_runtime::PointerKey,
    pub kind: Kind,
}

impl fastn_runtime::PointerKey {
    pub(crate) fn into_integer_pointer(self) -> KindPointer {
        KindPointer {
            key: self,
            kind: Kind::Integer,
        }
    }

    pub(crate) fn into_decimal_pointer(self) -> KindPointer {
        KindPointer {
            key: self,
            kind: Kind::Decimal,
        }
    }

    pub(crate) fn into_list_pointer(self) -> KindPointer {
        KindPointer {
            key: self,
            kind: Kind::List,
        }
    }

    pub(crate) fn into_record_pointer(self) -> KindPointer {
        KindPointer {
            key: self,
            kind: Kind::Record,
        }
    }
}

#[derive(Debug, Default)]
pub struct Frame {
    pointers: Vec<KindPointer>,
}

#[derive(Debug, Copy, Clone)]
pub enum UIProperty {
    WidthFixedPx,
    HeightFixedPx,
    HeightFixedPercentage,
    BackgroundSolid,
}

impl From<i32> for UIProperty {
    fn from(i: i32) -> UIProperty {
        match i {
            0 => UIProperty::WidthFixedPx,
            1 => UIProperty::HeightFixedPx,
            2 => UIProperty::HeightFixedPercentage,
            3 => UIProperty::BackgroundSolid,
            _ => panic!("Unknown UIProperty: {}", i),
        }
    }
}

impl From<UIProperty> for i32 {
    fn from(v: UIProperty) -> i32 {
        match v {
            UIProperty::WidthFixedPx => 0,
            UIProperty::HeightFixedPx => 1,
            UIProperty::HeightFixedPercentage => 2,
            UIProperty::BackgroundSolid => 3,
        }
    }
}

impl UIProperty {
    pub(crate) fn into_ui_dependent(self, node: fastn_runtime::NodeKey) -> UIDependent {
        UIDependent {
            property: self,
            node,
            closure: None,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Kind {
    Boolean,
    Integer,
    Record,
    OrType,
    Decimal,
    List,
}

impl<T> HeapData<T> {
    pub(crate) fn new(value: HeapValue<T>) -> HeapData<T> {
        HeapData {
            value,
            dependents: vec![],
            ui_properties: vec![],
        }
    }
}

impl<T> HeapValue<T> {
    pub(crate) fn new(value: T) -> HeapValue<T> {
        HeapValue::Value(value)
    }

    pub(crate) fn new_with_formula(
        cached_value: T,
        closure: fastn_runtime::ClosureKey,
    ) -> HeapValue<T> {
        HeapValue::Formula {
            cached_value,
            closure,
        }
    }

    pub(crate) fn into_heap_data(self) -> HeapData<T> {
        HeapData::new(self)
    }
}

impl Memory {
    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.stack.is_empty()
            && self.boolean.is_empty()
            && self.i32.is_empty()
            && self.f32.is_empty()
            && self.vec.is_empty()
            && self.or_type.is_empty()
            && self.closures.is_empty()
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
            .get(r_pointer.key)
            .expect("Expected r value")
            .value
            .value();

        let g_pointer = vec_value.get(0).expect("Expected g pointer");
        let g_value = self
            .i32
            .get(g_pointer.key)
            .expect("Expected g value")
            .value
            .value();

        let b_pointer = vec_value.get(0).expect("Expected b pointer");
        let b_value = self
            .i32
            .get(b_pointer.key)
            .expect("Expected b value")
            .value
            .value();

        let a_pointer = vec_value.get(0).expect("Expected a pointer");
        let a_value = self
            .f32
            .get(a_pointer.key)
            .expect("Expected a value")
            .value
            .value();

        (*r_value, *g_value, *b_value, *a_value)
    }

    pub fn detach_dom(&mut self, _dom: fastn_runtime::NodeKey) {
        // for pointer in self.ui_deps.remove(&dom).unwrap_or_default() {
        //     self.drop_pointer(&pointer);
        // }
    }

    pub fn attach_to_dom(&mut self, _dom: fastn_runtime::NodeKey, _ptr: KindPointer) {
        // add a new dependency to ptr, and recursively add it to all its dependencies
        todo!()
    }

    fn get_pointer_dep_children(&self, _pointer: &KindPointer) -> Option<Vec<KindPointer>> {
        // match &pointer.kind {
        //     Kind::Boolean => self.boolean.get(pointer.key).map(|v| v.value.clone()),
        //     Kind::Integer => self.boolean.get(pointer.key).map(|v| v.1.clone()),
        //     Kind::Record => self
        //         .vec
        //         .get(pointer.key)
        //         .map(|v| [v.0.clone(), v.1.clone()].concat().into_iter().collect()),
        //     Kind::OrType => self.or_type.get(pointer.key).map(|v| v.1.clone()),
        //     Kind::Decimal => self.f32.get(pointer.key).map(|v| v.1.clone()),
        // }
        todo!()
    }

    fn add_dep_child(&mut self, _pointer: &KindPointer, _child: KindPointer) {
        // if let Some(dep_children) = match &pointer.kind {
        //     Kind::Boolean => self.boolean.get_mut(pointer.key).map(|v| &mut v.1),
        //     Kind::Integer => self.boolean.get_mut(pointer.key).map(|v| &mut v.1),
        //     Kind::Record => self.vec.get_mut(pointer.key).map(|v| &mut v.1),
        //     Kind::OrType => self.or_type.get_mut(pointer.key).map(|v| &mut v.1),
        //     Kind::Decimal => self.f32.get_mut(pointer.key).map(|v| &mut v.1),
        // } {
        //     dep_children.push(child);
        // }
        todo!()
    }

    pub fn attach(&mut self, _parent: KindPointer, _child: KindPointer) {
        // let parent_attachments = if let Some(attachment) = self.attachment.get(&parent) {
        //     attachment.clone()
        // } else {
        //     return;
        // };
        // let mut child_attachments = self.attachment.entry(child.clone()).or_default().clone();
        // for parent_attachment in parent_attachments {
        //     // if parent has not already given the attachment to the child, add it
        //     let attachment = Attachment {
        //         element: parent_attachment.element,
        //         source: parent.clone(),
        //     };
        //     let is_attached = child_attachments.insert(attachment);
        //     if is_attached {
        //         let dep_children = self.get_pointer_dep_children(&child).unwrap();
        //         for dep in dep_children {
        //             self.attach(child.clone(), dep)
        //         }
        //     }
        // }
        //
        // *self.attachment.get_mut(&child).unwrap() = child_attachments;
        // self.add_dep_child(&parent, child.clone());
        // // TODO: pass all attachments from parent to child
        // self.drop_from_frame(&child);
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

    pub fn return_frame(&mut self, _k: fastn_runtime::PointerKey) -> fastn_runtime::PointerKey {
        todo!()
    }

    pub fn create_boolean(&mut self, value: bool) -> fastn_runtime::PointerKey {
        let pointer = self.boolean.insert(HeapValue::new(value).into_heap_data());
        self.insert_in_frame(pointer, Kind::Boolean);
        pointer
    }

    pub fn get_boolean(&mut self, _ptr: fastn_runtime::PointerKey) -> bool {
        // let pointer = self.boolean.insert((value, vec![]));
        // self.insert_in_frame(pointer, Kind::Boolean);
        // pointer
        todo!()
    }

    pub fn create_i32(&mut self, value: i32) -> fastn_runtime::PointerKey {
        let pointer = self.i32.insert(HeapValue::new(value).into_heap_data());
        self.insert_in_frame(pointer, Kind::Integer);
        pointer
    }

    pub(crate) fn create_closure(&mut self, closure: Closure) -> fastn_runtime::ClosureKey {
        self.closures.insert(closure)
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
        self.insert_in_frame(pointer, Kind::Integer);
        pointer
    }

    pub fn get_i32(&mut self, _ptr: fastn_runtime::PointerKey) -> i32 {
        // let pointer = self.i32.insert((value, vec![]));
        // self.insert_in_frame(pointer, Kind::Integer);
        // pointer
        todo!()
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
        *self.i32.get(ptr.key).unwrap().value.value()
    }

    pub fn array_i32_2(
        &mut self,
        ptr1: fastn_runtime::PointerKey,
        ptr2: fastn_runtime::PointerKey,
    ) -> fastn_runtime::PointerKey {
        let vec = self.vec.insert(
            HeapValue::new(vec![
                KindPointer {
                    key: ptr1,
                    kind: Kind::Integer,
                },
                KindPointer {
                    key: ptr2,
                    kind: Kind::Integer,
                },
            ])
            .into_heap_data(),
        );
        self.add_dependent(ptr1.into_integer_pointer(), vec.into_list_pointer());
        self.add_dependent(ptr2.into_integer_pointer(), vec.into_list_pointer());
        vec
    }

    pub fn add_dependent(&mut self, target: KindPointer, dependent: KindPointer) {
        let dependents = match target.kind {
            Kind::Integer => &mut self.i32.get_mut(target.key).unwrap().dependents,
            Kind::Boolean => &mut self.boolean.get_mut(target.key).unwrap().dependents,
            Kind::Decimal => &mut self.f32.get_mut(target.key).unwrap().dependents,
            Kind::List | Kind::Record | Kind::OrType => {
                &mut self.vec.get_mut(target.key).unwrap().dependents
            }
        };

        dependents.push(dependent);
    }

    pub fn add_ui_dependent(&mut self, target: KindPointer, dependent: UIDependent) {
        let dependents = match target.kind {
            Kind::Integer => &mut self.i32.get_mut(target.key).unwrap().ui_properties,
            Kind::Boolean => &mut self.boolean.get_mut(target.key).unwrap().ui_properties,
            Kind::Decimal => &mut self.f32.get_mut(target.key).unwrap().ui_properties,
            Kind::List | Kind::Record | Kind::OrType => {
                &mut self.vec.get_mut(target.key).unwrap().ui_properties
            }
        };

        dependents.push(dependent);
    }

    pub fn create_rgba(&mut self, r: i32, g: i32, b: i32, a: f32) -> fastn_runtime::PointerKey {
        let r_pointer = self.i32.insert(HeapValue::new(r).into_heap_data());
        let g_pointer = self.i32.insert(HeapValue::new(g).into_heap_data());
        let b_pointer = self.i32.insert(HeapValue::new(b).into_heap_data());
        let a_pointer = self.f32.insert(HeapValue::new(a).into_heap_data());

        let vec = self.vec.insert(
            HeapValue::new(vec![
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
            ])
            .into_heap_data(),
        );

        self.add_dependent(r_pointer.into_integer_pointer(), vec.into_record_pointer());
        self.add_dependent(g_pointer.into_integer_pointer(), vec.into_record_pointer());
        self.add_dependent(b_pointer.into_integer_pointer(), vec.into_record_pointer());
        self.add_dependent(a_pointer.into_integer_pointer(), vec.into_record_pointer());

        self.insert_in_frame(vec, Kind::Record);
        vec
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn gc() {
        let mut m = super::Memory::default();
        println!("{:#?}", m);
        assert!(m.is_empty());
        m.create_frame();
        m.create_boolean(true);
        m.end_frame();
        assert!(m.is_empty());
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
