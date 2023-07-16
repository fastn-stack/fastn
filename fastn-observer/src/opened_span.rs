// borrowed from https://github.com/QnnOkabayashi/tracing-forest/ (license: MIT)

pub struct OpenedSpan {
    span: fastn_observer::Span,
    pub start: std::time::Instant,
    last_enter: std::time::Instant,
}

impl OpenedSpan {
    pub fn new(
        attrs: &tracing::span::Attributes,
        parent_start: Option<std::time::Instant>,
    ) -> Self {
        let start = std::time::Instant::now();
        let mut fields = fastn_observer::FieldSet::default();

        attrs.record(
            &mut |field: &tracing::field::Field, value: &dyn std::fmt::Debug| {
                let value = format!("{:?}", value);
                fields.push(fastn_observer::Field::new(field.name(), value));
            },
        );

        let shared = fastn_observer::Shared {
            level: *attrs.metadata().level(),
            fields,
            on: parent_start.unwrap_or(start).elapsed(),
        };

        OpenedSpan {
            span: fastn_observer::Span::new(shared, attrs.metadata().name()),
            last_enter: start,
            start,
        }
    }

    pub fn enter(&mut self) {
        self.last_enter = std::time::Instant::now();
    }

    pub fn exit(&mut self) {
        self.span.total_duration += self.last_enter.elapsed();
    }

    pub fn close(self) -> fastn_observer::Span {
        self.span
    }

    pub fn record_event(&mut self, event: fastn_observer::Event) {
        self.span.nodes.push(fastn_observer::Tree::Event(event));
    }

    pub fn record_span(&mut self, span: fastn_observer::Span) {
        self.span.inner_duration += span.total_duration;
        self.span.nodes.push(fastn_observer::Tree::Span(span));
    }
}
