async fn create_pool() -> Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError> {
    let mut cfg = deadpool_postgres::Config::new();
    cfg.dbname = Some("deadpool".to_string());
    cfg.manager = Some(deadpool_postgres::ManagerConfig {
        recycling_method: deadpool_postgres::RecyclingMethod::Verified,
    });
    let runtime = Some(deadpool_postgres::Runtime::Tokio1);
    match std::env::var("FASTN_PG_CERTIFICATE") {
        Ok(cert) => {
            let cert = tokio::fs::read(cert).await.unwrap();
            let cert = native_tls::Certificate::from_pem(&cert).unwrap();
            let connector = native_tls::TlsConnector::builder()
                .add_root_certificate(cert)
                .build()
                .unwrap();
            let tls = postgres_native_tls::MakeTlsConnector::new(connector);
            cfg.create_pool(runtime, tls)
        }
        _ => cfg.create_pool(runtime, tokio_postgres::NoTls),
    }
}

static POOL_RESULT: tokio::sync::OnceCell<
    Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError>,
> = tokio::sync::OnceCell::const_new();

async fn pool() -> &'static Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError> {
    POOL_RESULT.get_or_init(create_pool).await
}

pub async fn process<'a>(
    _value: ftd::ast::VariableValue,
    _kind: ftd::interpreter::Kind,
    _doc: &ftd::interpreter::TDoc<'a>,
    _config: &fastn_core::Config,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let _pool = pool().await.as_ref().unwrap();

    todo!()
}
