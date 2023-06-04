slotmap::new_key_type! { pub struct PointerKey; }
slotmap::new_key_type! { pub struct ClosurePointer; }

/// Since a pointer can be present in any of the slotmaps on Memory, .boolean, .i32 etc, we need
/// to keep track of Kind so we know where this pointer came from
#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub struct Pointer {
    pub pointer: fastn_runtime::PointerKey,
    pub kind: PointerKind,
}

impl fastn_runtime::PointerKey {
    pub(crate) fn into_boolean_pointer(self) -> Pointer {
        Pointer {
            pointer: self,
            kind: PointerKind::Boolean,
        }
    }

    pub(crate) fn into_integer_pointer(self) -> Pointer {
        Pointer {
            pointer: self,
            kind: PointerKind::Integer,
        }
    }

    pub(crate) fn into_decimal_pointer(self) -> Pointer {
        Pointer {
            pointer: self,
            kind: PointerKind::Decimal,
        }
    }

    pub(crate) fn into_list_pointer(self) -> Pointer {
        Pointer {
            pointer: self,
            kind: PointerKind::List,
        }
    }

    pub(crate) fn into_record_pointer(self) -> Pointer {
        Pointer {
            pointer: self,
            kind: PointerKind::Record,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub enum PointerKind {
    Boolean,
    Integer,
    Record,
    OrType,
    Decimal,
    List,
    String,
}

impl From<i32> for PointerKind {
    fn from(i: i32) -> PointerKind {
        match i {
            0 => PointerKind::Boolean,
            1 => PointerKind::Integer,
            2 => PointerKind::Record,
            3 => PointerKind::OrType,
            4 => PointerKind::Decimal,
            5 => PointerKind::List,
            6 => PointerKind::String,
            _ => panic!("Unknown element kind: {}", i),
        }
    }
}

impl From<PointerKind> for i32 {
    fn from(s: PointerKind) -> i32 {
        match s {
            PointerKind::Boolean => 0,
            PointerKind::Integer => 1,
            PointerKind::Record => 2,
            PointerKind::OrType => 3,
            PointerKind::Decimal => 4,
            PointerKind::List => 5,
            PointerKind::String => 6,
        }
    }
}
