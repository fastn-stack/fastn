// handle: if request.url starts with /-/auth/
#[tracing::instrument(skip_all)]
pub async fn handle_auth(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
    config: &fastn_core::Config,
) -> fastn_core::Result<fastn_core::http::Response> {
    use fastn_core::auth::Route;

    let next = req.q("next", "/".to_string())?;

    let pool = fastn_core::db::pool(&req_config.config.ds)
        .await
        .as_ref()
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    let route = Into::<Route>::into(req.path());
    let ekind = route.to_event_kind();
    let response = match route {
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
            // [ERROR] logging (bad-request: InvalidRoute)
            let err_message = format!(
                "req: {}, method: {}",
                req_config.request.path.as_str(),
                req_config.request.method()
            );
            req.log(
                "invalid-route",
                fastn_core::log::BadRequestOutcome::InvalidRoute {
                    message: err_message,
                }
                .into_kind(),
                file!(),
                line!(),
            );
            Ok(fastn_core::not_found!("route not found: {}", req.path()))
        }
    };

    // [SUCCESS] logging: Default
    if response.is_ok() {
        match route {
            Route::GithubLogin
            | Route::GithubCallback
            | Route::Logout
            | Route::EmailConfirmationSent
            | Route::ConfirmEmail
            | Route::Onboarding
            | Route::ForgotPasswordSuccess
            | Route::SetPasswordSuccess => {
                req.log(
                    ekind.as_str(),
                    fastn_core::log::OutcomeKind::success_default(),
                    file!(),
                    line!(),
                );
            }
            _ => {} // other routes return form errors so handled separately
        };
    }

    response
}
