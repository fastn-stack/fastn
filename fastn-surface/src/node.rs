slotmap::new_key_type! { pub struct NodeKey; }

pub(crate) enum NodeKind {
    FlexContainer,
    Text,
    Image,
}

static GLOBAL_POLL_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub(crate) fn next_id() -> usize {
    GLOBAL_POLL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}
