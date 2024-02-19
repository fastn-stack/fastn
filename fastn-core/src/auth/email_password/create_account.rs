use crate::auth::email_password::{create_account_ftd, create_and_send_confirmation_email};

#[derive(serde::Deserialize, serde::Serialize, validator::Validate, Debug)]
struct UserPayload {
    #[validate(length(min = 4, message = "username must be at least 4 character long"))]
    username: String,
    #[validate(email(message = "invalid email format"))]
    email: String,
    #[validate(length(min = 1, message = "name must be at least 1 character long"))]
    name: String,
    #[validate(custom(
        function = "fastn_core::auth::validator::validate_strong_password",
        arg = "(&'v_a str, &'v_a str, &'v_a str)"
    ))]
    password: String,
}

pub(crate) async fn create_account(
    req_config: &mut fastn_core::RequestConfig,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use validator::ValidateArgs;

    let now = chrono::Utc::now();

    if req_config.request.method() != "POST" {
        // TODO: if user is logged in redirect to next

        let main = fastn_core::Document {
            package_name: req_config.config.package.name.clone(),
            id: "/-/auth/create-account/".to_string(),
            content: create_account_ftd().to_string(),
            parent_path: fastn_ds::Path::new("/"),
        };

        let resp = fastn_core::package::package_doc::read_ftd(req_config, &main, "/", false, false)
            .await?;

        return Ok(resp.into());
    }

    let user_payload = match req_config.request.json::<UserPayload>() {
        Ok(p) => p,
        Err(e) => return fastn_core::http::user_err(
            vec![("payload".into(), vec![format!("Invalid payload. Required the request body to contain json. Original error: {:?}", e)])],
            fastn_core::http::StatusCode::OK,
        )
    };

    if let Err(e) = user_payload.validate_args((
        user_payload.username.as_str(),
        user_payload.email.as_str(),
        user_payload.name.as_str(),
    )) {
        return fastn_core::http::validation_error_to_user_err(e);
    }

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

    if username_check > 0 {
        return fastn_core::http::user_err(
            vec![("username".into(), vec!["username already taken".into()])],
            fastn_core::http::StatusCode::OK,
        );
    }

    let email_check: i64 = fastn_core::schema::fastn_user::table
        .filter(
            fastn_core::schema::fastn_user::email
                .eq(fastn_core::utils::citext(&user_payload.email)),
        )
        .select(diesel::dsl::count(fastn_core::schema::fastn_user::id))
        .first(&mut conn)
        .await?;

    if email_check > 0 {
        return fastn_core::http::user_err(
            vec![("email".into(), vec!["email already taken".into()])],
            fastn_core::http::StatusCode::OK,
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
                        fastn_core::schema::fastn_user::email
                            .eq(fastn_core::utils::citext(&user_payload.email)),
                        fastn_core::schema::fastn_user::created_at.eq(now),
                        fastn_core::schema::fastn_user::updated_at.eq(now),
                    ))
                    .returning(fastn_core::auth::FastnUser::as_returning())
                    .get_result(c)
                    .await?;

                // just for record keeping
                // we do not use `fastn_user_email` for auth at all
                diesel::insert_into(fastn_core::schema::fastn_user_email::table)
                    .values((
                        fastn_core::schema::fastn_user_email::user_id.eq(user.id),
                        fastn_core::schema::fastn_user_email::email
                            .eq(fastn_core::utils::citext(user_payload.email.as_str())),
                        fastn_core::schema::fastn_user_email::verified.eq(false),
                        fastn_core::schema::fastn_user_email::primary.eq(true),
                        fastn_core::schema::fastn_user_email::created_at.eq(now),
                        fastn_core::schema::fastn_user_email::updated_at.eq(now),
                    ))
                    .returning(fastn_core::schema::fastn_user_email::email)
                    .execute(c)
                    .await?;

                tracing::info!("fastn_user created. user_id: {}", &user.id);
                Ok::<fastn_core::auth::FastnUser, diesel::result::Error>(user)
            })
        })
        .await;

    if let Err(e) = save_user_email_transaction {
        return fastn_core::http::user_err(
            vec![
                ("email".into(), vec!["invalid email".into()]),
                ("detail".into(), vec![format!("{e}")]),
            ],
            fastn_core::http::StatusCode::OK,
        );
    }

    let user = save_user_email_transaction.expect("expected transaction to yield Some");

    tracing::info!("fastn_user email inserted");

    let (conf_link, session_id) = create_and_send_confirmation_email(
        user.email.0.to_string(),
        db_pool,
        req_config,
        next.clone(),
    )
    .await?;

    // email is not enabled, we should log conf link assuming dev mode
    if !req_config
        .config
        .ds
        .env_bool("FASTN_ENABLE_EMAIL", true)
        .await
    {
        println!("CONFIRMATION LINK: {}", conf_link);
    }

    fastn_core::auth::set_session_cookie_and_redirect_to_next(
        &req_config.request,
        &req_config.config.ds,
        session_id,
        next,
    )
    .await
}