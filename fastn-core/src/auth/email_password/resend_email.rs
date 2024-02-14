use crate::auth::email_password::{create_and_send_confirmation_email, redirect_url_from_next};

pub(crate) async fn resend_email(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    // TODO: should be able to use username for this too
    // TODO: use req.body and make it POST
    // verify email with regex or validator crate
    // on GET this handler should render auth.resend-email-page
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

    create_and_send_confirmation_email(email, db_pool, req_config, next.clone()).await?;

    // TODO: there's no GET /-/auth/login/ yet
    // the client will have to create one for now
    // this path should be configuratble too
    Ok(fastn_core::http::redirect_with_code(
        redirect_url_from_next(req, next),
        302,
    ))
}
