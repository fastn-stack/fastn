/// currently returns the github user details
pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let mut ud = Default::default();

    if let Some(gh_cookie) = req_config.request.cookie("github") {
        dbg!(&gh_cookie);
        if let Ok(user_detail) = fastn_core::auth::decrypt(&gh_cookie)
            .await
            .map_err(|e| tracing::info!("[user-details]: Failed to decrypt cookie: {e}"))
            .and_then(|decrypted_cookie| {
                serde_json::from_str::<fastn_core::auth::github::UserDetail>(
                    decrypted_cookie.as_str(),
                )
                .map_err(|e| tracing::info!("[user-details]: Serde deserialization failed {e}:"))
            })
        {
            ud = UserDetails {
                is_logged_in: true,
                username: user_detail.user.login,
                name: user_detail.user.name.unwrap_or_default(),
                email: user_detail.user.email.unwrap_or_default(),
            }
        }
    }

    dbg!("ud::", &ud);

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
