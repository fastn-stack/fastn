// borrowed from https://github.com/QnnOkabayashi/tracing-forest/ (license: MIT)

#[derive(Debug)]
pub enum Tree {
    Event(Event),
    Span(Span),
}

#[derive(Debug)]
pub struct Event {
    pub(crate) shared: Shared,
    pub(crate) message: Option<String>,
}

#[derive(Debug)]
pub struct Span {
    pub(crate) shared: Shared,
    pub(crate) name: &'static str,
    pub(crate) total_duration: std::time::Duration,
    pub(crate) inner_duration: std::time::Duration,
    pub(crate) nodes: Vec<Tree>,
}

#[derive(Debug)]
pub struct Shared {
    pub(crate) level: tracing::Level,
    pub(crate) fields: fastn_observer::FieldSet,
    /// when did this event occur, with respect to immediate parent start
    pub(crate) on: std::time::Duration,
}

impl Span {
    pub(crate) fn new(shared: Shared, name: &'static str) -> Self {
        Span {
            shared,
            name,
            total_duration: std::time::Duration::ZERO,
            inner_duration: std::time::Duration::ZERO,
            nodes: Vec::new(),
        }
    }
}
