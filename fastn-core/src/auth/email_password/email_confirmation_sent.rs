use crate::auth::email_password::email_confirmation_sent_ftd;

// TODO: this is unused right now
// the user is immediately logged in after creating an account and this page is never
// visited/redirected to
pub(crate) async fn email_confirmation_sent(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
) -> fastn_core::Result<fastn_core::http::Response> {
    let main = fastn_core::Document {
        package_name: req_config.config.package.name.clone(),
        id: fastn_core::auth::Route::EmailConfirmationSent.to_string(),
        content: email_confirmation_sent_ftd().to_string(),
        parent_path: fastn_ds::Path::new("/"),
    };

    return match fastn_core::package::package_doc::read_ftd(req_config, &main, "/", false, false)
        .await
    {
        Ok(resp) => Ok(resp.into()),
        Err(e) => {
            // [ERROR] logging (server-error: ReadFTDError)
            let err_message = format!("{:?}", &e);
            req.log(
                "email-confirmation-sent",
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
