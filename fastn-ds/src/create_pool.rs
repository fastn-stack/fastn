// async fn create_pool(
//     req_config: &fastn_core::RequestConfig,
// ) -> Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError> {
//     let mut cfg = deadpool_postgres::Config::new();
//     cfg.libpq_style_connection_string = match req_config.config.ds.env("FASTN_DB_URL").await {
//         Ok(v) => Some(v),
//         Err(_) => {
//             fastn_core::warning!("FASTN_DB_URL is not set");
//             return Err(deadpool_postgres::CreatePoolError::Config(
//                 deadpool_postgres::ConfigError::ConnectionStringInvalid,
//             ));
//         }
//     };
//     cfg.manager = Some(deadpool_postgres::ManagerConfig {
//         // TODO: make this configurable
//         recycling_method: deadpool_postgres::RecyclingMethod::Verified,
//     });
//     let runtime = Some(deadpool_postgres::Runtime::Tokio1);
//
//     if let Ok(true) = req_config
//         .config
//         .ds
//         .env_bool("FASTN_PG_DANGER_ENABLE_SSL", false)
//         .await
//     {
//         fastn_core::warning!(
//             "FASTN_PG_DANGER_DISABLE_SSL is set to false, this is not recommended for production use",
//         );
//         cfg.ssl_mode = Some(deadpool_postgres::SslMode::Disable);
//         return cfg.create_pool(runtime, tokio_postgres::NoTls);
//     }
//
//     let mut connector = native_tls::TlsConnector::builder();
//
//     match req_config
//         .config
//         .ds
//         .env("FASTN_PG_SSL_MODE")
//         .await
//         .as_deref()
//     {
//         Err(_) | Ok("require") => {
//             cfg.ssl_mode = Some(deadpool_postgres::SslMode::Require);
//         }
//         Ok("prefer") => {
//             fastn_core::warning!(
//                 "FASTN_PG_SSL_MODE is set to prefer, which roughly means \"I don't care about \
//                 encryption, but I wish to pay the overhead of encryption if the server supports it.\"\
//                 and is not recommended for production use",
//             );
//             cfg.ssl_mode = Some(deadpool_postgres::SslMode::Prefer);
//         }
//         Ok(v) => {
//             // TODO: openssl also allows `verify-ca` and `verify-full` but native_tls does not
//             fastn_core::warning!(
//                 "FASTN_PG_SSL_MODE is set to {}, which is invalid, only allowed values are prefer and require",
//                 v,
//             );
//             return Err(deadpool_postgres::CreatePoolError::Config(
//                 deadpool_postgres::ConfigError::ConnectionStringInvalid,
//             ));
//         }
//     }
//
//     if let Ok(true) = req_config
//         .config
//         .ds
//         .env_bool("FASTN_PG_DANGER_ALLOW_UNVERIFIED_CERTIFICATE", false)
//         .await
//     {
//         fastn_core::warning!(
//             "FASTN_PG_DANGER_ALLOW_UNVERIFIED_CERTIFICATE is set to true, this is not \
//             recommended for production use",
//         );
//         connector.danger_accept_invalid_certs(true);
//     }
//
//     if let Ok(cert) = req_config.config.ds.env("FASTN_PG_CERTIFICATE").await {
//         // TODO: This does not work with Heroku certificate.
//         let cert = req_config
//             .config
//             .ds
//             .read_content(&fastn_ds::Path::new(cert))
//             .await
//             .unwrap();
//         // TODO: We should allow DER formatted certificates too, maybe based on file extension?
//         let cert = native_tls::Certificate::from_pem(&cert).unwrap();
//         connector.add_root_certificate(cert);
//     }
//
//     let tls = postgres_native_tls::MakeTlsConnector::new(connector.build().unwrap());
//     cfg.create_pool(runtime, tls)
// }

pub async fn create_pool(
    db_url: &str,
    _is_default: bool,
) -> Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError> {
    deadpool_postgres::Config {
        url: Some(db_url.to_string()),
        ..Default::default()
    }
    .create_pool(
        Some(deadpool_postgres::Runtime::Tokio1),
        tokio_postgres::NoTls,
    )
}
