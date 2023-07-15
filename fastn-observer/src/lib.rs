extern crate self as fastn_observer;

mod duration_display;
mod field;
mod formatter;
mod layer;
mod opened_span;
mod tree;

pub(crate) use duration_display::DurationDisplay;
pub use field::{Field, FieldSet};
pub use formatter::write_immediate;
pub use layer::Layer;
pub use opened_span::OpenedSpan;
pub use tree::{Event, Shared, Span, Tree};

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
            .with(Layer::default());
        tracing::subscriber::set_global_default(s).unwrap();
    } else {
        let s = tracing_subscriber::registry()
            //.with(level)
            .with(Layer::default());
        tracing::subscriber::set_global_default(s).unwrap();
        // let s = tracing_subscriber::registry().with(level);
        // tracing::subscriber::set_global_default(s).unwrap();
    }
}

pub fn is_traced() -> bool {
    std::env::var("TRACING").is_ok() || std::env::args().any(|e| e == "--trace")
}
