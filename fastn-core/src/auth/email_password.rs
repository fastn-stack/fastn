pub async fn create_user(
    req: &fastn_core::http::Request,
    db_pool: &fastn_core::db::PgPool,
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

    let user_payload: UserPayload = req.json()?;

    let mut conn = db_pool
        .get()
        .await
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    let count: i64 = fastn_core::schema::fastn_user::table
        .filter(fastn_core::schema::fastn_user::username.eq(&user_payload.email))
        .select(diesel::dsl::count(fastn_core::schema::fastn_user::id))
        .first(&mut conn)
        .await?;

    if count > 0 {
        return fastn_core::http::api_error(
            "username taken.",
            fastn_core::http::StatusCode::BAD_REQUEST.into(),
        );
    }

    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);

    let argon2 = argon2::Argon2::default();

    let hashed_password =
        argon2::PasswordHasher::hash_password(&argon2, user_payload.password.as_bytes(), &salt)
            .map_err(|e| fastn_core::Error::generic(format!("error in hashing password: {e}")))?
            .to_string();

    let user = diesel::insert_into(fastn_core::schema::fastn_user::table)
        .values((
            fastn_core::schema::fastn_user::username.eq(user_payload.username),
            fastn_core::schema::fastn_user::password.eq(hashed_password),
            fastn_core::schema::fastn_user::name.eq(user_payload.name),
        ))
        .returning(fastn_core::auth::FastnUser::as_returning())
        .get_result(&mut conn)
        .await?;

    let affected = diesel::insert_into(fastn_core::schema::fastn_user_email::table)
        .values((
            fastn_core::schema::fastn_user_email::user_id.eq(user.id),
            fastn_core::schema::fastn_user_email::email
                .eq(fastn_core::utils::citext(user_payload.email.as_str())),
            fastn_core::schema::fastn_user_email::verified.eq(false),
            fastn_core::schema::fastn_user_email::primary.eq(true),
        ))
        .execute(&mut conn)
        .await?;

    tracing::info!("fastn_user created. user_id: {}", &user.id);

    if affected > 0 {
        tracing::info!("fastn_user_email created");
    }

    fastn_core::http::api_ok(user)
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

    let payload: Payload = req.json()?;

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
        return fastn_core::http::api_error(
            "invalid payload",
            fastn_core::http::StatusCode::BAD_REQUEST.into(),
        );
    }

    let user = user.expect("already checked for None");

    // OAuth users don't have password
    if user.password.is_empty() {
        // TODO: create feature to ask if the user wants to convert their account to an email
        // password
        // or should we redirect them to the oauth provider they used last time?
        // redirecting them will require saving the method they used to login which de don't atm
        return fastn_core::http::api_error(
            "use available oauth providers to sign in",
            fastn_core::http::StatusCode::BAD_REQUEST.into(),
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
        return fastn_core::http::api_error(
            "incorrect username/password",
            fastn_core::http::StatusCode::BAD_REQUEST.into(),
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
    fastn_core::auth::set_session_cookie_and_end_response(req, session_id, next).await
}
