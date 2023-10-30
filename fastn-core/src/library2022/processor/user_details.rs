pub fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let mut found_cookie = false;
    let is_login = {
        for auth_provider in fastn_core::auth::AuthProviders::AUTH_ITER.iter() {
            if req_config.request.cookie(auth_provider.as_str()).is_some() {
                found_cookie = true;
                break;
            }
        }
        found_cookie
    };

    #[derive(Debug, serde::Serialize)]
    struct UserDetails {
        #[serde(rename = "is-login")]
        is_login: bool,
    }
    let ud = UserDetails { is_login };
    doc.from_json(&ud, &kind, &value)
}
