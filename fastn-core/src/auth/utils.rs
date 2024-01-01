// 127.0.0.1:8000 -> 127.0.0.1
pub fn domain(host: &str) -> String {
    match host.split_once(':') {
        Some((domain, _port)) => domain.to_string(),
        None => host.to_string(),
    }
}

pub fn is_authenticated(req: &fastn_core::http::Request) -> bool {
    req.cookie(fastn_core::auth::COOKIE_NAME).is_some()
}
