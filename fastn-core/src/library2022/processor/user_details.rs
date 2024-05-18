/// returns details of the logged in user
pub async fn process(
    value: ftd_ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    if let Some(ud) = req_config.request.ud(&req_config.config.ds).await {
        let ud = UserDetails {
            is_logged_in: true,
            username: ud.identity,
            name: ud.name,
            email: ud.email,
            verified_email: ud.verified_email,
        };

        return doc.from_json(&ud, &kind, &value);
    }

    let _ = req_config;
    let ud: UserDetails = Default::default();
    doc.from_json(&ud, &kind, &value)
}

#[derive(Debug, serde::Serialize, Default)]
struct UserDetails {
    #[serde(rename = "is-logged-in")]
    is_logged_in: bool,
    username: String,
    name: String,
    email: String,
    #[serde(rename = "verified-email")]
    verified_email: bool,
}
