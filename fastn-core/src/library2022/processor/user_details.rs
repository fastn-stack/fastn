/// currently returns the github user details
pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let mut ud = UserDetails {
        is_login: false,
        user: None,
    };

    if let Some(gh_cookie) = req_config.request.cookie("github") {
        if let Ok(user_detail) = fastn_core::auth::utils::decrypt_str(&gh_cookie)
            .await
            .map_err(|e| eprintln!("Failed to decrypt cookie: {e}"))
            .and_then(|decrypted_cookie| {
                serde_json::from_str::<fastn_core::auth::github::UserDetail>(
                    decrypted_cookie.as_str(),
                )
                .map_err(|e| eprintln!("Serde deserialization failed: {e}"))
            })
        {
            match fastn_core::auth::github::apis::user_details(user_detail.access_token.as_str())
                .await
            {
                Ok(user) => {
                    ud = UserDetails {
                        is_login: true,
                        user: Some(user),
                    }
                }
                Err(e) => eprintln!("Failed to get github user: {e}"),
            }
        }
    }

    doc.from_json(&ud, &kind, &value)
}

#[derive(Debug, serde::Serialize)]
struct UserDetails {
    #[serde(rename = "is-login")]
    is_login: bool,
    user: Option<fastn_core::auth::github::apis::GhUserDetails>,
}
