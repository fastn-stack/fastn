use crate::auth::email_password::{onboarding_ftd, redirect_url_from_next};

pub(crate) async fn onboarding(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
    config: &fastn_core::Config,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    req.log(
        fastn_core::log::EventKind::Auth(fastn_core::log::AuthEvent::Onboarding),
        fastn_core::log::EntityKind::Myself,
        fastn_core::log::OutcomeKind::Info,
        file!(),
        line!(),
    );

    // The user is logged in after having verified their email. This is them first time signing
    // in so we render `onboarding_ftd`.
    // If this is an old user, the cookie FIRST_TIME_SESSION_COOKIE_NAME won't be set for them
    // and this will redirect to `next` which is usually the home page.
    if req
        .cookie(fastn_core::auth::FIRST_TIME_SESSION_COOKIE_NAME)
        .is_none()
    {
        return Ok(fastn_core::http::temporary_redirect(
            redirect_url_from_next(req, next),
        ));
    }

    let first_signin_doc = fastn_core::Document {
        package_name: config.package.name.clone(),
        id: "/-/onboarding/".to_string(),
        content: onboarding_ftd().to_string(),
        parent_path: fastn_ds::Path::new("/"),
    };

    let resp = fastn_core::package::package_doc::read_ftd(
        req_config,
        &first_signin_doc,
        "/",
        false,
        false,
    )
    .await?;

    let mut resp: fastn_core::http::Response = resp.into();

    // clear the cookie so that subsequent requests redirect to `next`
    // this gives the onboarding page a single chance to do the process
    resp.add_cookie(
        &actix_web::cookie::Cookie::build(fastn_core::auth::FIRST_TIME_SESSION_COOKIE_NAME, "")
            .domain(fastn_core::auth::utils::domain(req.connection_info.host()))
            .path("/")
            .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
            .finish(),
    )
    .map_err(|e| fastn_core::Error::generic(format!("failed to set cookie: {e}")))?;

    Ok(resp)
}
