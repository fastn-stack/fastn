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
}
