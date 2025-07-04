impl fastn::commands::Serve {
    pub async fn run(self, _package: fastn_package::MainPackage, _router: fastn_router::Router) {
        let listener = match tokio::net::TcpListener::bind(&self.listen).await {
            Ok(listener) => listener,
            Err(e) => panic!("failed to bind to {}: {}", self.listen, e),
        };
        println!("Listening on {}://{}.", self.protocol, self.listen);
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(stream) => stream,
                Err(e) => {
                    eprintln!("failed to accept: {e:?}");
                    // TODO: is continue safe here?
                    //       can we go in accept failure infinite loop?
                    //       why would accept fail?
                    continue;
                }
            };

            // Use an adapter to access something implementing `tokio::io` traits as if they implement
            // `hyper::rt` IO traits.
            let io = hyper_util::rt::TokioIo::new(stream);

            // Spawn a tokio task to serve multiple connections concurrently
            tokio::task::spawn(async move {
                // Finally, we bind the incoming connection to our `hello` service
                if let Err(err) = hyper::server::conn::http1::Builder::new()
                    // `service_fn` converts our function in a `Service`
                    .serve_connection(io, hyper::service::service_fn(render))
                    .await
                {
                    eprintln!("Error serving connection: {err:?}");
                }
            });
        }
    }
}

async fn render(
    r: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<http_body_util::Full<hyper::body::Bytes>>, std::convert::Infallible> {
    println!("rendering1 {}: {}", r.method(), r.uri());
    // let route = fastn_core::Route::Document("index.ftd".to_string(), serde_json::Value::Null);
    Ok(hyper::Response::new(http_body_util::Full::new(
        hyper::body::Bytes::from(
            fastn::commands::render::render_document(
                // global_aliases,
                "index.ftd",
                serde_json::Map::new(),
                false,
            )
            .await
            .into_bytes(),
        ),
    )))
}
