impl fastn_core::iroh::Proxy {
    pub async fn run(&self) -> fastn_core::Result<()> {
        let listen = format!("127.0.0.1:{}", self.port);
        let listener = match tokio::net::TcpListener::bind(listen.as_str()).await {
            Ok(listener) => listener,
            Err(e) => panic!("failed to bind to {}: {}", self.port, e),
        };
        println!("Listening on {}://127.0.0.1:{}.", self.protocol, self.port);
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
                    .serve_connection(io, hyper::service::service_fn(proxy))
                    .await
                {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}

// working of the proxy
// ====================
//
// startup
// -------
//
// we should keep a connection open to the remote service when the service starts.
// this way we can print some diagnostic/success message as soon as you start the service.
//
// against the connection, we get access to a bidirectional channel,
// and we should store that as well.
//
// the connection can break, say during a network outage, or if the remote service is restarted,
// we should drop all incoming http requests in that connection and renew the connection and
// re-create the bidirectional channel.
//
// request multiplexing
// --------------------
//
// when a incoming request comes, we should see if there is a channel open to the remote service,
// if so we should send the request to the remote service and wait for the response.
// since multiple connections can come in at the same time, we should create a request-id for each
// request, we should await the response for that request-id.
//
// we will probably keep a max limit on the number of requests we can handle at the same time.
// for now, we are not going to implement that.

async fn proxy(
    _r: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<http_body_util::Full<hyper::body::Bytes>>, std::convert::Infallible> {
    todo!()
}
