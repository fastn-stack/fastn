pub(crate) async fn create_user(
    req: &fastn_core::http::Request,
    config: &fastn_core::Config,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

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

    let user_payload = req.json::<UserPayload>();

    if let Err(e) = user_payload {
        return fastn_core::http::user_err(
            vec![("payload", format!("invalid payload: {:?}", e).as_str())],
            fastn_core::http::StatusCode::BAD_REQUEST,
        );
    }

    let user_payload = user_payload.unwrap();

    let mut conn = db_pool
        .get()
        .await
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    let username_check: i64 = fastn_core::schema::fastn_user::table
        .filter(fastn_core::schema::fastn_user::username.eq(&user_payload.username))
        .select(diesel::dsl::count(fastn_core::schema::fastn_user::id))
        .first(&mut conn)
        .await?;

    let email_check: Result<i64, diesel::result::Error> =
        fastn_core::schema::fastn_user_email::table
            .filter(
                fastn_core::schema::fastn_user_email::email
                    .eq(fastn_core::utils::citext(user_payload.email.as_str())),
            )
            .select(diesel::dsl::count(fastn_core::schema::fastn_user_email::id))
            .first(&mut conn)
            .await;

    if let Err(_e) = email_check {
        return fastn_core::http::user_err(
            vec![("email", "invalid email")],
            fastn_core::http::StatusCode::BAD_REQUEST,
        );
    }

    let email_check = email_check.expect("expected email_check to be Some");

    if username_check > 0 {
        return fastn_core::http::user_err(
            vec![("username", "username already taken")],
            fastn_core::http::StatusCode::BAD_REQUEST,
        );
    }

    if email_check > 0 {
        return fastn_core::http::user_err(
            vec![("email", "email already taken")],
            fastn_core::http::StatusCode::BAD_REQUEST,
        );
    }

    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);

    let argon2 = argon2::Argon2::default();

    let hashed_password =
        argon2::PasswordHasher::hash_password(&argon2, user_payload.password.as_bytes(), &salt)
            .map_err(|e| fastn_core::Error::generic(format!("error in hashing password: {e}")))?
            .to_string();

    let save_user_email_transaction = conn
        .build_transaction()
        .run(|c| {
            Box::pin(async move {
                let user = diesel::insert_into(fastn_core::schema::fastn_user::table)
                    .values((
                        fastn_core::schema::fastn_user::username.eq(user_payload.username),
                        fastn_core::schema::fastn_user::password.eq(hashed_password),
                        fastn_core::schema::fastn_user::name.eq(user_payload.name),
                    ))
                    .returning(fastn_core::auth::FastnUser::as_returning())
                    .get_result(c)
                    .await?;

                tracing::info!("fastn_user created. user_id: {}", &user.id);

                let email: fastn_core::utils::CiString =
                    diesel::insert_into(fastn_core::schema::fastn_user_email::table)
                        .values((
                            fastn_core::schema::fastn_user_email::user_id.eq(user.id),
                            fastn_core::schema::fastn_user_email::email
                                .eq(fastn_core::utils::citext(user_payload.email.as_str())),
                            fastn_core::schema::fastn_user_email::verified.eq(false),
                            fastn_core::schema::fastn_user_email::primary.eq(true),
                        ))
                        .returning(fastn_core::schema::fastn_user_email::email)
                        .get_result(c)
                        .await?;

                Ok::<
                    (fastn_core::auth::FastnUser, fastn_core::utils::CiString),
                    diesel::result::Error,
                >((user, email))
            })
        })
        .await;

    if let Err(e) = save_user_email_transaction {
        return fastn_core::http::user_err(
            vec![
                ("email", "invalid email"),
                ("detail", format!("{e}").as_str()),
            ],
            fastn_core::http::StatusCode::BAD_REQUEST,
        );
    }

    let (user, email) = save_user_email_transaction.expect("expected transaction to yield Some");

    tracing::info!("fastn_user email inserted");

    let conf_link = create_and_send_confirmation_email(email.0.to_string(), db_pool, req).await?;

    let resp_body = serde_json::json!({
        "user": user,
        "redirect": redirect_url_from_next(req, next),
    });


    let mut resp = actix_web::HttpResponse::Ok();

    if config.test_command_running {
        resp
            .insert_header(("X-Fastn-Test", "true"))
            .insert_header(("X-Fastn-Test-Email-Confirmation-Link", conf_link));
    }

    Ok(resp.json(resp_body))
}

pub(crate) async fn login(
    req: &fastn_core::http::Request,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    if req.method() != "POST" {
        return Ok(fastn_core::not_found!("invalid route"));
    }

    #[derive(serde::Deserialize, Debug)]
    struct Payload {
        // TODO: add support for login using email
        username: String,
        password: String,
    }

    let payload = req.json::<Payload>();

    if let Err(e) = payload {
        return fastn_core::http::user_err(
            vec![("payload", format!("invalid payload: {:?}", e).as_str())],
            fastn_core::http::StatusCode::BAD_REQUEST,
        );
    }

    let payload = payload.unwrap();

    let mut conn = db_pool
        .get()
        .await
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    let user: Option<fastn_core::auth::FastnUser> = fastn_core::schema::fastn_user::table
        .filter(fastn_core::schema::fastn_user::username.eq(&payload.username))
        .select(fastn_core::auth::FastnUser::as_select())
        .first(&mut conn)
        .await
        .optional()?;

    if user.is_none() {
        return fastn_core::http::user_err(
            vec![("username", "invalid username")],
            fastn_core::http::StatusCode::BAD_REQUEST,
        );
    }

    let user = user.expect("expected user to be Some");

    // OAuth users don't have password
    if user.password.is_empty() {
        // TODO: create feature to ask if the user wants to convert their account to an email
        // password
        // or should we redirect them to the oauth provider they used last time?
        // redirecting them will require saving the method they used to login which de don't atm
        return fastn_core::http::user_err(
            vec![("username", "invalid username")],
            fastn_core::http::StatusCode::BAD_REQUEST,
        );
    }

    let parsed_hash = argon2::PasswordHash::new(&user.password)
        .map_err(|e| fastn_core::Error::generic(format!("failed to parse hashed password: {e}")))?;

    let password_match = argon2::PasswordVerifier::verify_password(
        &argon2::Argon2::default(),
        payload.password.as_bytes(),
        &parsed_hash,
    );

    if password_match.is_err() {
        return fastn_core::http::user_err(
            vec![("password", "incorrect username/password")],
            fastn_core::http::StatusCode::BAD_REQUEST,
        );
    }

    // TODO: session should store device that was used to login (chrome desktop on windows)
    let session_id: i32 = diesel::insert_into(fastn_core::schema::fastn_session::table)
        .values((fastn_core::schema::fastn_session::user_id.eq(&user.id),))
        .returning(fastn_core::schema::fastn_session::id)
        .get_result(&mut conn)
        .await?;

    tracing::info!("session created. session id: {}", &session_id);

    // client has to 'follow' this request
    // https://stackoverflow.com/a/39739894
    fastn_core::auth::set_session_cookie_and_redirect_to_next(req, session_id, next).await
}

pub(crate) async fn confirm_email(
    req: &fastn_core::http::Request,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let code = req.query().get("code");

    if code.is_none() {
        return fastn_core::http::api_error(
            "Bad Request",
            fastn_core::http::StatusCode::BAD_REQUEST.into(),
        );
    }

    let code = match code.unwrap() {
        serde_json::Value::String(c) => c,
        _ => {
            return fastn_core::http::api_error(
                "Bad Request",
                fastn_core::http::StatusCode::BAD_REQUEST.into(),
            );
        }
    };

    let mut conn = db_pool
        .get()
        .await
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    let conf_data: Option<(i32, chrono::DateTime<chrono::Utc>)> =
        fastn_core::schema::fastn_email_confirmation::table
            .select((
                fastn_core::schema::fastn_email_confirmation::email_id,
                fastn_core::schema::fastn_email_confirmation::sent_at,
            ))
            .filter(fastn_core::schema::fastn_email_confirmation::key.eq(code))
            .first(&mut conn)
            .await
            .optional()?;

    if conf_data.is_none() {
        return fastn_core::http::api_error(
            "Bad Request",
            fastn_core::http::StatusCode::BAD_REQUEST.into(),
        );
    }

    let (email_id, sent_at) = conf_data.unwrap();

    if key_expired(sent_at) {
        // TODO: this redirect route should be configurable
        return Ok(fastn_core::http::redirect_with_code(
            format!(
                "{}://{}/-/auth/resend-confirmation-email/",
                req.connection_info.scheme(),
                req.connection_info.host(),
            ),
            302,
        ));
    }

    let affected = diesel::update(fastn_core::schema::fastn_user_email::table)
        .set(fastn_core::schema::fastn_user_email::verified.eq(true))
        .filter(fastn_core::schema::fastn_user_email::id.eq(email_id))
        .execute(&mut conn)
        .await?;

    tracing::info!("verified {} email", affected);

    Ok(fastn_core::http::redirect_with_code(
        redirect_url_from_next(req, next),
        302,
    ))
}

pub(crate) async fn resend_email(
    req: &fastn_core::http::Request,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    // TODO: should be able to use username for this too
    let email = req.query().get("email");

    if email.is_none() {
        return fastn_core::http::api_error(
            "Bad Request",
            fastn_core::http::StatusCode::BAD_REQUEST.into(),
        );
    }

    let email = match email.unwrap() {
        serde_json::Value::String(c) => c.to_owned(),
        _ => {
            return fastn_core::http::api_error(
                "Bad Request",
                fastn_core::http::StatusCode::BAD_REQUEST.into(),
            );
        }
    };

    create_and_send_confirmation_email(email, db_pool, req).await?;

    // TODO: there's no GET /-/auth/login/ yet
    // the client will have to create one for now
    // this path should be configuratble too
    Ok(fastn_core::http::redirect_with_code(
        redirect_url_from_next(req, next),
        302,
    ))
}

async fn create_and_send_confirmation_email(
    email: String,
    db_pool: &fastn_core::db::PgPool,
    req: &fastn_core::http::Request,
) -> fastn_core::Result<String> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let key = generate_key(64);

    let mut conn = db_pool
        .get()
        .await
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    let (email_id, user_id): (i32, i32) = fastn_core::schema::fastn_user_email::table
        .select((
            fastn_core::schema::fastn_user_email::id,
            fastn_core::schema::fastn_user_email::user_id,
        ))
        .filter(
            fastn_core::schema::fastn_user_email::email
                .eq(fastn_core::utils::citext(email.as_str())),
        )
        .first(&mut conn)
        .await?;

    let stored_key: String =
        diesel::insert_into(fastn_core::schema::fastn_email_confirmation::table)
            .values((
                fastn_core::schema::fastn_email_confirmation::email_id.eq(email_id),
                fastn_core::schema::fastn_email_confirmation::sent_at
                    .eq(chrono::offset::Utc::now()),
                fastn_core::schema::fastn_email_confirmation::key.eq(key),
            ))
            .returning(fastn_core::schema::fastn_email_confirmation::key)
            .get_result(&mut conn)
            .await?;

    let confirmation_link = confirmation_link(req, stored_key);

    let mailer = fastn_core::mail::Mailer::from_env();

    if mailer.is_err() {
        return Err(fastn_core::Error::generic(
            "Failed to create mailer from env. Creating mailer requires the following environment variables: \
                \tFASTN_SMTP_USERNAME \
                \tFASTN_SMTP_PASSWORD \
                \tFASTN_SMTP_HOST \
                \tFASTN_SMTP_SENDER_EMAIL \
                \tFASTN_SMTP_SENDER_NAME",
        ));
    }

    let mut mailer = mailer.unwrap();

    if let Ok(debug_mode) = std::env::var("DEBUG") {
        if debug_mode == "true" {
            mailer.mock();
        }
    }

    let name: String = fastn_core::schema::fastn_user::table
        .select(fastn_core::schema::fastn_user::name)
        .filter(fastn_core::schema::fastn_user::id.eq(user_id))
        .first(&mut conn)
        .await?;

    mailer
        .send_raw(
            format!("{} <{}>", name, email)
                .parse::<lettre::message::Mailbox>()
                .unwrap(),
            "Verify your email",
            confirmation_mail_body(&confirmation_link),
        )
        .await
        .map_err(|e| fastn_core::Error::generic(format!("failed to send email: {e}")))?;

    Ok(confirmation_link)
}

/// check if it has been 3 days since the verification code was sent
/// can be configured using EMAIL_CONFIRMATION_EXPIRE_DAYS
fn key_expired(sent_at: chrono::DateTime<chrono::Utc>) -> bool {
    let expiry_limit_in_days: u64 = std::env::var("EMAIL_CONFIRMATION_EXPIRE_DAYS")
        .unwrap_or("3".to_string())
        .parse()
        .expect("EMAIL_CONFIRMATION_EXPIRE_DAYS should be a number");

    sent_at
        .checked_add_days(chrono::Days::new(expiry_limit_in_days))
        .unwrap()
        <= chrono::offset::Utc::now()
}

fn confirmation_mail_body(link: &str) -> String {
    format!("Use this link to verify your email: {link}")
}

fn generate_key(length: usize) -> String {
    let mut rng = rand::thread_rng();
    rand::distributions::DistString::sample_string(
        &rand::distributions::Alphanumeric,
        &mut rng,
        length,
    )
}

fn confirmation_link(req: &fastn_core::http::Request, key: String) -> String {
    format!(
        "{}://{}/-/auth/confirm-email/?code={key}",
        req.connection_info.scheme(),
        req.connection_info.host()
    )
}

fn redirect_url_from_next(req: &fastn_core::http::Request, next: String) -> String {
    format!(
        "{}://{}{}",
        req.connection_info.scheme(),
        req.connection_info.host(),
        next,
    )
}
