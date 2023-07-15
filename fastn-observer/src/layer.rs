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
        let opened = fastn_observer::OpenedSpan::new(attrs);

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

        let shared = fastn_observer::Shared {
            level: *event.metadata().level(),
            fields: visitor.fields,
            on: std::time::Duration::ZERO, // TODO
        };

        let tree_event = fastn_observer::Event {
            shared,
            message: visitor.message,
        };

        let current_span = ctx.event_span(event);

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

    fn on_enter(&self, id: &tracing::Id, ctx: tracing_subscriber::layer::Context<S>) {
        ctx.span(id)
            .expect(SPAN_NOT_IN_CONTEXT)
            .extensions_mut()
            .get_mut::<fastn_observer::OpenedSpan>()
            .expect(OPENED_SPAN_NOT_IN_EXTENSIONS)
            .enter();
    }

    fn on_exit(&self, id: &tracing::Id, ctx: tracing_subscriber::layer::Context<S>) {
        ctx.span(id)
            .expect(SPAN_NOT_IN_CONTEXT)
            .extensions_mut()
            .get_mut::<fastn_observer::OpenedSpan>()
            .expect(OPENED_SPAN_NOT_IN_EXTENSIONS)
            .exit();
    }

    fn on_close(&self, id: tracing::Id, ctx: tracing_subscriber::layer::Context<S>) {
        let span_ref = ctx.span(&id).expect(SPAN_NOT_IN_CONTEXT);

        let mut span = span_ref
            .extensions_mut()
            .remove::<fastn_observer::OpenedSpan>()
            .expect(OPENED_SPAN_NOT_IN_EXTENSIONS)
            .close();

        // Ensure that the total duration is at least as much as the inner
        // duration. This is caused by when a child span is manually passed
        // a parent span and then enters without entering the parent span. Also
        // when a child span is created within a parent, and then stored and
        // entered again when the parent isn't opened.
        //
        // Issue: https://github.com/QnnOkabayashi/tracing-forest/issues/11
        if span.total_duration < span.inner_duration {
            span.total_duration = span.inner_duration;
        }

        match span_ref.parent() {
            Some(parent) => parent
                .extensions_mut()
                .get_mut::<fastn_observer::OpenedSpan>()
                .expect(OPENED_SPAN_NOT_IN_EXTENSIONS)
                .record_span(span),
            None => {
                dbg!(span);
                todo!()
            }
        }
    }
}
