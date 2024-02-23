pub(crate) fn confirmation_link(
    req: &fastn_core::http::Request,
    key: String,
    next: String,
) -> String {
    format!(
        "{scheme}://{host}{confirm_email_route}?code={key}&next={next}",
        scheme = req.connection_info.scheme(),
        host = req.connection_info.host(),
        confirm_email_route = fastn_core::auth::Route::ConfirmEmail,
    )
}

pub(crate) fn redirect_url_from_next(req: &fastn_core::http::Request, next: String) -> String {
    format!(
        "{scheme}://{host}{next}",
        scheme = req.connection_info.scheme(),
        host = req.connection_info.host(),
    )
}
