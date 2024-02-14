pub(crate) fn confirmation_link(
    req: &fastn_core::http::Request,
    key: String,
    next: String,
) -> String {
    format!(
        "{}://{}/-/auth/confirm-email/?code={key}&next={}",
        req.connection_info.scheme(),
        req.connection_info.host(),
        next
    )
}

pub(crate) fn redirect_url_from_next(req: &fastn_core::http::Request, next: String) -> String {
    format!(
        "{}://{}{}",
        req.connection_info.scheme(),
        req.connection_info.host(),
        next,
    )
}
