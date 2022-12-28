#[allow(unused_imports)]
use actix_web::{error, web, App, Error, HttpRequest as _, HttpResponse, HttpServer};
use futures_util::StreamExt as _;
use serde::{Deserialize, Serialize};
use tracing::{event, instrument, Level};
const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[derive(Debug, Serialize, Deserialize)]
struct MyObj {
    name: String,
    number: i32,
}

#[instrument(name = "i-am-foo1")]
async fn foo1() {
    event!(Level::INFO, "inside foo1 function!");
    std::thread::sleep(std::time::Duration::from_secs(5));
    event!(Level::INFO, "foo1 function! ends");
}

#[instrument(name = "i-am-foo", skip_all)]
async fn foo(_item: &web::Json<MyObj>, _p1: &str, _p2: &str) {
    event!(Level::INFO, msg = "inside foo function!", param1 = _p1,);
    event!(Level::INFO, "calling foo1 function!");
    foo1().await;
    std::thread::sleep(std::time::Duration::from_secs(5));
    event!(Level::INFO, "foo function! ends");
}

/// This handler uses json extractor
#[instrument(skip(item, _req), ret)]
async fn index(_req: actix_web::HttpRequest, item: web::Json<MyObj>) -> HttpResponse {
    event!(Level::INFO, "inside index function!");
    event!(Level::INFO, "calling function foo!");
    foo(&item, "param1", "param2").await;
    HttpResponse::Ok().json(item.0) // <- send response
}

/// This handler manually load request payload and parse json object
///
async fn index_manual(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<MyObj>(&body)?;
    Ok(HttpResponse::Ok().json(obj)) // <- send response
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // log::info!("starting HTTP server at http://localhost:8080");

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
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use actix_web::{body::to_bytes, dev::Service, http, test, web, App};

    use super::*;

    #[actix_web::test]
    async fn test_index() {
        let app =
            test::init_service(App::new().service(web::resource("/").route(web::post().to(index))))
                .await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&MyObj {
                name: "my-name".to_owned(),
                number: 43,
            })
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(body_bytes, r##"{"name":"my-name","number":43}"##);
    }
}

/*
/// This handler manually load request payload and parse json-rust
async fn index_mjsonrust(body: web::Bytes) -> Result<HttpResponse, Error> {
    // body is loaded, now we can deserialize json-rust
    let result = json::parse(std::str::from_utf8(&body).unwrap()); // return Result
    let injson: JsonValue = match result {
        Ok(v) => v,
        Err(e) => json::object! {"err" => e.to_string() },
    };
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(injson.dump()))
}

/// This handler uses json extractor with limit
async fn extract_item(item: web::Json<MyObj>, req: HttpRequest) -> HttpResponse {
    println!("request: {req:?}");
    println!("model: {item:?}");
    HttpResponse::Ok().json(item.0) // <- send json response
}

use json::JsonValue;

 */
