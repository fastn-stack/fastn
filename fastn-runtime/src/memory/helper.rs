impl fastn_runtime::Memory {
    pub(crate) fn get_event_handlers(
        &self,
        event_kind: fastn_runtime::DomEventKind,
        _node: Option<fastn_runtime::NodeKey>,
    ) -> Option<&[fastn_runtime::EventHandler]> {
        if let Some(e) = self.event_handler.get(&event_kind) {
            if event_kind.is_key() {
                return Some(e);
            }
        }
        None
    }

    pub(crate) fn get_heapdata_from_pointer(&self, _pointer: fastn_runtime::Pointer) {
        /* match pointer.kind {
            fastn_runtime::PointerKind::Boolean => self.boolean.get()
            fastn_runtime::PointerKind::Integer => {}
            fastn_runtime::PointerKind::Record => {}
            fastn_runtime::PointerKind::OrType => {}
            fastn_runtime::PointerKind::Decimal => {}
            fastn_runtime::PointerKind::List => {}
            fastn_runtime::PointerKind::String => {}
        }*/
    }
}
