// handle: if request.url starts with /-/auth/
#[tracing::instrument(skip_all)]
pub async fn handle_auth(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
    config: &fastn_core::Config,
) -> fastn_core::Result<fastn_core::http::Response> {
    use fastn_core::auth::Route;

    req.log(
        "initial",
        fastn_core::log::OutcomeKind::Info,
        file!(),
        line!(),
    );

    let next = req.q("next", "/".to_string())?;

    let pool = fastn_core::db::pool(&req_config.config.ds)
        .await
        .as_ref()
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    match Into::<Route>::into(req.path()) {
        Route::Login => fastn_core::auth::email_password::login(&req, req_config, pool, next).await,
        Route::GithubLogin => {
            fastn_core::auth::github::login(&req, &req_config.config.ds, next).await
        }
        Route::GithubCallback => {
            fastn_core::auth::github::callback(&req, &req_config.config.ds, pool, next).await
        }
        Route::Logout => fastn_core::auth::logout(&req, &req_config.config.ds, pool, next).await,
        Route::CreateAccount => {
            fastn_core::auth::email_password::create_account(&req, req_config, pool, next).await
        }
        Route::EmailConfirmationSent => {
            fastn_core::auth::email_password::email_confirmation_sent(&req, req_config).await
        }
        Route::ConfirmEmail => {
            fastn_core::auth::email_password::confirm_email(&req, req_config, pool, next).await
        }
        Route::ResendConfirmationEmail => {
            fastn_core::auth::email_password::resend_confirmation_email(
                &req, req_config, pool, next,
            )
            .await
        }
        Route::Onboarding => {
            fastn_core::auth::email_password::onboarding(&req, req_config, config, next).await
        }
        Route::ForgotPassword => {
            fastn_core::auth::email_password::forgot_password_request(&req, req_config, pool, next)
                .await
        }
        Route::ForgotPasswordSuccess => {
            fastn_core::auth::email_password::forgot_password_request_success(&req, req_config)
                .await
        }
        Route::SetPassword => {
            fastn_core::auth::email_password::set_password(&req, req_config, pool, next).await
        }
        Route::SetPasswordSuccess => {
            fastn_core::auth::email_password::set_password_success(&req, req_config).await
        }
        Route::Invalid => {
            req.log(
                "invalid-route",
                fastn_core::log::OutcomeKind::Error(fastn_core::log::Outcome::Default),
                file!(),
                line!(),
            );
            Ok(fastn_core::not_found!("route not found: {}", req.path()))
        }
    }
}
