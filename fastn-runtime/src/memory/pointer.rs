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
}
