pub(crate) async fn login(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

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
                // [SUCCESS] logging: GET
                let log_success_message = "login: GET".to_string();
                req.log(
                    "login",
                    fastn_core::log::OutcomeKind::Success(
                        fastn_core::log::SuccessOutcome::Descriptive(log_success_message),
                    ),
                    file!(),
                    line!(),
                );

                Ok(resp.into())
            }
            Err(e) => {
                // [ERROR] logging (server-error: ReadFTDError)
                let err_message = format!("{:?}", &e);
                req.log(
                    "login",
                    fastn_core::log::ServerErrorOutcome::ReadFTDError {
                        message: err_message,
                    }
                    .into_kind(),
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
            // [ERROR] logging (form-error: PayloadError)
            let err_message = format!("invalid payload: {:?}", &e);
            req.log(
                "login",
                fastn_core::log::FormErrorOutcome::PayloadError {
                    message: err_message.clone(),
                }
                .into_kind(),
                file!(),
                line!(),
            );

            return fastn_core::http::user_err(
                vec![("payload".into(), vec![err_message])],
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
        // [ERROR] logging (form-error: ValidationError)
        let err_message = errors
            .iter()
            .flat_map(|(field, messages)| {
                messages
                    .iter()
                    .map(|message| format!("{}: {}", field.as_str(), message.as_str()))
            })
            .collect::<Vec<String>>()
            .join(", ");

        req.log(
            "login",
            fastn_core::log::FormErrorOutcome::ValidationError {
                message: err_message,
            }
            .into_kind(),
            file!(),
            line!(),
        );

        return fastn_core::http::user_err(errors, fastn_core::http::StatusCode::OK);
    }

    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            // [ERROR] logging (server-error: PoolError)
            let err_message = format!("Failed to get connection to db. {:?}", &e);
            req.log(
                "login",
                fastn_core::log::ServerErrorOutcome::PoolError {
                    message: err_message.clone(),
                }
                .into_kind(),
                file!(),
                line!(),
            );

            return Err(fastn_core::Error::DatabaseError {
                message: err_message,
            });
        }
    };

    let user: Option<fastn_core::auth::FastnUser> = match fastn_core::schema::fastn_user::table
        .filter(fastn_core::schema::fastn_user::username.eq(&payload.username))
        .or_filter(
            fastn_core::schema::fastn_user::email.eq(fastn_core::utils::citext(&payload.username)),
        )
        .select(fastn_core::auth::FastnUser::as_select())
        .first(&mut conn)
        .await
        .optional()
    {
        Ok(v) => v,
        Err(e) => {
            // [ERROR] logging (server-error: DatabaseQueryError)
            let err_message = format!("{:?}", &e);
            req.log(
                "login",
                fastn_core::log::ServerErrorOutcome::DatabaseQueryError {
                    message: err_message,
                }
                .into_kind(),
                file!(),
                line!(),
            );
            return Err(e.into());
        }
    };

    let user = match user {
        Some(user) => user,
        None => {
            // [ERROR] logging (form-error: ValidationError)
            let err_message = "User: Invalid email/username".to_string();
            req.log(
                "login",
                fastn_core::log::FormErrorOutcome::ValidationError {
                    message: err_message,
                }
                .into_kind(),
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

        // [ERROR] logging (form-error: ValidationError)
        let err_message = "password: can't be empty".to_string();
        req.log(
            "login",
            fastn_core::log::FormErrorOutcome::ValidationError {
                message: err_message,
            }
            .into_kind(),
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
            // [ERROR] logging (hashed password: parse error)
            let err_message = format!("failed to parse hashed password: {e}");
            req.log(
                "login",
                fastn_core::log::ServerErrorOutcome::HashingError {
                    message: err_message.clone(),
                }
                .into_kind(),
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
        // [ERROR] logging (form-error: ValidationError)
        let err_message = "password: incorrect password".to_string();
        req.log(
            "login",
            fastn_core::log::FormErrorOutcome::ValidationError {
                message: err_message,
            }
            .into_kind(),
            file!(),
            line!(),
        );

        return fastn_core::http::user_err(
            vec![("password".into(), vec!["incorrect password".into()])],
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
            // [ERROR] logging (server-error: DatabaseQueryError)
            let err_message = format!("{:?}", &e);
            req.log(
                "login",
                fastn_core::log::ServerErrorOutcome::DatabaseQueryError {
                    message: err_message,
                }
                .into_kind(),
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
        "login",
        &req_config.config.ds,
        session_id,
        next,
    )
    .await
}
