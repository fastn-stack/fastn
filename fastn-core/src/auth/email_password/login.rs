pub(crate) async fn login(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    // [INFO] logging
    req.log(
        fastn_core::log::EventKind::Auth(fastn_core::log::AuthEvent::Login),
        fastn_core::log::OutcomeKind::Info,
        file!(),
        line!(),
    );

    if req_config.request.method() != "POST" {
        // TODO: if user is logged in redirect to next

        let main = fastn_core::Document {
            package_name: req_config.config.package.name.clone(),
            id: fastn_core::auth::Route::Login.to_string(),
            content: fastn_core::auth::email_password::login_ftd().to_string(),
            parent_path: fastn_ds::Path::new("/"),
        };

        return match fastn_core::package::package_doc::read_ftd(
            req_config, &main, "/", false, false,
        )
        .await
        {
            Ok(resp) => {
                // [SUCCESS] logging
                req.log(
                    fastn_core::log::EventKind::Auth(fastn_core::log::AuthEvent::Login),
                    fastn_core::log::OutcomeKind::Success(fastn_core::log::Outcome::Default),
                    file!(),
                    line!(),
                );

                Ok(resp.into())
            }
            Err(e) => {
                // [ERROR] logging (read_ftd)
                let log_err_message = format!("read_ftd: {:?}", &e);
                req.log(
                    fastn_core::log::EventKind::Auth(fastn_core::log::AuthEvent::Login),
                    fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                    file!(),
                    line!(),
                );

                Err(e)
            }
        };
    }

    #[derive(serde::Deserialize, validator::Validate, Debug)]
    struct Payload {
        username: String,
        password: String,
    }

    let payload = match req_config.request.json::<Payload>() {
        Ok(payload) => payload,
        Err(e) => {
            // [ERROR] logging (payload)
            let log_err_message = format!("payload: invalid payload {:?}", &e);
            req.log(
                fastn_core::log::EventKind::Auth(fastn_core::log::AuthEvent::Login),
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );

            return fastn_core::http::user_err(
                vec![("payload".into(), vec![format!("invalid payload: {:?}", e)])],
                fastn_core::http::StatusCode::OK,
            );
        }
    };

    let mut errors: Vec<(String, Vec<String>)> = Vec::new();

    if payload.username.is_empty() {
        errors.push(("username".into(), vec!["username/email is required".into()]));
    }

    if payload.password.is_empty() {
        errors.push(("password".into(), vec!["password is required".into()]));
    }

    if !errors.is_empty() {
        let err_message = errors
            .iter()
            .flat_map(|(field, messages)| {
                messages
                    .iter()
                    .map(|message| format!("{}: {}", field.as_str(), message.as_str()))
            })
            .collect::<Vec<String>>()
            .join(", ");

        let log_err_message = format!("User err: {}", err_message);
        // [ERROR] logging (user)
        req.log(
            fastn_core::log::EventKind::Auth(fastn_core::log::AuthEvent::Login),
            fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
            file!(),
            line!(),
        );

        return fastn_core::http::user_err(errors, fastn_core::http::StatusCode::OK);
    }

    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            let err_message = format!("Failed to get connection to db. {:?}", &e);
            let log_err_message = format!("Pool error: {}", err_message.as_str());

            // [ERROR] logging (pool error)
            req.log(
                fastn_core::log::EventKind::Auth(fastn_core::log::AuthEvent::Login),
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );

            return Err(fastn_core::Error::DatabaseError {
                message: err_message,
            });
        }
    };

    let query = fastn_core::schema::fastn_user::table
        .filter(fastn_core::schema::fastn_user::username.eq(&payload.username))
        .or_filter(
            fastn_core::schema::fastn_user::email.eq(fastn_core::utils::citext(&payload.username)),
        )
        .select(fastn_core::auth::FastnUser::as_select());

    let user: Option<fastn_core::auth::FastnUser> = query.first(&mut conn).await.optional()?;

    let user = match user {
        Some(user) => user,
        None => {
            let log_err_message = "User: Invalid email/username".to_string();

            // [ERROR] logging (user not found)
            req.log(
                fastn_core::log::EventKind::Auth(fastn_core::log::AuthEvent::Login),
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );

            return fastn_core::http::user_err(
                vec![("username".into(), vec!["invalid email/username".into()])],
                fastn_core::http::StatusCode::OK,
            );
        }
    };

    // OAuth users don't have password
    if user.password.is_empty() {
        // TODO: create feature to ask if the user wants to convert their account to an email
        // password
        // or should we redirect them to the oauth provider they used last time?
        // redirecting them will require saving the method they used to login which de don't atm

        // [ERROR] logging (user password is empty)
        let log_err_message = "User password: Is empty/blank".to_string();
        req.log(
            fastn_core::log::EventKind::Auth(fastn_core::log::AuthEvent::Login),
            fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
            file!(),
            line!(),
        );

        return fastn_core::http::user_err(
            vec![("password".into(), vec!["password can't be empty".into()])],
            fastn_core::http::StatusCode::OK,
        );
    }

    let parsed_hash = match argon2::PasswordHash::new(&user.password) {
        Ok(hash) => hash,
        Err(e) => {
            let err_message = format!("failed to parse hashed password: {e}");

            // [ERROR] logging (hashed password: parse error)
            let log_err_message = format!("hashed password: {}", err_message.as_str());
            req.log(
                fastn_core::log::EventKind::Auth(fastn_core::log::AuthEvent::Login),
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );

            return Err(fastn_core::Error::generic(err_message));
        }
    };

    let password_match = argon2::PasswordVerifier::verify_password(
        &argon2::Argon2::default(),
        payload.password.as_bytes(),
        &parsed_hash,
    );

    if password_match.is_err() {
        // [ERROR] logging (password: mismatch)
        let log_err_message = "password: incorrect password".to_string();
        req.log(
            fastn_core::log::EventKind::Auth(fastn_core::log::AuthEvent::Login),
            fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
            file!(),
            line!(),
        );

        return fastn_core::http::user_err(
            vec![(
                "password".into(),
                vec!["incorrect username/password".into()],
            )],
            fastn_core::http::StatusCode::OK,
        );
    }

    let now = chrono::Utc::now();

    // TODO: session should store device that was used to login (chrome desktop on windows)
    let session_id: i64 = match diesel::insert_into(fastn_core::schema::fastn_auth_session::table)
        .values((
            fastn_core::schema::fastn_auth_session::user_id.eq(&user.id),
            fastn_core::schema::fastn_auth_session::created_at.eq(now),
            fastn_core::schema::fastn_auth_session::updated_at.eq(now),
        ))
        .returning(fastn_core::schema::fastn_auth_session::id)
        .get_result(&mut conn)
        .await
    {
        Ok(id) => id,
        Err(e) => {
            // [ERROR] logging (session_id)
            let log_err_message = format!("session_id: {:?}", &e);
            req.log(
                fastn_core::log::EventKind::Auth(fastn_core::log::AuthEvent::Login),
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            return Err(e.into());
        }
    };

    tracing::info!("session created. session id: {}", &session_id);

    // client has to 'follow' this request
    // https://stackoverflow.com/a/39739894
    fastn_core::auth::set_session_cookie_and_redirect_to_next(
        &req_config.request,
        fastn_core::log::EventKind::Auth(fastn_core::log::AuthEvent::Login),
        &req_config.config.ds,
        session_id,
        next,
    )
    .await
}
