pub type Heap<T> = slotmap::SlotMap<fastn_runtime::PointerKey, HeapData<T>>;

/// For every ftd value we have one such entry
#[derive(Debug)]
pub struct HeapData<T> {
    /// The inner value being stored in ftd
    pub value: HeapValue<T>,
    /// the list of values that depend on this, eg if we add x to a list l, we also do a
    /// x.parents.add(l)
    pub parents: Vec<fastn_runtime::Pointer>,
    /// whenever a dom node is added or deleted, it is added or removed from this list.
    pub ui_properties: Vec<fastn_runtime::DynamicProperty>,

    /// things are connected to root us via branches. One can be attached to more than one branches,
    /// or to same branch by more than "via"s. When a pointer is created it is connected with no
    /// branches. When the pointer is added to a UI via set_property(), we add an Attachment object
    /// to this vector. If T is a container,
    pub branches: std::collections::HashSet<Attachment>,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct Attachment {
    pub branch: fastn_runtime::DynamicProperty,
    pub via: fastn_runtime::Pointer,
}

/// This is the data we store in the heap for any value.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum HeapValue<T> {
    Value(T),

    /// If a value is defined in terms of a function, we store the last computed value and the
    /// closure. We cached the last computed value so if the data is not changing we do not have
    /// to re-compute the closure.
    ///
    /// -- integer x: 10 (stored as HeapValue::Value(10))
    /// -- integer y: 20 (stored as HeapValue::Value(10))
    /// -- integer z: { x + y } ;; (stored as HeapValue::Formula { cached_value: 30, closure: 1v2 }
    Formula {
        cached_value: T,
        closure: fastn_runtime::ClosurePointer,
    },
}

impl<T> HeapData<T> {
    pub(crate) fn new(value: HeapValue<T>) -> HeapData<T> {
        HeapData {
            value,
            parents: vec![],
            ui_properties: vec![],
            branches: std::collections::HashSet::new(),
        }
    }
}

impl<T> HeapValue<T> {
    pub(crate) fn mut_value(&mut self) -> &mut T {
        match self {
            HeapValue::Value(v) => v,
            HeapValue::Formula { cached_value, .. } => cached_value,
        }
    }
    pub(crate) fn value(&self) -> &T {
        match self {
            HeapValue::Value(v) => v,
            HeapValue::Formula { cached_value, .. } => cached_value,
        }
    }
    pub(crate) fn set_value(&mut self, v: T) {
        *self = HeapValue::Value(v);
    }
}

impl<T> HeapValue<T> {
    pub(crate) fn new(value: T) -> HeapValue<T> {
        HeapValue::Value(value)
    }

    pub(crate) fn new_with_formula(
        cached_value: T,
        closure: fastn_runtime::ClosurePointer,
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
