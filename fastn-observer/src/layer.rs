// borrowed from https://github.com/QnnOkabayashi/tracing-forest/ (license: MIT)

pub const SPAN_NOT_IN_CONTEXT: &str = "Span not in context, this is a bug";
pub const OPENED_SPAN_NOT_IN_EXTENSIONS: &str =
    "Span extension doesn't contain `OpenedSpan`, this is a bug";
pub const WRITING_URGENT_ERROR: &str = "writing_urgent failed, this is a bug";

#[derive(Default)]
pub struct Layer {}

impl<S> tracing_subscriber::Layer<S> for Layer
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a> + std::fmt::Debug,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes,
        id: &tracing::Id,
        ctx: tracing_subscriber::layer::Context<S>,
    ) {
        let span = ctx.span(id).expect(SPAN_NOT_IN_CONTEXT);
        let opened = fastn_observer::OpenedSpan::new(
            attrs,
            span.parent().and_then(|v| {
                v.extensions()
                    .get::<fastn_observer::OpenedSpan>()
                    .map(|v| v.start)
            }),
        );

        let mut extensions = span.extensions_mut();
        extensions.insert(opened);
    }

    fn on_event(&self, event: &tracing::Event, ctx: tracing_subscriber::layer::Context<S>) {
        struct Visitor {
            message: Option<String>,
            fields: fastn_observer::FieldSet,
            immediate: bool,
        }

        impl tracing::field::Visit for Visitor {
            fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
                match field.name() {
                    "immediate" => self.immediate |= value,
                    _ => self.record_debug(field, &value),
                }
            }

            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                let value = format!("{:?}", value);
                match field.name() {
                    "message" if self.message.is_none() => self.message = Some(value),
                    key => self.fields.push(fastn_observer::Field::new(key, value)),
                }
            }
        }

        let mut visitor = Visitor {
            message: None,
            fields: fastn_observer::FieldSet::default(),
            immediate: false,
        };

        event.record(&mut visitor);
        let current_span = ctx.event_span(event);

        let shared = fastn_observer::Shared {
            level: *event.metadata().level(),
            fields: visitor.fields,
            on: current_span
                .as_ref()
                .and_then(|v| {
                    v.extensions()
                        .get::<fastn_observer::OpenedSpan>()
                        .map(|v| v.start)
                })
                .unwrap_or_else(std::time::Instant::now)
                .elapsed(),
        };

        let tree_event = fastn_observer::Event {
            shared,
            message: visitor.message,
        };

        if visitor.immediate {
            fastn_observer::write_immediate(&tree_event, current_span.as_ref())
                .expect(WRITING_URGENT_ERROR);
        }

        match current_span.as_ref() {
            Some(parent) => parent
                .extensions_mut()
                .get_mut::<fastn_observer::OpenedSpan>()
                .expect(OPENED_SPAN_NOT_IN_EXTENSIONS)
                .record_event(tree_event),
            None => fastn_observer::write_immediate(&tree_event, current_span.as_ref())
                .expect(WRITING_URGENT_ERROR),
        }
    }

    fn on_close(&self, id: tracing::Id, ctx: tracing_subscriber::layer::Context<S>) {
        let span_ref = ctx.span(&id).expect(SPAN_NOT_IN_CONTEXT);

        let span = span_ref
            .extensions_mut()
            .remove::<fastn_observer::OpenedSpan>()
            .expect(OPENED_SPAN_NOT_IN_EXTENSIONS)
            .close();

        match span_ref.parent() {
            Some(parent) => parent
                .extensions_mut()
                .get_mut::<fastn_observer::OpenedSpan>()
                .expect(OPENED_SPAN_NOT_IN_EXTENSIONS)
                .record_span(span),
            None => {
                if fastn_observer::is_traced() {
                    println!(
                        "{}",
                        fastn_observer::formatter::Pretty {}
                            .fmt(&fastn_observer::Tree::Span(span))
                            .unwrap()
                    );
                }
            }
        }
    }
}
