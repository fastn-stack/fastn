// Return the login information of the user
pub fn user_details<'a>(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    let is_login = match &config.request {
        Some(req) => {
            req.cookie(fpm::auth::AuthProviders::GitHub.as_str())
                .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::TeleGram.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Discord.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Slack.as_str())
                    .is_some()
                || req
                    .cookie(fpm::auth::AuthProviders::Google.as_str())
                    .is_some()
        }
        None => false,
    };

    #[derive(Debug, serde::Serialize)]
    struct UserDetails {
        #[serde(rename = "is-login")]
        is_login: bool,
    }
    let ud = UserDetails { is_login };
    doc.from_json(&ud, section)
}
