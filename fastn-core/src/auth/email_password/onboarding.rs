use crate::auth::email_password::{onboarding_ftd, redirect_url_from_next};

pub(crate) async fn onboarding(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
    config: &fastn_core::Config,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    // [INFO] logging: onboarding
    req.log(
        "onboarding",
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
        // [SUCCESS] logging: redirect-next
        let log_success_message = "onboarding: redirect-next".to_string();
        req.log(
            "onboarding",
            fastn_core::log::OutcomeKind::Success(fastn_core::log::SuccessOutcome::Descriptive(
                log_success_message,
            )),
            file!(),
            line!(),
        );
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

    let mut resp: fastn_core::http::Response = match fastn_core::package::package_doc::read_ftd(
        req_config,
        &first_signin_doc,
        "/",
        false,
        false,
    )
    .await
    {
        Ok(response) => response.into(),
        Err(e) => {
            // [ERROR] logging (server-error: ReadFTDError)
            let err_message = format!("{:?}", &e);
            req.log(
                "onboarding",
                fastn_core::log::ServerErrorOutcome::ReadFTDError {
                    message: err_message,
                }
                .into_kind(),
                file!(),
                line!(),
            );

            return Err(e);
        }
    };

    // clear the cookie so that subsequent requests redirect to `next`
    // this gives the onboarding page a single chance to do the process
    resp.add_cookie(
        &actix_web::cookie::Cookie::build(fastn_core::auth::FIRST_TIME_SESSION_COOKIE_NAME, "")
            .domain(fastn_core::auth::utils::domain(req.connection_info.host()))
            .path("/")
            .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
            .finish(),
    )
    .map_err(|e| {
        // [ERROR] logging (server-error: CookieError)
        let err_message = format!("failed to set cookie: {:?}", &e);
        req.log(
            "onboarding",
            fastn_core::log::ServerErrorOutcome::CookieError {
                message: err_message.clone(),
            }
            .into_kind(),
            file!(),
            line!(),
        );

        fastn_core::Error::generic(err_message)
    })?;

    Ok(resp)
}
