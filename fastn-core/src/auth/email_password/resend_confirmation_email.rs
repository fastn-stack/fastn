use crate::auth::email_password::create_and_send_confirmation_email;

pub(crate) async fn resend_confirmation_email(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    req.log_with_no_site(
        fastn_core::log::EventKind::Auth(fastn_core::log::AuthEvent::ResendConfirmationEmail),
        fastn_core::log::EntityKind::Myself,
        fastn_core::log::OutcomeKind::Info,
        line!(),
    );

    // TODO: should be able to use username for this too
    let email = req.query().get("email");

    if email.is_none() {
        return Ok(fastn_core::http::api_error("Bad Request")?);
    }

    let email = match email.unwrap() {
        serde_json::Value::String(c) => c.to_owned(),
        _ => {
            return Ok(fastn_core::http::api_error("Bad Request")?);
        }
    };

    if !validator::validate_email(&email) {
        return Ok(fastn_core::http::api_error("Bad Request")?);
    }

    let mut conn = db_pool
        .get()
        .await
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    let (conf_link, session_id) =
        create_and_send_confirmation_email(email, &mut conn, req_config, next.clone()).await?;

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
        &req_config.config.ds,
        session_id,
        next,
    )
    .await
}
