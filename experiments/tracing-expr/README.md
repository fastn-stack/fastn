# Experiments

In this experiment, we are going to test what all we can do with tracing crate.

# Adding crates

`cargo add tracing tracing_subscriber actix-web`

# Example

This is how we have to initialize the subscriber. 

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // log::info!("starting HTTP server at http://localhost:8080");

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::INFO)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    HttpServer::new(|| {
        App::new()
            // enable logger
            // .wrap(middleware::Logger::default())
            .app_data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
            .service(web::resource("/m").route(web::post().to(index_manual)))
            .service(web::resource("/").route(web::post().to(index)))

        // .service(web::resource("/extractor").route(web::post().to(index)))
            // .service(
            //     web::resource("/extractor2")
            //         .app_data(web::JsonConfig::default().limit(1024)) // <- limit size of the payload (resource level)
            //         .route(web::post().to(extract_item)),
            // )
            // .service(web::resource("/mjsonrust").route(web::post().to(index_mjsonrust)))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
```

This is we are using tracing macros on the functions

```rust

#[instrument]
async fn foo1() {
    event!(Level::INFO, "inside foo1 function!");
    std::thread::sleep(std::time::Duration::from_secs(5));
    event!(Level::INFO, "foo1 function! ends");
}

#[instrument]
async fn foo() {
    event!(Level::INFO, "inside foo function!");
    event!(Level::INFO, "calling foo1 function!");
    foo1().await;
    std::thread::sleep(std::time::Duration::from_secs(5));
    event!(Level::INFO, "foo function! ends");
}

/// This handler uses json extractor
#[instrument]
async fn index(item: web::Json<MyObj>) -> HttpResponse {
    event!(Level::INFO, "inside index function!");
    // println!("model: {:?}", &item);
    event!(Level::INFO, "calling function foo!");
    foo().await;
    HttpResponse::Ok().json(item.0) // <- send response
}
```

This is what the output is coming on the terminal while curling the request for index

Run the actix server using `cargo run`, use the below curl request with the new terminal

```shell
curl --header "Content-type: application/json" -d '{"name": "Abrar", "number": 1}' "http://127.0.0.1:8080/"
{"name":"Abrar","number":1}
```

```shell
➜  tracing-expr git:(main) ✗ cargo run
Compiling tracing-expr v0.1.0 (/Users/wilderbit/github/fpm/experiments/tracing-expr)
Finished dev [unoptimized + debuginfo] target(s) in 3.00s
Running `target/debug/tracing-expr`

2022-12-26T16:59:16.494215Z  INFO actix_server::builder: Starting 8 workers
2022-12-26T16:59:16.494338Z  INFO actix_server::server: Actix runtime found; starting in Actix runtime

2022-12-26T17:01:56.225765Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}: tracing_expr: new
2022-12-26T17:01:56.225800Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}: tracing_expr: enter
2022-12-26T17:01:56.225814Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}: tracing_expr: inside index function!
2022-12-26T17:01:56.225827Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}: tracing_expr: calling function foo!
2022-12-26T17:01:56.225854Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}:foo: tracing_expr: new
2022-12-26T17:01:56.225883Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}:foo: tracing_expr: enter
2022-12-26T17:01:56.225905Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}:foo: tracing_expr: inside foo function!
2022-12-26T17:01:56.255226Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}:foo: tracing_expr: calling foo1 function!
2022-12-26T17:01:56.255310Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}:foo:foo1: tracing_expr: new
2022-12-26T17:01:56.255337Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}:foo:foo1: tracing_expr: enter
2022-12-26T17:01:56.255353Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}:foo:foo1: tracing_expr: inside foo1 function!
2022-12-26T17:02:01.258873Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}:foo:foo1: tracing_expr: foo1 function! ends
2022-12-26T17:02:01.259132Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}:foo:foo1: tracing_expr: exit
2022-12-26T17:02:01.259317Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}:foo:foo1: tracing_expr: close time.busy=5.00s time.idle=215µs
2022-12-26T17:02:06.263115Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}:foo: tracing_expr: foo function! ends
2022-12-26T17:02:06.263329Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}:foo: tracing_expr: exit
2022-12-26T17:02:06.263384Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}:foo: tracing_expr: close time.busy=10.0s time.idle=84.2µs
2022-12-26T17:02:06.263694Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}: tracing_expr: exit
2022-12-26T17:02:06.263739Z  INFO index{item=Json(MyObj { name: "Abrar", number: 1 })}: tracing_expr: close time.busy=10.0s time.idle=84.0µs

```