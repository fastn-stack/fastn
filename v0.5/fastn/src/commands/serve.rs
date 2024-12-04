use std::sync::Arc;

impl fastn::commands::Serve {
    pub async fn run(self, config: fastn_core::Config, arena: fastn_unresolved::Arena) {
        let listener = match tokio::net::TcpListener::bind(&self.listen).await {
            Ok(listener) => listener,
            Err(e) => panic!("failed to bind to {}: {}", self.listen, e),
        };
        let arena = Arc::new(arena);
        println!("Listening on {}://{}.", self.listen, self.protocol);
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
                    .serve_connection(
                        io,
                        hyper::service::service_fn(|r| render(r, config.auto_imports, &arena)),
                    )
                    .await
                {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}

async fn render(
    _: hyper::Request<hyper::body::Incoming>,
    auto_imports: fastn_unresolved::AliasesID,
    global_arena: &fastn_unresolved::Arena,
) -> Result<hyper::Response<http_body_util::Full<hyper::body::Bytes>>, std::convert::Infallible> {
    Ok(hyper::Response::new(http_body_util::Full::new(
        // hyper::body::Bytes::from("Hello, World!"),
        hyper::body::Bytes::from(
            fastn::commands::render::render_document(
                auto_imports,
                global_arena,
                "/",
                serde_json::Value::Null,
                false,
            )
            .await
            .into_bytes(),
        ),
    )))
}
