/// currently returns the github user details
pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    #[cfg(feature = "auth")]
    if let Some(session_data) = req_config.request.cookie(fastn_core::auth::COOKIE_NAME) {
        let mut ud = Default::default();

        if !session_data.is_empty() {
            let session_data = fastn_core::auth::utils::decrypt(&session_data)
                .await
                .map_err(|e| {
                    ftd::interpreter::Error::OtherError(format!(
                        "Failed to decrypt session data. {:?}",
                        e
                    ))
                })?;

            #[derive(serde::Deserialize)]
            struct User {
                username: String,
                name: String,
                email: String,
            }

            #[derive(serde::Deserialize)]
            struct SessionData {
                user: User,
            }

            if let Ok(data) = serde_json::from_str::<SessionData>(session_data.as_str()) {
                ud = UserDetails {
                    is_logged_in: true,
                    username: data.user.username,
                    name: data.user.name,
                    email: data.user.email,
                }
            }
        }

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
