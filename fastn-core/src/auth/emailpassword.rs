use argon2::PasswordHasher;

// TODO: handle errors
pub async fn create_user(
    req: &fastn_core::http::Request,
) -> fastn_core::Result<fastn_core::http::Response> {
    if req.method() != "POST" {
        return Ok(fastn_core::not_found!("invalid route"));
    }

    let user: User = serde_json::from_slice(req.body())?;

    dbg!(&user);

    let client = db::get_client().await?;

    let res = client
        .query(
            "select count(id) from fastn_user where username = $1 or email = $2",
            &[&user.username, &user.email],
        )
        .await
        .unwrap();

    let user_count: i64 = res[0].get(0);

    if user_count > 0 {
        // TODO: use correct HTTP status code
        return fastn_core::http::api_error("user already exists");
    }

    // encrypt password
    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);

    let argon2 = argon2::Argon2::default();

    let hashed_password = argon2
        .hash_password(user.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let user_id = uuid::Uuid::new_v4();

    // TODO: email verification
    let affected = client
        .execute(
            "INSERT into fastn_user(id, email, username, password, name) values ($1, $2, $3, $4, $5)",
            &[&user_id, &user.email, &user.username, &hashed_password, &user.name],
        )
        .await
        .unwrap();

    dbg!(affected);

    fastn_core::http::api_ok(user)
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct User {
    username: String,
    email: String,
    name: String,
    password: String,
}

// TODO: this is borrowed from pg.rs
// pg.rs and this mod should use the same code
mod db {
    async fn create_pool() -> Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError> {
        let mut cfg = deadpool_postgres::Config::new();
        cfg.libpq_style_connection_string = match std::env::var("FASTN_DB_URL") {
            Ok(v) => Some(v),
            Err(_) => {
                fastn_core::warning!("FASTN_DB_URL is not set");
                return Err(deadpool_postgres::CreatePoolError::Config(
                    deadpool_postgres::ConfigError::ConnectionStringInvalid,
                ));
            }
        };
        cfg.manager = Some(deadpool_postgres::ManagerConfig {
            // TODO: make this configurable
            recycling_method: deadpool_postgres::RecyclingMethod::Verified,
        });
        let runtime = Some(deadpool_postgres::Runtime::Tokio1);

        if std::env::var("FASTN_PG_DANGER_DISABLE_SSL") == Ok("false".to_string()) {
            fastn_core::warning!(
            "FASTN_PG_DANGER_DISABLE_SSL is set to false, this is not recommended for production use",
        );
            cfg.ssl_mode = Some(deadpool_postgres::SslMode::Disable);
            return cfg.create_pool(runtime, tokio_postgres::NoTls);
        }

        let mut connector = native_tls::TlsConnector::builder();

        match std::env::var("FASTN_PG_SSL_MODE").as_deref() {
            Err(_) | Ok("require") => {
                cfg.ssl_mode = Some(deadpool_postgres::SslMode::Require);
            }
            Ok("prefer") => {
                fastn_core::warning!(
                "FASTN_PG_SSL_MODE is set to prefer, which roughly means \"I don't care about \
                encryption, but I wish to pay the overhead of encryption if the server supports it.\"\
                and is not recommended for production use",
            );
                cfg.ssl_mode = Some(deadpool_postgres::SslMode::Prefer);
            }
            Ok(v) => {
                // TODO: openssl also allows `verify-ca` and `verify-full` but native_tls does not
                fastn_core::warning!(
                "FASTN_PG_SSL_MODE is set to {}, which is invalid, only allowed values are prefer and require",
                v,
            );
                return Err(deadpool_postgres::CreatePoolError::Config(
                    deadpool_postgres::ConfigError::ConnectionStringInvalid,
                ));
            }
        }

        if std::env::var("FASTN_PG_DANGER_ALLOW_UNVERIFIED_CERTIFICATE") == Ok("true".to_string()) {
            fastn_core::warning!(
                "FASTN_PG_DANGER_ALLOW_UNVERIFIED_CERTIFICATE is set to true, this is not \
            recommended for production use",
            );
            connector.danger_accept_invalid_certs(true);
        }

        if let Ok(cert) = std::env::var("FASTN_PG_CERTIFICATE") {
            // TODO: This does not work with Heroku certificate.
            let cert = tokio::fs::read(cert).await.unwrap();
            // TODO: We should allow DER formatted certificates too, maybe based on file extension?
            let cert = native_tls::Certificate::from_pem(&cert).unwrap();
            connector.add_root_certificate(cert);
        }

        let tls = postgres_native_tls::MakeTlsConnector::new(connector.build().unwrap());
        cfg.create_pool(runtime, tls)
    }

    static POOL_RESULT: tokio::sync::OnceCell<
        Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError>,
    > = tokio::sync::OnceCell::const_new();

    static EXECUTE_QUERY_LOCK: once_cell::sync::Lazy<tokio::sync::Mutex<()>> =
        once_cell::sync::Lazy::new(|| tokio::sync::Mutex::new(()));

    async fn pool() -> &'static Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError>
    {
        POOL_RESULT.get_or_init(create_pool).await
    }

    pub async fn get_client() -> ftd::interpreter::Result<deadpool_postgres::Object> {
        let _lock = EXECUTE_QUERY_LOCK.lock().await;

        let client = pool().await.as_ref().unwrap().get().await.unwrap();

        Ok(client)
    }
}
