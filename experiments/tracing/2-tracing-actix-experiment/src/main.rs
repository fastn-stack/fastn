use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{web, App, Error, HttpServer};
use opentelemetry::{
    global, runtime::TokioCurrentThread, sdk::propagation::TraceContextPropagator,
};
use std::io;
use tracing::{event, instrument, Level, Span};
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpan, RootSpanBuilder, TracingLogger};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

/// We will define a custom root span builder to capture additional fields, specific
/// to our application, on top of the ones provided by `DefaultRootSpanBuilder` out of the box.
pub struct CustomRootSpanBuilder;

impl RootSpanBuilder for CustomRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        // Not sure why you'd be keen to capture this, but it's an example and we try to keep it simple
        let n_headers = request.headers().len();
        // We set `cloud_provider` to a constant value.
        //
        // `name` is not known at this point - we delegate the responsibility to populate it
        // to the `personal_hello` handler. We MUST declare the field though, otherwise
        // `span.record("caller_name", XXX)` will just be silently ignored by `tracing`.
        tracing_actix_web::root_span!(
            request,
            n_headers,
            cloud_provider = "localhost",
            caller_name = tracing::field::Empty
        )
    }

    fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        // Capture the standard fields when the request finishes.
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}

#[tracing::instrument]
async fn foo1() {
    tracing::event!(Level::INFO, "inside foo1 function!");
    std::thread::sleep(std::time::Duration::from_secs(5));
    event!(Level::INFO, "foo1 function! ends");
}

#[tracing::instrument]
async fn foo(_p1: &str, _p2: &str) {
    tracing::event!(Level::INFO, msg = "inside foo function!", param1 = _p1,);
    tracing::event!(Level::INFO, "calling foo1 function!");
    foo1().await;
    std::thread::sleep(std::time::Duration::from_secs(5));
    tracing::event!(Level::INFO, "foo function! ends");
}

#[tracing::instrument]
async fn hello() -> &'static str {
    "Hello world!"
}

#[tracing::instrument(skip(root_span))]
async fn personal_hello(root_span: RootSpan, name: web::Path<String>) -> String {
    // Add more context to the root span of the request.
    root_span.record("caller_name", &name.as_str());
    tracing::info!(foo = "hello-world");
    foo("param1", "param2").await;
    format!("Hello {}!", name)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    init_telemetry();

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::<CustomRootSpanBuilder>::new())
            .service(web::resource("/hello").to(hello))
            .service(web::resource("/hello/{name}").to(personal_hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    // Ensure all spans have been shipped to Jaeger.
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}

/// Init a `tracing` subscriber that prints spans to stdout as well as
/// ships them to Jaeger.
///
/// Check the `opentelemetry` example for more details.
fn init_telemetry() {
    let app_name = "tracing-actix-web-demo";

    // global::set_text_map_propagator(TraceContextPropagator::new());
    // let tracer = opentelemetry_jaeger::new_pipeline()
    //     .with_service_name(app_name)
    //     .install_batch(TokioCurrentThread)
    //     .expect("Failed to install OpenTelemetry tracer.");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));
    // let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let formatting_layer = BunyanFormattingLayer::new(app_name.into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        // .with(telemetry)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    // let subscriber = tracing_subscriber::FmtSubscriber::builder()
    //     // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
    //     // will be written to stdout.
    //     .with_max_level(tracing::Level::INFO)
    //     .with_level(false)
    //     .event_format(tracing_subscriber::fmt::format().compact())
    //     .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
    //     .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
    //     // completes the builder.
    //     .finish();
    //
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install `tracing` subscriber.")
}
