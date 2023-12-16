/// currently returns the github user details
pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let mut ud = Default::default();

    if let Some(session_id) = req_config.request.cookie(fastn_core::auth::COOKIE_NAME) {
        if !session_id.is_empty() {
            let session_id = <uuid::Uuid as std::str::FromStr>::from_str(session_id.as_str())
                .map_err(|e| {
                    ftd::interpreter::Error::OtherError(format!(
                        "Failed to parse uuid from string: {e}"
                    ))
                })?;

            if let Ok(user) = fastn_core::auth::get_authenticated_user(&session_id).await {
                ud = UserDetails {
                    is_logged_in: true,
                    username: user.username,
                    // TODO: workaround until we have a step in signup process where we ask for their
                    // name and email if OAuth provider returns null
                    name: user.name.unwrap_or("".to_string()),
                    email: user.email.unwrap_or("".to_string()),
                }
            }
        }
    }

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
