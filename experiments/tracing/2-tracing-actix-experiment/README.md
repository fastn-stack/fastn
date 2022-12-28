## tracing_actix_web

## Example1 Code

### For Running the Example

```shell

cargo run

# Open Different Shell 
curl -v http://localhost:8080/hello/my-name

```

### Middleware

```rust
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
```

### Subscriber

```rust
fn init_telemetry() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(tracing::Level::INFO)
        .with_level(false)
        .event_format(tracing_subscriber::fmt::format().compact())
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install `tracing` subscriber.")
}

```

### Functions

```rust

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

```


## Logs Output

```shell
2022-12-28T04:43:38.123550Z  INFO actix_server::builder: Starting 8 workers
2022-12-28T04:43:38.123731Z  INFO actix_server::server: Actix runtime found; starting in Actix runtime
2022-12-28T04:43:39.798274Z  INFO HTTP request:personal_hello: tracing_actix: foo="hello-world" http.method=GET http.route=/hello/{name} http.flavor=1.1 http.scheme=http http.host=localhost:8080 http.client_ip=127.0.0.1 http.user_agent=curl/7.84.0 http.target=/hello/my-name otel.name=HTTP GET /hello/{name} otel.kind="server" request_id=4311588b-a76d-49cb-8a03-c96bb01ba8d1 n_headers=3 cloud_provider="localhost" caller_name="my-name" name=Path("my-name")
2022-12-28T04:43:39.798322Z  INFO HTTP request:personal_hello:foo: tracing_actix: msg="inside foo function!" param1="param1" http.method=GET http.route=/hello/{name} http.flavor=1.1 http.scheme=http http.host=localhost:8080 http.client_ip=127.0.0.1 http.user_agent=curl/7.84.0 http.target=/hello/my-name otel.name=HTTP GET /hello/{name} otel.kind="server" request_id=4311588b-a76d-49cb-8a03-c96bb01ba8d1 n_headers=3 cloud_provider="localhost" caller_name="my-name" name=Path("my-name") _p1="param1" _p2="param2"
2022-12-28T04:43:39.798347Z  INFO HTTP request:personal_hello:foo: tracing_actix: calling foo1 function! http.method=GET http.route=/hello/{name} http.flavor=1.1 http.scheme=http http.host=localhost:8080 http.client_ip=127.0.0.1 http.user_agent=curl/7.84.0 http.target=/hello/my-name otel.name=HTTP GET /hello/{name} otel.kind="server" request_id=4311588b-a76d-49cb-8a03-c96bb01ba8d1 n_headers=3 cloud_provider="localhost" caller_name="my-name" name=Path("my-name") _p1="param1" _p2="param2"
2022-12-28T04:43:39.798808Z  INFO HTTP request:personal_hello:foo:foo1: tracing_actix: inside foo1 function! http.method=GET http.route=/hello/{name} http.flavor=1.1 http.scheme=http http.host=localhost:8080 http.client_ip=127.0.0.1 http.user_agent=curl/7.84.0 http.target=/hello/my-name otel.name=HTTP GET /hello/{name} otel.kind="server" request_id=4311588b-a76d-49cb-8a03-c96bb01ba8d1 n_headers=3 cloud_provider="localhost" caller_name="my-name" name=Path("my-name") _p1="param1" _p2="param2"
2022-12-28T04:43:44.802457Z  INFO HTTP request:personal_hello:foo:foo1: tracing_actix: foo1 function! ends http.method=GET http.route=/hello/{name} http.flavor=1.1 http.scheme=http http.host=localhost:8080 http.client_ip=127.0.0.1 http.user_agent=curl/7.84.0 http.target=/hello/my-name otel.name=HTTP GET /hello/{name} otel.kind="server" request_id=4311588b-a76d-49cb-8a03-c96bb01ba8d1 n_headers=3 cloud_provider="localhost" caller_name="my-name" name=Path("my-name") _p1="param1" _p2="param2"
2022-12-28T04:43:44.802956Z  INFO HTTP request:personal_hello:foo:foo1: tracing_actix: close time.busy=5.00s time.idle=13.9µs http.method=GET http.route=/hello/{name} http.flavor=1.1 http.scheme=http http.host=localhost:8080 http.client_ip=127.0.0.1 http.user_agent=curl/7.84.0 http.target=/hello/my-name otel.name=HTTP GET /hello/{name} otel.kind="server" request_id=4311588b-a76d-49cb-8a03-c96bb01ba8d1 n_headers=3 cloud_provider="localhost" caller_name="my-name" name=Path("my-name") _p1="param1" _p2="param2"
2022-12-28T04:43:49.804262Z  INFO HTTP request:personal_hello:foo: tracing_actix: foo function! ends http.method=GET http.route=/hello/{name} http.flavor=1.1 http.scheme=http http.host=localhost:8080 http.client_ip=127.0.0.1 http.user_agent=curl/7.84.0 http.target=/hello/my-name otel.name=HTTP GET /hello/{name} otel.kind="server" request_id=4311588b-a76d-49cb-8a03-c96bb01ba8d1 n_headers=3 cloud_provider="localhost" caller_name="my-name" name=Path("my-name") _p1="param1" _p2="param2"
2022-12-28T04:43:49.804533Z  INFO HTTP request:personal_hello:foo: tracing_actix: close time.busy=10.0s time.idle=13.7µs http.method=GET http.route=/hello/{name} http.flavor=1.1 http.scheme=http http.host=localhost:8080 http.client_ip=127.0.0.1 http.user_agent=curl/7.84.0 http.target=/hello/my-name otel.name=HTTP GET /hello/{name} otel.kind="server" request_id=4311588b-a76d-49cb-8a03-c96bb01ba8d1 n_headers=3 cloud_provider="localhost" caller_name="my-name" name=Path("my-name") _p1="param1" _p2="param2"
2022-12-28T04:43:49.804774Z  INFO HTTP request:personal_hello: tracing_actix: close time.busy=10.0s time.idle=12.1µs http.method=GET http.route=/hello/{name} http.flavor=1.1 http.scheme=http http.host=localhost:8080 http.client_ip=127.0.0.1 http.user_agent=curl/7.84.0 http.target=/hello/my-name otel.name=HTTP GET /hello/{name} otel.kind="server" request_id=4311588b-a76d-49cb-8a03-c96bb01ba8d1 n_headers=3 cloud_provider="localhost" caller_name="my-name" name=Path("my-name")
2022-12-28T04:43:49.807514Z  INFO HTTP request: tracing_actix: close time.busy=10.0s time.idle=104µs http.method=GET http.route=/hello/{name} http.flavor=1.1 http.scheme=http http.host=localhost:8080 http.client_ip=127.0.0.1 http.user_agent=curl/7.84.0 http.target=/hello/my-name otel.name=HTTP GET /hello/{name} otel.kind="server" request_id=4311588b-a76d-49cb-8a03-c96bb01ba8d1 n_headers=3 cloud_provider="localhost" caller_name="my-name" http.status_code=200 otel.status_code="OK"
```

## Example2: With JSON Formatter

```rust
fn subscriber() {

    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new(app_name.into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install `tracing` subscriber.")
}


```

```json
{"v":0,"name":"tracing-actix-web-demo","msg":"[HTTP REQUEST - START]","level":30,"hostname":"192.168.1.2","pid":85599,"time":"2022-12-28T04:47:50.984876Z","target":"tracing_actix","line":27,"file":"src/main.rs","otel.kind":"server","http.host":"localhost:8080","http.method":"GET","http.route":"/hello/{name}","http.client_ip":"127.0.0.1","n_headers":3,"http.scheme":"http","http.target":"/hello/my-name","http.flavor":"1.1","http.user_agent":"curl/7.84.0","request_id":"c7876ef5-4842-47ba-b51d-f6fbbee32731","otel.name":"HTTP GET /hello/{name}","cloud_provider":"localhost"}
{"v":0,"name":"tracing-actix-web-demo","msg":"[PERSONAL_HELLO - START]","level":30,"hostname":"192.168.1.2","pid":85599,"time":"2022-12-28T04:47:50.985054Z","target":"tracing_actix","line":62,"file":"src/main.rs","otel.kind":"server","http.host":"localhost:8080","http.method":"GET","http.route":"/hello/{name}","http.client_ip":"127.0.0.1","n_headers":3,"http.scheme":"http","http.target":"/hello/my-name","http.flavor":"1.1","http.user_agent":"curl/7.84.0","request_id":"c7876ef5-4842-47ba-b51d-f6fbbee32731","otel.name":"HTTP GET /hello/{name}","cloud_provider":"localhost"}
{"v":0,"name":"tracing-actix-web-demo","msg":"[PERSONAL_HELLO - EVENT] tracing_actix","level":30,"hostname":"192.168.1.2","pid":85599,"time":"2022-12-28T04:47:50.985811Z","target":"tracing_actix","line":66,"file":"src/main.rs","foo":"hello-world","otel.kind":"server","http.host":"localhost:8080","http.method":"GET","http.route":"/hello/{name}","http.client_ip":"127.0.0.1","n_headers":3,"http.scheme":"http","http.target":"/hello/my-name","http.flavor":"1.1","http.user_agent":"curl/7.84.0","request_id":"c7876ef5-4842-47ba-b51d-f6fbbee32731","otel.name":"HTTP GET /hello/{name}","cloud_provider":"localhost"}
{"v":0,"name":"tracing-actix-web-demo","msg":"[FOO - START]","level":30,"hostname":"192.168.1.2","pid":85599,"time":"2022-12-28T04:47:50.986017Z","target":"tracing_actix","line":48,"file":"src/main.rs","_p2":"param2","http.method":"GET","http.scheme":"http","http.target":"/hello/my-name","otel.name":"HTTP GET /hello/{name}","otel.kind":"server","http.host":"localhost:8080","http.route":"/hello/{name}","http.client_ip":"127.0.0.1","n_headers":3,"_p1":"param1","http.flavor":"1.1","http.user_agent":"curl/7.84.0","request_id":"c7876ef5-4842-47ba-b51d-f6fbbee32731","cloud_provider":"localhost"}
{"v":0,"name":"tracing-actix-web-demo","msg":"[FOO - EVENT] tracing_actix","level":30,"hostname":"192.168.1.2","pid":85599,"time":"2022-12-28T04:47:50.986091Z","target":"tracing_actix","line":50,"file":"src/main.rs","param1":"param1","_p2":"param2","http.method":"GET","http.scheme":"http","http.target":"/hello/my-name","otel.name":"HTTP GET /hello/{name}","otel.kind":"server","http.host":"localhost:8080","http.route":"/hello/{name}","http.client_ip":"127.0.0.1","n_headers":3,"_p1":"param1","http.flavor":"1.1","http.user_agent":"curl/7.84.0","request_id":"c7876ef5-4842-47ba-b51d-f6fbbee32731","cloud_provider":"localhost"}
{"v":0,"name":"tracing-actix-web-demo","msg":"[FOO - EVENT] calling foo1 function!","level":30,"hostname":"192.168.1.2","pid":85599,"time":"2022-12-28T04:47:50.987209Z","target":"tracing_actix","line":51,"file":"src/main.rs","_p2":"param2","http.method":"GET","http.scheme":"http","http.target":"/hello/my-name","otel.name":"HTTP GET /hello/{name}","otel.kind":"server","http.host":"localhost:8080","http.route":"/hello/{name}","http.client_ip":"127.0.0.1","n_headers":3,"_p1":"param1","http.flavor":"1.1","http.user_agent":"curl/7.84.0","request_id":"c7876ef5-4842-47ba-b51d-f6fbbee32731","cloud_provider":"localhost"}
{"v":0,"name":"tracing-actix-web-demo","msg":"[FOO1 - START]","level":30,"hostname":"192.168.1.2","pid":85599,"time":"2022-12-28T04:47:50.987447Z","target":"tracing_actix","line":41,"file":"src/main.rs","_p2":"param2","http.method":"GET","http.scheme":"http","http.target":"/hello/my-name","otel.name":"HTTP GET /hello/{name}","otel.kind":"server","http.host":"localhost:8080","http.route":"/hello/{name}","http.client_ip":"127.0.0.1","n_headers":3,"_p1":"param1","http.flavor":"1.1","http.user_agent":"curl/7.84.0","request_id":"c7876ef5-4842-47ba-b51d-f6fbbee32731","cloud_provider":"localhost"}
{"v":0,"name":"tracing-actix-web-demo","msg":"[FOO1 - EVENT] inside foo1 function!","level":30,"hostname":"192.168.1.2","pid":85599,"time":"2022-12-28T04:47:50.989559Z","target":"tracing_actix","line":43,"file":"src/main.rs","_p2":"param2","http.method":"GET","http.scheme":"http","http.target":"/hello/my-name","otel.name":"HTTP GET /hello/{name}","otel.kind":"server","http.host":"localhost:8080","http.route":"/hello/{name}","http.client_ip":"127.0.0.1","n_headers":3,"_p1":"param1","http.flavor":"1.1","http.user_agent":"curl/7.84.0","request_id":"c7876ef5-4842-47ba-b51d-f6fbbee32731","cloud_provider":"localhost"}
{"v":0,"name":"tracing-actix-web-demo","msg":"[FOO1 - EVENT] foo1 function! ends","level":30,"hostname":"192.168.1.2","pid":85599,"time":"2022-12-28T04:47:55.990925Z","target":"tracing_actix","line":45,"file":"src/main.rs","_p2":"param2","http.method":"GET","http.scheme":"http","http.target":"/hello/my-name","otel.name":"HTTP GET /hello/{name}","otel.kind":"server","http.host":"localhost:8080","http.route":"/hello/{name}","http.client_ip":"127.0.0.1","n_headers":3,"_p1":"param1","http.flavor":"1.1","http.user_agent":"curl/7.84.0","request_id":"c7876ef5-4842-47ba-b51d-f6fbbee32731","cloud_provider":"localhost"}
{"v":0,"name":"tracing-actix-web-demo","msg":"[FOO1 - END]","level":30,"hostname":"192.168.1.2","pid":85599,"time":"2022-12-28T04:47:55.991412Z","target":"tracing_actix","line":41,"file":"src/main.rs","_p2":"param2","http.method":"GET","elapsed_milliseconds":5001,"http.scheme":"http","http.target":"/hello/my-name","otel.name":"HTTP GET /hello/{name}","otel.kind":"server","http.host":"localhost:8080","http.route":"/hello/{name}","http.client_ip":"127.0.0.1","n_headers":3,"_p1":"param1","http.flavor":"1.1","http.user_agent":"curl/7.84.0","request_id":"c7876ef5-4842-47ba-b51d-f6fbbee32731","cloud_provider":"localhost"}
{"v":0,"name":"tracing-actix-web-demo","msg":"[FOO - EVENT] foo function! ends","level":30,"hostname":"192.168.1.2","pid":85599,"time":"2022-12-28T04:48:00.992833Z","target":"tracing_actix","line":54,"file":"src/main.rs","_p2":"param2","http.method":"GET","http.scheme":"http","http.target":"/hello/my-name","otel.name":"HTTP GET /hello/{name}","otel.kind":"server","http.host":"localhost:8080","http.route":"/hello/{name}","http.client_ip":"127.0.0.1","n_headers":3,"_p1":"param1","http.flavor":"1.1","http.user_agent":"curl/7.84.0","request_id":"c7876ef5-4842-47ba-b51d-f6fbbee32731","cloud_provider":"localhost"}
{"v":0,"name":"tracing-actix-web-demo","msg":"[FOO - END]","level":30,"hostname":"192.168.1.2","pid":85599,"time":"2022-12-28T04:48:00.993102Z","target":"tracing_actix","line":48,"file":"src/main.rs","_p2":"param2","http.method":"GET","elapsed_milliseconds":10007,"http.scheme":"http","http.target":"/hello/my-name","otel.name":"HTTP GET /hello/{name}","otel.kind":"server","http.host":"localhost:8080","http.route":"/hello/{name}","http.client_ip":"127.0.0.1","n_headers":3,"_p1":"param1","http.flavor":"1.1","http.user_agent":"curl/7.84.0","request_id":"c7876ef5-4842-47ba-b51d-f6fbbee32731","cloud_provider":"localhost"}
{"v":0,"name":"tracing-actix-web-demo","msg":"[PERSONAL_HELLO - END]","level":30,"hostname":"192.168.1.2","pid":85599,"time":"2022-12-28T04:48:00.993253Z","target":"tracing_actix","line":62,"file":"src/main.rs","http.method":"GET","elapsed_milliseconds":10007,"http.scheme":"http","http.target":"/hello/my-name","otel.name":"HTTP GET /hello/{name}","otel.kind":"server","http.host":"localhost:8080","http.route":"/hello/{name}","http.client_ip":"127.0.0.1","n_headers":3,"http.flavor":"1.1","http.user_agent":"curl/7.84.0","request_id":"c7876ef5-4842-47ba-b51d-f6fbbee32731","cloud_provider":"localhost"}
{"v":0,"name":"tracing-actix-web-demo","msg":"[HTTP REQUEST - END]","level":30,"hostname":"192.168.1.2","pid":85599,"time":"2022-12-28T04:48:00.993547Z","target":"tracing_actix","line":27,"file":"src/main.rs","http.method":"GET","elapsed_milliseconds":10008,"http.scheme":"http","http.target":"/hello/my-name","http.status_code":200,"otel.name":"HTTP GET /hello/{name}","otel.kind":"server","http.host":"localhost:8080","http.route":"/hello/{name}","http.client_ip":"127.0.0.1","n_headers":3,"caller_name":"my-name","otel.status_code":"OK","http.flavor":"1.1","http.user_agent":"curl/7.84.0","request_id":"c7876ef5-4842-47ba-b51d-f6fbbee32731","cloud_provider":"localhost"}
```