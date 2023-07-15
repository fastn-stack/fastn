pub fn observe() {
    use tracing_subscriber::layer::SubscriberExt;

    // let level = std::env::var("TRACING")
    //     .unwrap_or_else(|_| "info".to_string())
    //     .parse::<tracing_forest::util::LevelFilter>()
    //     .unwrap_or(tracing_forest::util::LevelFilter::INFO);

    // only difference between the two branches of this if condition is the extra forest layer.
    if is_traced() {
        let s = tracing_subscriber::registry()
            //.with(level)
            .with(Layer::new());
        tracing::subscriber::set_global_default(s).unwrap();
    } else {
        let s = tracing_subscriber::registry()
            //.with(level)
            .with(Layer::new());
        tracing::subscriber::set_global_default(s).unwrap();
        // let s = tracing_subscriber::registry().with(level);
        // tracing::subscriber::set_global_default(s).unwrap();
    }
}

pub fn is_traced() -> bool {
    std::env::var("TRACING").is_ok() || std::env::args().any(|e| e == "--trace")
}

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
        let duration = DurationDisplay(self.start.elapsed().as_nanos() as f64);
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

struct DurationDisplay(f64);

// Taken from chrono
impl std::fmt::Display for DurationDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut t = self.0;
        for unit in ["ns", "µs", "ms", "s"] {
            if t < 10.0 {
                return write!(f, "{:.2}{}", t, unit);
            } else if t < 100.0 {
                return write!(f, "{:.1}{}", t, unit);
            } else if t < 1000.0 {
                return write!(f, "{:.0}{}", t, unit);
            }
            t /= 1000.0;
        }
        write!(f, "{:.0}s", t * 1000.0)
    }
}
