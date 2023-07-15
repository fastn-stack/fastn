pub struct Layer {
    start: std::time::Instant,
}

impl Layer {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    fn dbg(&self, message: String) {
        let duration = fastn_observer::DurationDisplay(self.start.elapsed().as_nanos() as f64);
        println!("{duration} {message}");
    }
}

impl<S> tracing_subscriber::Layer<S> for Layer
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a> + std::fmt::Debug,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes,
        id: &tracing::Id,
        _ctx: tracing_subscriber::layer::Context<S>,
    ) {
        self.dbg(format!("on_new_span: {id:?} {attrs:#?}"));
    }

    fn on_event(&self, event: &tracing::Event, _ctx: tracing_subscriber::layer::Context<S>) {
        self.dbg(format!("on_event: {event:#?}"));
    }

    fn on_enter(&self, id: &tracing::Id, _ctx: tracing_subscriber::layer::Context<S>) {
        self.dbg(format!("on_enter: {id:?}"));
    }

    fn on_exit(&self, id: &tracing::Id, _ctx: tracing_subscriber::layer::Context<S>) {
        self.dbg(format!("on_exit: {id:?}"));
    }

    fn on_close(&self, id: tracing::Id, _ctx: tracing_subscriber::layer::Context<S>) {
        self.dbg(format!("on_close: {id:?}"));
    }
}
