/// currently returns the github user details
pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let mut ud = Default::default();

    if let Some(gh_cookie) = req_config.request.cookie("github") {
        if let Ok(user_detail) = fastn_core::auth::decrypt(&gh_cookie)
            .await
            .map_err(|e| tracing::info!("[user-details]: Failed to decrypt cookie: {e}"))
            .and_then(|decrypted_cookie| {
                serde_json::from_str::<fastn_core::auth::github::UserDetail>(decrypted_cookie.as_str()).map_err(
                    |e| tracing::info!("[user-details]: Serde deserialization failed {e}:"),
                )
            })
        {
            match fastn_core::auth::github::user_details(user_detail.access_token.as_str()).await {
                Ok(user) => {
                    ud = UserDetails {
                        is_logged_in: true,
                        user: Some(user),
                    }
                }
                Err(e) => tracing::info!("[user-details]: Failed to get github user: {e}"),
            }
        }
    }

    doc.from_json(&ud, &kind, &value)
}

#[derive(Debug, serde::Serialize, Default)]
struct UserDetails {
    #[serde(rename = "is-logged-in")]
    is_logged_in: bool,
    user: Option<fastn_core::auth::github::GhUserDetails>,
}

