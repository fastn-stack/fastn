pub fn process<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    let mut found_cookie = false;
    let is_login = match &config.request {
        Some(req) => {
            for auth_provider in fpm::auth::AuthProviders::AUTH_ITER.iter() {
                if req.cookie(auth_provider.as_str()).is_some() {
                    found_cookie = true;
                }
            }
            found_cookie
        }
        None => false,
    };

    #[derive(Debug, serde::Serialize)]
    struct UserDetails {
        #[serde(rename = "is-login")]
        is_login: bool,
    }
    let ud = UserDetails { is_login };
    doc.from_json(&ud, &kind, value.line_number())
}
