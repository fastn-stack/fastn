use crate::auth::email_password::create_and_send_confirmation_email;

pub(crate) async fn resend_confirmation_email(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    // [INFO] logging: resend-confirmation-email
    req.log(
        "resend-confirmation-email",
        fastn_core::log::OutcomeKind::Info,
        file!(),
        line!(),
    );

    // TODO: should be able to use username for this too
    let email = match req.query().get("email") {
        Some(email) => email,
        None => {
            // [ERROR] logging (query: not-found)
            let log_err_message = "query: email not found".to_string();
            req.log(
                "resend-confirmation-email",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );

            return Ok(fastn_core::http::api_error("Bad Request")?);
        }
    };

    let email = match email {
        serde_json::Value::String(c) => c.to_owned(),
        _ => {
            // [ERROR] logging (query: failed-to-deserialize)
            let log_err_message = "query: failed to deserialize email".to_string();
            req.log(
                "resend-confirmation-email",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );

            return Ok(fastn_core::http::api_error("Bad Request")?);
        }
    };

    if !validator::validate_email(&email) {
        // [ERROR] logging (query: validation-error)
        let log_err_message = "query: failed to validate email".to_string();
        req.log(
            "resend-confirmation-email",
            fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
            file!(),
            line!(),
        );

        return Ok(fastn_core::http::api_error("Bad Request")?);
    }

    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            // [ERROR] logging (pool-error)
            let err_message = format!("Failed to get connection to db. {:?}", &e);
            let log_err_message = format!("pool error: {}", err_message.as_str());
            req.log(
                "resend-confirmation-email",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );

            return Err(fastn_core::Error::DatabaseError {
                message: err_message,
            });
        }
    };

    let (conf_link, session_id) =
        create_and_send_confirmation_email(email, &mut conn, req, req_config, next.clone()).await?;

    // email is not enabled, we should log conf link assuming dev mode
    if !req_config
        .config
        .ds
        .env_bool("FASTN_ENABLE_EMAIL", true)
        .await?
    {
        println!("CONFIRMATION LINK: {}", conf_link);
    }

    fastn_core::auth::set_session_cookie_and_redirect_to_next(
        &req_config.request,
        "resend-confirmation-email",
        &req_config.config.ds,
        session_id,
        next,
    )
    .await
}
