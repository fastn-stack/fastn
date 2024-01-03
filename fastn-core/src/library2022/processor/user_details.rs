/// currently returns the github user details
pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    #[cfg(feature = "auth")]
    if let Some(session_id) = req_config.request.cookie(fastn_core::auth::COOKIE_NAME) {
        let mut ud = Default::default();

        if !session_id.is_empty() {
            let session_id: i32 = session_id.parse().map_err(|e| {
                ftd::interpreter::Error::OtherError(format!("Failed to parse id from string: {e}"))
            })?;

            if let Ok((user, email)) =
                fastn_core::auth::get_authenticated_user_with_email(&session_id).await
            {
                ud = UserDetails {
                    is_logged_in: true,
                    username: user.username,
                    name: user.name,
                    email,
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
