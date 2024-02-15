use crate::auth::email_password::{email_confirmation_sent_ftd, key_expired};

pub(crate) async fn confirm_email(
    req_config: &mut fastn_core::RequestConfig,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let code = req_config.request.query().get("code");

    if code.is_none() {
        let main = fastn_core::Document {
            package_name: req_config.config.package.name.clone(),
            id: "/-/confirm-email/".to_string(),
            content: email_confirmation_sent_ftd().to_string(),
            parent_path: fastn_ds::Path::new("/"),
        };

        let resp = fastn_core::package::package_doc::read_ftd(req_config, &main, "/", false, false)
            .await?;

        return Ok(resp.into());
    }

    let code = match code.unwrap() {
        serde_json::Value::String(c) => c,
        _ => {
            tracing::info!("failed to Deserialize ?code as string");
            return Ok(fastn_core::http::api_error("Bad Request")?);
        }
    };

    let mut conn = db_pool
        .get()
        .await
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    let conf_data: Option<(i32, i32, chrono::DateTime<chrono::Utc>)> =
        fastn_core::schema::fastn_email_confirmation::table
            .select((
                fastn_core::schema::fastn_email_confirmation::email_id,
                fastn_core::schema::fastn_email_confirmation::session_id,
                fastn_core::schema::fastn_email_confirmation::sent_at,
            ))
            .filter(fastn_core::schema::fastn_email_confirmation::key.eq(&code))
            .first(&mut conn)
            .await
            .optional()?;

    if conf_data.is_none() {
        tracing::info!("invalid code value. No entry exists for the given code in db");
        tracing::info!("provided code: {}", &code);
        return Ok(fastn_core::http::api_error("Bad Request")?);
    }

    let (email_id, session_id, sent_at) = conf_data.unwrap();

    if key_expired(&req_config.config.ds, sent_at).await {
        // TODO: this redirect route should be configurable
        tracing::info!("provided code has expired.");
        return Ok(fastn_core::http::redirect_with_code(
            format!(
                "{}://{}/-/auth/resend-confirmation-email/",
                req_config.request.connection_info.scheme(),
                req_config.request.connection_info.host(),
            ),
            302,
        ));
    }

    diesel::update(fastn_core::schema::fastn_user_email::table)
        .set(fastn_core::schema::fastn_user_email::verified.eq(true))
        .filter(fastn_core::schema::fastn_user_email::id.eq(email_id))
        .execute(&mut conn)
        .await?;

    let affected = diesel::update(fastn_core::schema::fastn_session::table)
        .set(fastn_core::schema::fastn_session::active.eq(true))
        .filter(fastn_core::schema::fastn_session::id.eq(session_id))
        .execute(&mut conn)
        .await?;

    tracing::info!("session created, rows affected: {}", affected);

    // Onboarding step is opt-in
    let onboarding_enabled = req_config
        .config
        .ds
        .env("FASTN_AUTH_ADD_ONBOARDING_STEP")
        .await
        .is_ok();

    let next_path = if onboarding_enabled {
        format!("/-/auth/onboarding/?next={}", next)
    } else {
        next.to_string()
    };

    // redirect to onboarding route with a GET request
    let mut resp = fastn_core::auth::set_session_cookie_and_redirect_to_next(
        &req_config.request,
        &req_config.config.ds,
        session_id,
        next_path,
    )
    .await?;

    if onboarding_enabled {
        resp.add_cookie(
            &actix_web::cookie::Cookie::build(
                fastn_core::auth::FIRST_TIME_SESSION_COOKIE_NAME,
                "1",
            )
            .domain(fastn_core::auth::utils::domain(
                req_config.request.connection_info.host(),
            ))
            .path("/")
            .finish(),
        )
        .map_err(|e| fastn_core::Error::generic(format!("failed to set cookie: {e}")))?;
    }

    Ok(resp)
}
