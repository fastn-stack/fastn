use crate::auth::email_password::{create_account_ftd, redirect_url_from_next};

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
    #[validate(must_match(
        other = "password",
        message = "password and confirm password field do not match"
    ))]
    password2: String,
    #[validate(custom(
        function = "fastn_core::auth::validator::accept_terms",
        message = "you must accept the terms and conditions"
    ))]
    accept_terms: bool,
}

pub(crate) async fn create_account(
    req: &fastn_core::http::Request,
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
            id: fastn_core::auth::Route::CreateAccount.to_string(),
            content: create_account_ftd().to_string(),
            parent_path: fastn_ds::Path::new("/"),
        };
        return match fastn_core::package::package_doc::read_ftd(
            req_config, &main, "/", false, false,
        )
        .await
        {
            Ok(resp) => {
                // [SUCCESS] logging: GET
                let log_success_message = "create-account: GET".to_string();
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
                    "create-account",
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

    let user_payload = match req_config.request.json::<UserPayload>() {
        Ok(p) => p,
        Err(e) => {
            // [ERROR] logging (form-error: PayloadError)
            let err_message = format!(
                "Invalid payload. Required the request body to contain json. Original error: {:?}",
                e
            );
            req.log(
                "create-account",
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

    if let Err(mut e) = user_payload.validate_args((
        user_payload.username.as_str(),
        user_payload.email.as_str(),
        user_payload.name.as_str(),
    )) {
        if req_config
            .config
            .ds
            .env_bool("DEBUG_FASTN_DISABLE_STRONG_PASSWORD_CHECK", false)
            .await?
            && !user_payload.password.is_empty()
        {
            e.errors_mut().remove("password");
        }

        if !e.is_empty() {
            // [ERROR] logging (form-error: ValidationError)
            let err_message = format!("{:?}", &e);
            req.log(
                "create-account",
                fastn_core::log::FormErrorOutcome::ValidationError {
                    message: err_message,
                }
                .into_kind(),
                file!(),
                line!(),
            );

            return fastn_core::http::validation_error_to_user_err(e);
        }
    }

    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            // [ERROR] logging (server-error: PoolError)
            let err_message = format!("Failed to get connection to db. {:?}", &e);
            req.log(
                "create-account",
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

    let username_check: i64 = match fastn_core::schema::fastn_user::table
        .filter(fastn_core::schema::fastn_user::username.eq(&user_payload.username))
        .select(diesel::dsl::count(fastn_core::schema::fastn_user::id))
        .first(&mut conn)
        .await
    {
        Ok(user_count) => user_count,
        Err(e) => {
            // [ERROR] logging (server-error: DatabaseQueryError)
            let err_message = format!("{:?}", &e);
            req.log(
                "create-account",
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

    if username_check > 0 {
        // [ERROR] logging (form-error: ValidationError)
        let err_message = "username already taken".to_string();
        req.log(
            "create-account",
            fastn_core::log::FormErrorOutcome::ValidationError {
                message: err_message.clone(),
            }
            .into_kind(),
            file!(),
            line!(),
        );

        return fastn_core::http::user_err(
            vec![("username".into(), vec![err_message])],
            fastn_core::http::StatusCode::OK,
        );
    }

    let email_check: i64 = match fastn_core::schema::fastn_user::table
        .filter(
            fastn_core::schema::fastn_user::email
                .eq(fastn_core::utils::citext(&user_payload.email)),
        )
        .select(diesel::dsl::count(fastn_core::schema::fastn_user::id))
        .first(&mut conn)
        .await
    {
        Ok(email_count) => email_count,
        Err(e) => {
            // [ERROR] logging (server-error: DatabaseQueryError)
            let err_message = format!("{:?}", &e);
            req.log(
                "create-account",
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

    if email_check > 0 {
        // [ERROR] logging (form-error: ValidationError)
        let err_message = "email already taken".to_string();
        req.log(
            "create-account",
            fastn_core::log::FormErrorOutcome::ValidationError {
                message: err_message.clone(),
            }
            .into_kind(),
            file!(),
            line!(),
        );

        return fastn_core::http::user_err(
            vec![("email".into(), vec![err_message])],
            fastn_core::http::StatusCode::OK,
        );
    }

    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);

    let argon2 = argon2::Argon2::default();

    let hashed_password =
        argon2::PasswordHasher::hash_password(&argon2, user_payload.password.as_bytes(), &salt)
            .map_err(|e| {
                // [ERROR] logging (server-error: HashingError)
                let err_message = format!("error in hashing password: {:?}", &e);
                req.log(
                    "create-account",
                    fastn_core::log::ServerErrorOutcome::HashingError {
                        message: err_message.clone(),
                    }
                    .into_kind(),
                    file!(),
                    line!(),
                );
                fastn_core::Error::generic(err_message)
            })?
            .to_string();

    let save_user_email_transaction = conn
        .build_transaction()
        .run(|c| {
            Box::pin(async move {
                let user = match diesel::insert_into(fastn_core::schema::fastn_user::table)
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
                    .await
                {
                    Ok(fastn_user) => fastn_user,
                    Err(e) => {
                        // [ERROR] logging (server-error: DatabaseQueryError)
                        let err_message = format!("{:?}", &e);
                        req.log(
                            "create-account",
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

                // just for record keeping
                // we do not use `fastn_user_email` for auth at all
                let _user_email =
                    match diesel::insert_into(fastn_core::schema::fastn_user_email::table)
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
                        .await
                    {
                        Ok(email) => email,
                        Err(e) => {
                            // [ERROR] logging (server-error: DatabaseQueryError)
                            let err_message = format!("{:?}", &e);
                            req.log(
                                "create-account",
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

                tracing::info!("fastn_user created. user_id: {}", &user.id);
                Ok::<fastn_core::auth::FastnUser, diesel::result::Error>(user)
            })
        })
        .await;

    let user = match save_user_email_transaction {
        Ok(user) => user,
        Err(e) => {
            // [ERROR] logging (form-error: ValidationError)
            let err_message = format!("email: invalid email, detail: {:?}", &e);
            req.log(
                "create-account",
                fastn_core::log::FormErrorOutcome::ValidationError {
                    message: err_message,
                }
                .into_kind(),
                file!(),
                line!(),
            );

            return fastn_core::http::user_err(
                vec![
                    ("email".into(), vec!["invalid email".into()]),
                    ("detail".into(), vec![format!("{e}")]),
                ],
                fastn_core::http::StatusCode::OK,
            );
        }
    };

    tracing::info!("fastn_user email inserted, id: {}", user.id);

    let next = format!(
        "{resend_conf_route}?email={email}&next={next}",
        resend_conf_route = fastn_core::auth::Route::ResendConfirmationEmail,
        email = user.email.0
    );

    Ok(fastn_core::http::temporary_redirect(
        redirect_url_from_next(&req_config.request, next),
    ))
}
