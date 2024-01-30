/// currently returns the github user details
pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    #[cfg(feature = "auth")]
    if let Some(ud) = req_config.request.ud() {
        let ud = UserDetails {
            is_logged_in: true,
            username: ud.username,
            name: ud.name,
            email: ud.email,
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
}
