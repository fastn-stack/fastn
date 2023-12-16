use argon2::{Argon2, PasswordHasher, PasswordVerifier};

#[derive(Debug)]
pub struct Token {
    pub session_id: uuid::Uuid,
    pub token: uuid::Uuid,
    pub provider: String,
}

impl Token {
    fn from_row(row: &tokio_postgres::Row) -> Self {
        Token {
            session_id: row.get("session_id"),
            token: row.get("token"),
            provider: row.get("provider"),
        }
    }
}

// TODO: handle errors
pub async fn create_user(
    req: &fastn_core::http::Request,
) -> fastn_core::Result<fastn_core::http::Response> {
    if req.method() != "POST" {
        return Ok(fastn_core::not_found!("invalid route"));
    }

    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    struct UserPayload {
        username: String,
        email: String,
        name: String,
        password: String,
    }

    let user: UserPayload = serde_json::from_slice(req.body())?;

    tracing::info!("user payload: {:?}", &user);

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
    // TODO: password can be null for oauth users. if it's null they should not login using
    // emailpassword flow
    let user = upsert_user(
        &user_id,
        user.email.as_str(),
        user.username.as_str(),
        user.name.as_str(),
        hashed_password.as_str(),
    )
    .await?;

    tracing::info!("fastn_user created. user: {:?}", &user);

    fastn_core::http::api_ok(user)
}

pub(crate) async fn login(
    req: &crate::http::Request,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    if req.method() != "POST" {
        return Ok(fastn_core::not_found!("invalid route"));
    }

    #[derive(serde::Deserialize, Debug)]
    struct Payload {
        // TODO: add email/username
        email: String,
        password: String,
    }

    let body: Payload = serde_json::from_slice(req.body())?;

    tracing::info!("request payload: {:?}", body);

    let client = db::get_client().await?;

    let user = client
        .query(
            "select id, username, email, name, password from fastn_user where email = $1 limit 1",
            &[&body.email],
        )
        .await
        .unwrap_or(vec![]);

    if user.is_empty() {
        return fastn_core::http::api_error("invalid payload");
    }

    let user = user.first().expect("vec of length must have first");

    let password: String = user.get("password");
    let user_id: uuid::Uuid = user.get("id");

    let parsed_hash = argon2::PasswordHash::new(&password).unwrap();

    let password_match = Argon2::default().verify_password(body.password.as_bytes(), &parsed_hash);

    if password_match.is_err() {
        return fastn_core::http::api_error("invalid payload");
    }

    let session_id = uuid::Uuid::new_v4();

    let affected = create_session(&session_id, &user_id).await.unwrap_or(0);

    tracing::info!("session created. Rows affected: {}", &affected);

    // client has to 'follow' this request
    // https://stackoverflow.com/a/39739894
    fastn_core::auth::set_session_cookie_and_end_response(req, session_id, next).await
}

/// insert user if it not exists in the database returning the rows affected
/// email and username is unique in fastn_user table
pub async fn upsert_user(
    user_id: &uuid::Uuid,
    email: &str,
    username: &str,
    name: &str,
    hashed_password: &str,
) -> fastn_core::Result<fastn_core::auth::FastnUser> {
    let client = db::get_client().await?;

    let affected = client
        .execute(
            // TODO: what if user manually signups with email test@test then later logins using
            // github with the same email but the username are differente in both cases?
            // we should ask one time if they want to update their username or keep using existing
            // one?
            "insert into fastn_user(id, email, username, password, name) values ($1, $2, $3, $4, $5) on conflict do nothing",
            &[&user_id, &email, &username, &hashed_password, &name],
        )
        .await
        .unwrap();

    tracing::info!("upsert user affected rows: {affected}");

    let rows = client
        .query(
            "select id, username, name, email \
                    from fastn_user \
                    where email = $1",
            &[&email],
        )
        .await
        .unwrap_or(vec![]);

    if rows.is_empty() {
        return fastn_core::error::Error::generic_err("no session found for the given session_id");
    }

    Ok(fastn_core::auth::FastnUser::from_row(
        rows.first().expect("vec of length 1 must have first()"),
    ))
}

// insert a row in fastn_session table
pub async fn create_session(
    session_id: &uuid::Uuid,
    user_id: &uuid::Uuid,
) -> fastn_core::Result<u64> {
    let client = db::get_client().await?;

    Ok(client
        .execute(
            "insert into fastn_session(id, user_id) values ($1, $2)",
            &[&session_id, &user_id],
        )
        .await
        .unwrap())
}

pub async fn destroy_session(session_id: uuid::Uuid) -> fastn_core::Result<u64> {
    let client = db::get_client().await?;

    Ok(client
        .execute("delete from fastn_session where id = $1", &[&session_id])
        .await
        .unwrap())
}

pub async fn insert_oauth_token(
    session_id: &uuid::Uuid,
    token: &str,
    provider: fastn_core::auth::AuthProviders,
) -> fastn_core::Result<u64> {
    let client = fastn_core::auth::emailpassword::db::get_client().await?;

    let id = uuid::Uuid::new_v4();

    Ok(client
        .execute(
            "insert into fastn_oauthtoken(id, session_id, token, provider) values ($1, $2, $3, $4)",
            &[&id, &session_id, &token, &provider.as_str()],
        )
        .await
        .unwrap())
}

pub async fn get_token_from_db(
    session_id: &uuid::Uuid,
    provider: &str,
) -> fastn_core::Result<Token> {
    let client = db::get_client().await?;

    let token = client
        .query(
            "select session_id, token, provider from fastn_oauthtoken where session_id = $1 and provider = $2",
            &[&session_id, &provider],
        )
        .await
        .unwrap_or(vec![]);

    if token.is_empty() {
        return fastn_core::error::Error::generic_err("no token exists");
    }

    let token = token.first().expect("vec of length must have first");

    Ok(Token::from_row(token))
}

pub async fn get_user_from_session(
    session_id: &uuid::Uuid,
) -> fastn_core::Result<fastn_core::auth::FastnUser> {
    let client = db::get_client().await?;

    let rows = client
        .query(
            "select id, username, name, email \
                    from fastn_user \
                    where id = (select user_id from fastn_session where id = $1)",
            &[&session_id],
        )
        .await
        .unwrap_or(vec![]);

    if rows.is_empty() {
        return fastn_core::error::Error::generic_err("no session found for the given session_id");
    }

    Ok(fastn_core::auth::FastnUser::from_row(
        rows.first().expect("vec of length 1 must have first()"),
    ))
}

// TODO: this is borrowed from pg.rs
// pg.rs and this mod should use the same code
pub mod db {
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
