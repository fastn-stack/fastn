use crate::auth::email_password::{
    forgot_password_form_ftd, forgot_password_request_success_ftd, generate_key,
    redirect_url_from_next, set_password_form_ftd, set_password_success_ftd,
};

/// GET | POST /-/auth/forgot-password/
/// POST forgot_password_request: send email with a link containing a key to set password
/// for unauthenticated users
pub(crate) async fn forgot_password_request(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    // [INFO] logging: forgot-password
    req.log(
        "forgot-password",
        fastn_core::log::OutcomeKind::Info,
        file!(),
        line!(),
    );

    if req_config.request.ud(&req_config.config.ds).await.is_some() {
        // [ERROR] logging (bad-request)
        let log_err_message = "bad request".to_string();
        req.log(
            "forgot-password",
            fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
            file!(),
            line!(),
        );

        return Ok(fastn_core::http::api_error("Bad Request")?);
    }

    if req_config.request.method() == "GET" {
        let main = fastn_core::Document {
            package_name: req_config.config.package.name.clone(),
            id: fastn_core::auth::Route::ForgotPassword.to_string(),
            content: forgot_password_form_ftd().to_string(),
            parent_path: fastn_ds::Path::new("/"),
        };

        return match fastn_core::package::package_doc::read_ftd(
            req_config, &main, "/", false, false,
        )
        .await
        {
            Ok(response) => Ok(response.into()),
            Err(e) => {
                // [ERROR] logging (read_ftd)
                let log_err_message = format!("read_ftd: {:?}", &e);
                req.log(
                    "forgot-password",
                    fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                    file!(),
                    line!(),
                );
                Err(e)
            }
        };
    }

    if req_config.request.method() != "POST" {
        // [ERROR] logging (invalid-route)
        let log_err_message = "invalid route".to_string();
        req.log(
            "forgot-password",
            fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
            file!(),
            line!(),
        );

        return Ok(fastn_core::not_found!("invalid route"));
    }

    #[derive(serde::Deserialize)]
    struct Payload {
        #[serde(rename = "username")]
        email_or_username: String,
    }

    let payload = match req_config.request.json::<Payload>() {
        Ok(payload) => payload,
        Err(e) => {
            let errors = vec![
                (
                    "payload".to_string(),
                    vec![format!("invalid payload: {:?}", &e)],
                ),
                (
                    "username".to_string(),
                    vec!["username/email is required".to_string()],
                ),
            ];

            // [ERROR] logging (payload-error)
            let err_message = fastn_core::auth::utils::errors_to_message(&errors);
            let log_err_message = format!("payload: {:?}", &err_message);
            req.log(
                "forgot-password",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );

            return fastn_core::http::user_err(errors, fastn_core::http::StatusCode::OK);
        }
    };

    if payload.email_or_username.is_empty() {
        let errors = vec![(
            "username".into(),
            vec!["username/email is required".to_string()],
        )];

        // [ERROR] logging (user-error)
        let err_message = fastn_core::auth::utils::errors_to_message(&errors);
        let log_err_message = format!("user: {:?}", &err_message);
        req.log(
            "forgot-password",
            fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
            file!(),
            line!(),
        );

        return fastn_core::http::user_err(errors, fastn_core::http::StatusCode::OK);
    }

    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            // [ERROR] logging (pool-error)
            let err_message = format!("Failed to get connection to db. {:?}", &e);
            let log_err_message = format!("pool error: {}", err_message.as_str());
            req.log(
                "forgot-password",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );

            return Err(fastn_core::Error::DatabaseError {
                message: err_message,
            });
        }
    };

    let user: Option<(fastn_core::auth::FastnUser, fastn_core::utils::CiString)> =
        match fastn_core::schema::fastn_user::table
            .inner_join(fastn_core::schema::fastn_user_email::table)
            .filter(fastn_core::schema::fastn_user::username.eq(&payload.email_or_username))
            .or_filter(
                fastn_core::schema::fastn_user_email::email
                    .eq(fastn_core::utils::citext(&payload.email_or_username)),
            )
            .select((
                fastn_core::auth::FastnUser::as_select(),
                fastn_core::schema::fastn_user_email::email,
            ))
            .first(&mut conn)
            .await
            .optional()
        {
            Ok(v) => v,
            Err(e) => {
                // [ERROR] logging (database-error)
                let log_err_message = format!("database: {:?}", &e);
                req.log(
                    "forgot-password",
                    fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                    file!(),
                    line!(),
                );
                return Err(e.into());
            }
        };

    let (user, email) = match user {
        Some(v) => v,
        None => {
            let errors = vec![(
                "username".into(),
                vec!["invalid email/username".to_string()],
            )];

            // [ERROR] logging (user-error)
            let err_message = fastn_core::auth::utils::errors_to_message(&errors);
            let log_err_message = format!("user: {:?}", &err_message);
            req.log(
                "forgot-password",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );

            return fastn_core::http::user_err(errors, fastn_core::http::StatusCode::OK);
        }
    };

    let key = generate_key(64);

    let _ = match diesel::insert_into(fastn_core::schema::fastn_password_reset::table)
        .values((
            fastn_core::schema::fastn_password_reset::user_id.eq(&user.id),
            fastn_core::schema::fastn_password_reset::key.eq(&key),
            fastn_core::schema::fastn_password_reset::sent_at.eq(chrono::offset::Utc::now()),
        ))
        .execute(&mut conn)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            // [ERROR] logging (database-error)
            let log_err_message = format!("database: {:?}", &e);
            req.log(
                "forgot-password",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            return Err(e.into());
        }
    };

    let reset_link = format!(
        "{scheme}://{host}{reset_password_route}?code={key}&next={next}",
        scheme = req_config.request.connection_info.scheme(),
        host = req_config.request.connection_info.host(),
        reset_password_route = fastn_core::auth::Route::SetPassword,
    );

    // To use auth. The package has to have auto import with alias `auth` setup
    let path = req_config
        .config
        .package
        .eval_auto_import("auth")
        .unwrap()
        .to_owned();

    let path = path
        .strip_prefix(format!("{}/", req_config.config.package.name).as_str())
        .unwrap();

    let content = match req_config
        .config
        .ds
        .read_to_string(&fastn_ds::Path::new(format!("{}.ftd", path)))
        .await
    {
        Ok(content) => content,
        Err(e) => {
            // [ERROR] logging (read-error)
            let log_err_message = format!("read: {:?}", &e);
            req.log(
                "forgot-password",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            return Err(e.into());
        }
    };

    let auth_doc = fastn_core::Document {
        package_name: req_config.config.package.name.clone(),
        id: path.to_string(),
        content,
        parent_path: fastn_ds::Path::new("/"),
    };

    let main_ftd_doc = match fastn_core::doc::interpret_helper(
        auth_doc.id_with_package().as_str(),
        auth_doc.content.as_str(),
        req_config,
        "/",
        false,
        0,
    )
    .await
    {
        Ok(doc) => doc,
        Err(e) => {
            // [ERROR] logging (interpreter-error)
            let log_err_message = format!("read: {:?}", &e);
            req.log(
                "forgot-password",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            return Err(e.into());
        }
    };

    let html_email_templ = format!(
        "{}/{}#reset-password-request-mail-html",
        req_config.config.package.name, path
    );

    let html: String = match main_ftd_doc.get(&html_email_templ) {
        Ok(html) => html,
        _ => {
            // [ERROR] logging (html-email-template: not-found)
            let err_message = "html email template not found".to_string();
            let log_err_message = format!("mail: {:?}", &err_message);
            req.log(
                "forgot-password",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            return Err(fastn_core::Error::GenericError(err_message));
        }
    };
    let html = html.replace("{{link}}", &reset_link);

    let enable_email = req_config
        .config
        .ds
        .env_bool("FASTN_ENABLE_EMAIL", true)
        .await
        .unwrap_or(true);

    if !enable_email {
        println!("RESET LINK: {}", &reset_link);
    }

    fastn_core::mail::Mailer::send_raw(
        enable_email,
        &req_config.config.ds,
        format!("{} <{}>", user.name, email.0)
            .parse::<lettre::message::Mailbox>()
            .unwrap(),
        "Reset your password",
        html,
    )
    .await
    .map_err(|e| {
        // [ERROR] logging (mail-error)
        let err_message = format!("failed to send email: {:?}", &e);
        let log_err_message = format!("mail: {:?}", &err_message);
        req.log(
            "forgot-password",
            fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
            file!(),
            line!(),
        );
        fastn_core::Error::generic(err_message)
    })?;

    let mut resp = actix_web::HttpResponse::Ok();
    let resp_body = serde_json::json!({
        "success": true,
        "redirect": redirect_url_from_next(&req_config.request, fastn_core::auth::Route::ForgotPasswordSuccess.to_string()),
    });

    if req_config.config.test_command_running {
        resp.insert_header(("X-Fastn-Test", "true"))
            .insert_header(("X-Fastn-Test-Reset-Link", reset_link));
    }

    Ok(resp.json(resp_body))
}

pub(crate) async fn forgot_password_request_success(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
) -> fastn_core::Result<fastn_core::http::Response> {
    // [INFO] logging: forgot-password-success
    req.log(
        "forgot-password-success",
        fastn_core::log::OutcomeKind::Info,
        file!(),
        line!(),
    );

    if req_config.request.method() != "GET" {
        // [ERROR] logging (invalid-route)
        let log_err_message = "invalid route".to_string();
        req.log(
            "forgot-password-success",
            fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
            file!(),
            line!(),
        );

        return Ok(fastn_core::not_found!("invalid route"));
    }

    let main = fastn_core::Document {
        package_name: req_config.config.package.name.clone(),
        id: fastn_core::auth::Route::ForgotPasswordSuccess.to_string(),
        content: forgot_password_request_success_ftd().to_string(),
        parent_path: fastn_ds::Path::new("/"),
    };

    return match fastn_core::package::package_doc::read_ftd(req_config, &main, "/", false, false)
        .await
    {
        Ok(response) => Ok(response.into()),
        Err(e) => {
            // [ERROR] logging (read_ftd)
            let log_err_message = format!("read_ftd: {:?}", &e);
            req.log(
                "forgot-password-success",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            Err(e)
        }
    };
}

/// GET | POST /-/auth/set-password/
pub(crate) async fn set_password(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    // [INFO] logging: set-password
    req.log(
        "set-password",
        fastn_core::log::OutcomeKind::Info,
        file!(),
        line!(),
    );

    if req_config.request.method() == "GET" {
        // render set password form
        let main = fastn_core::Document {
            package_name: req_config.config.package.name.clone(),
            id: "/-/set-password".to_string(),
            content: set_password_form_ftd().to_string(),
            parent_path: fastn_ds::Path::new("/"),
        };

        return match fastn_core::package::package_doc::read_ftd(
            req_config, &main, "/", false, false,
        )
        .await
        {
            Ok(response) => Ok(response.into()),
            Err(e) => {
                // [ERROR] logging (read_ftd)
                let log_err_message = format!("read_ftd: {:?}", &e);
                req.log(
                    "set-password",
                    fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                    file!(),
                    line!(),
                );
                Err(e)
            }
        };
    }

    if req_config.request.method() != "POST" {
        // [ERROR] logging (invalid-route)
        let log_err_message = "invalid route".to_string();
        req.log(
            "set-password",
            fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
            file!(),
            line!(),
        );

        return Ok(fastn_core::not_found!("invalid route"));
    }

    #[derive(serde::Deserialize)]
    struct Payload {
        new_password: String,
        new_password2: String,
    }

    let payload = match req_config.request.json::<Payload>() {
        Ok(payload) => payload,
        Err(e) => {
            let errors = vec![
                (
                    "payload".to_string(),
                    vec![format!("invalid payload: {:?}", e)],
                ),
                (
                    "new_password".to_string(),
                    vec!["new password is required".to_string()],
                ),
                (
                    "new_password2".to_string(),
                    vec!["confirm new password is required".to_string()],
                ),
            ];

            // [ERROR] logging (user-error)
            let err_message = fastn_core::auth::utils::errors_to_message(&errors);
            let log_err_message = format!("user: {:?}", &err_message);
            req.log(
                "set-password",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );

            return fastn_core::http::user_err(errors, fastn_core::http::StatusCode::OK);
        }
    };

    let mut errors = Vec::new();
    if payload.new_password.is_empty() {
        errors.push((
            "new_password".into(),
            vec!["new password is required".to_string()],
        ));
    }

    if payload.new_password2.is_empty() {
        errors.push((
            "new_password2".into(),
            vec!["confirm new password is required".to_string()],
        ));
    }

    if payload.new_password != payload.new_password2 {
        errors.push((
            "new_password2".into(),
            vec!["new password and confirm new password do not match".to_string()],
        ));
    }

    if !errors.is_empty() {
        // [ERROR] logging (user-error)
        let err_message = fastn_core::auth::utils::errors_to_message(&errors);
        let log_err_message = format!("user: {:?}", &err_message);
        req.log(
            "set-password",
            fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
            file!(),
            line!(),
        );

        return fastn_core::http::user_err(errors, fastn_core::http::StatusCode::OK);
    }

    let user_id = match req_config.request.ud(&req_config.config.ds).await {
        Some(v) => v.id,
        None => {
            // use the ?code from query params, this is set in /-/auth/forgot-password/
            let key = match req_config.request.query().get("code") {
                Some(key) => key,
                None => {
                    // [ERROR] logging (query-not-found)
                    let log_err_message = "query: code not found".to_string();
                    req.log(
                        "set-password",
                        fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                        file!(),
                        line!(),
                    );

                    return Ok(fastn_core::http::api_error("Bad Request")?);
                }
            };

            let key = match key {
                serde_json::Value::String(c) => c.to_owned(),
                _ => {
                    // [ERROR] logging (query: failed-to-deserialize)
                    let log_err_message = "query: failed to deserialize code".to_string();
                    req.log(
                        "set-password",
                        fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                        file!(),
                        line!(),
                    );

                    return Ok(fastn_core::http::api_error("Bad Request")?);
                }
            };

            let mut conn = match db_pool.get().await {
                Ok(conn) => conn,
                Err(e) => {
                    // [ERROR] logging (pool error)
                    let err_message = format!("Failed to get connection to db. {:?}", &e);
                    let log_err_message = format!("pool error: {}", err_message.as_str());
                    req.log(
                        "set-password",
                        fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                        file!(),
                        line!(),
                    );

                    return Err(fastn_core::Error::DatabaseError {
                        message: err_message,
                    });
                }
            };

            let user_id: Option<i64> = match diesel::delete(
                fastn_core::schema::fastn_password_reset::table
                    .filter(fastn_core::schema::fastn_password_reset::key.eq(&key)),
            )
            .returning(fastn_core::schema::fastn_password_reset::user_id)
            .get_result(&mut conn)
            .await
            .optional()
            {
                Ok(v) => v,
                Err(e) => {
                    // [ERROR] logging (Database Error)
                    let log_err_message = format!("database: {:?}", &e);
                    req.log(
                        "set-password",
                        fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                        file!(),
                        line!(),
                    );
                    return Err(e.into());
                }
            };

            match user_id {
                Some(user_id) => user_id,
                None => {
                    // [ERROR] logging (bad-request)
                    let log_err_message = "bad-request: user-id not found".to_string();
                    req.log(
                        "set-password",
                        fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                        file!(),
                        line!(),
                    );
                    return Ok(fastn_core::http::api_error("Bad Request")?);
                }
            };
        }
    };

    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            // [ERROR] logging (pool error)
            let err_message = format!("Failed to get connection to db. {:?}", &e);
            let log_err_message = format!("pool error: {}", err_message.as_str());
            req.log(
                "set-password",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );

            return Err(fastn_core::Error::DatabaseError {
                message: err_message,
            });
        }
    };

    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);

    let argon2 = argon2::Argon2::default();

    let hashed_password =
        argon2::PasswordHasher::hash_password(&argon2, payload.new_password.as_bytes(), &salt)
            .map_err(|e| {
                // [ERROR] logging (password-hash-error)
                let err_message = format!("error in hashing password: {e}");
                let log_err_message = format!("password: {}", err_message.as_str());
                req.log(
                    "set-password",
                    fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                    file!(),
                    line!(),
                );
                fastn_core::Error::generic(err_message)
            })?
            .to_string();

    let _ = match diesel::update(fastn_core::schema::fastn_user::table)
        .set(fastn_core::schema::fastn_user::password.eq(&hashed_password))
        .filter(fastn_core::schema::fastn_user::id.eq(&user_id))
        .execute(&mut conn)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            // [ERROR] logging (Database Error)
            let log_err_message = format!("database: {:?}", &e);
            req.log(
                "set-password",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            return Err(e.into());
        }
    };

    // log the user out of all sessions
    let affected = match diesel::delete(
        fastn_core::schema::fastn_auth_session::table
            .filter(fastn_core::schema::fastn_auth_session::user_id.eq(&user_id)),
    )
    .execute(&mut conn)
    .await
    {
        Ok(affected) => affected,
        Err(e) => {
            // [ERROR] logging (Database Error)
            let log_err_message = format!("database: {:?}", &e);
            req.log(
                "set-password",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            return Err(e.into());
        }
    };

    tracing::info!("{affected} session removed");

    let success_route = redirect_url_from_next(
        &req_config.request,
        format!(
            "{}?next={next}",
            fastn_core::auth::Route::SetPasswordSuccess
        ),
    );

    Ok(actix_web::HttpResponse::TemporaryRedirect()
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::SESSION_COOKIE_NAME, "")
                .domain(fastn_core::auth::utils::domain(
                    req_config.request.connection_info.host(),
                ))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .append_header((actix_web::http::header::LOCATION, success_route))
        .finish())
}

pub(crate) async fn set_password_success(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
) -> fastn_core::Result<fastn_core::http::Response> {
    // [INFO] logging: set-password-success
    req.log(
        "set-password-success",
        fastn_core::log::OutcomeKind::Info,
        file!(),
        line!(),
    );

    if req_config.request.method() != "GET" {
        // [ERROR] logging (invalid-route)
        let log_err_message = "invalid route".to_string();
        req.log(
            "set-password-success",
            fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
            file!(),
            line!(),
        );

        return Ok(fastn_core::not_found!("invalid route"));
    }

    let main = fastn_core::Document {
        package_name: req_config.config.package.name.clone(),
        id: fastn_core::auth::Route::SetPasswordSuccess.to_string(),
        content: set_password_success_ftd().to_string(),
        parent_path: fastn_ds::Path::new("/"),
    };

    return match fastn_core::package::package_doc::read_ftd(req_config, &main, "/", false, false)
        .await
    {
        Ok(response) => Ok(response.into()),
        Err(e) => {
            // [ERROR] logging (read_ftd)
            let log_err_message = format!("read_ftd: {:?}", &e);
            req.log(
                "set-password-success",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            Err(e)
        }
    };
}
