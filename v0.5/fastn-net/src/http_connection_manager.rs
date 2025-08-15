/// Connection pool for HTTP/1.1 connections.
///
/// Uses bb8 for connection pooling with automatic health checks and recycling.
pub type HttpConnectionPool = bb8::Pool<HttpConnectionManager>;

/// Collection of connection pools indexed by server address.
///
/// Allows maintaining separate pools for different HTTP servers,
/// enabling efficient connection reuse across multiple targets.
pub type HttpConnectionPools =
    std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, HttpConnectionPool>>>;

/// Manages HTTP/1.1 connections for connection pooling.
///
/// Implements the bb8::ManageConnection trait to handle connection
/// lifecycle including creation, validation, and cleanup.
pub struct HttpConnectionManager {
    addr: String,
}

impl HttpConnectionManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }

    pub async fn connect(
        &self,
    ) -> eyre::Result<
        hyper::client::conn::http1::SendRequest<
            http_body_util::combinators::BoxBody<hyper::body::Bytes, eyre::Error>,
        >,
    > {
        use eyre::WrapErr;

        let stream = tokio::net::TcpStream::connect(&self.addr)
            .await
            .wrap_err_with(|| "failed to open tcp connection")?;
        let io = hyper_util::rt::TokioIo::new(stream);

        let (sender, conn) = hyper::client::conn::http1::handshake(io)
            .await
            .wrap_err_with(|| "failed to do http1 handshake")?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await.wrap_err_with(|| "connection failed") {
                tracing::error!("Connection failed: {err:?}");
            }
        });

        Ok(sender)
    }
}

impl bb8::ManageConnection for HttpConnectionManager {
    type Connection = hyper::client::conn::http1::SendRequest<
        http_body_util::combinators::BoxBody<hyper::body::Bytes, eyre::Error>,
    >;
    type Error = eyre::Error;

    fn connect(&self) -> impl Future<Output = Result<Self::Connection, Self::Error>> + Send {
        Box::pin(async move { self.connect().await })
    }

    fn is_valid(
        &self,
        conn: &mut Self::Connection,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        Box::pin(async {
            if conn.is_closed() {
                return Err(eyre::anyhow!("connection is closed"));
            }

            Ok(())
        })
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        conn.is_closed()
    }
}
