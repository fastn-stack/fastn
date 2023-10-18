// pub(crate) mod amazon;
// pub(crate) mod apple;
// pub(crate) mod baidu;
// pub(crate) mod bitbucket;
pub(crate) mod config;
// pub(crate) mod digitalocean;
// pub(crate) mod discord;
// pub(crate) mod doorkeeper;
// pub(crate) mod dropbox;
// pub(crate) mod facebook;
pub(crate) mod github;
// pub(crate) mod gitlab;
// pub(crate) mod gmail;
// pub(crate) mod google;
// pub(crate) mod instagram;
// pub(crate) mod linkedin;
// pub(crate) mod microsoft;
// pub(crate) mod okta;
// pub(crate) mod pintrest;
pub(crate) mod processor;
pub(crate) mod routes;
// pub(crate) mod slack;
// pub(crate) mod telegram;
// pub(crate) mod tiktok;
// pub(crate) mod twitch;
// pub(crate) mod twitter;
// pub(crate) mod wechat;
// pub(crate) mod yahoo;
// pub(crate) mod zoho;

pub mod utils;
#[derive(Debug)]
pub(crate) enum AuthProviders {
    GitHub,
    // TeleGram,
    // Google,
    // Discord,
    // Slack,
    // Amazon,
    // Apple,
    // Baidu,
    // BitBucket,
    // DigitalOcean,
    // DoorKeeper,
    // DropBox,
    // Facebook,
    // GitLab,
    // Instagram,
    // LinkedIn,
    // Microsoft,
    // Okta,
    // Pintrest,
    // TikTok,
    // Twitch,
    // Twitter,
    // WeChat,
    // Yahoo,
    // Zoho,
    // Gmail,
}

impl AuthProviders {
    pub(crate) const AUTH_ITER: [AuthProviders; 1] = [
        AuthProviders::GitHub,
        // AuthProviders::TeleGram,
        // AuthProviders::Google,
        // AuthProviders::Discord,
        // AuthProviders::Slack,
        // AuthProviders::Amazon,
        // AuthProviders::Apple,
        // AuthProviders::Baidu,
        // AuthProviders::BitBucket,
        // AuthProviders::DigitalOcean,
        // AuthProviders::DoorKeeper,
        // AuthProviders::DropBox,
        // AuthProviders::Facebook,
        // AuthProviders::GitLab,
        // AuthProviders::Instagram,
        // AuthProviders::LinkedIn,
        // AuthProviders::Microsoft,
        // AuthProviders::Okta,
        // AuthProviders::Pintrest,
        // AuthProviders::TikTok,
        // AuthProviders::Twitch,
        // AuthProviders::Twitter,
        // AuthProviders::WeChat,
        // AuthProviders::Yahoo,
        // AuthProviders::Zoho,
        // AuthProviders::Gmail,
    ];
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            AuthProviders::GitHub => "github",
            // AuthProviders::TeleGram => "telegram",
            // AuthProviders::Google => "google",
            // AuthProviders::Discord => "discord",
            // AuthProviders::Slack => "slack",
            // AuthProviders::Amazon => "amazon",
            // AuthProviders::Apple => "apple",
            // AuthProviders::Baidu => "baidu",
            // AuthProviders::BitBucket => "bitbucket",
            // AuthProviders::DigitalOcean => "digitalocean",
            // AuthProviders::DoorKeeper => "doorkeeper",
            // AuthProviders::DropBox => "dropbox",
            // AuthProviders::Facebook => "facebook",
            // AuthProviders::GitLab => "gitlab",
            // AuthProviders::Instagram => "instagram",
            // AuthProviders::LinkedIn => "linkedin",
            // AuthProviders::Microsoft => "microsoft",
            // AuthProviders::Okta => "okta",
            // AuthProviders::Pintrest => "pintrest",
            // AuthProviders::TikTok => "tiktok",
            // AuthProviders::Twitch => "twitch",
            // AuthProviders::Twitter => "twitter",
            // AuthProviders::WeChat => "wechat",
            // AuthProviders::Yahoo => "yahoo",
            // AuthProviders::Zoho => "zoho",
            // AuthProviders::Gmail => "gmail",
        }
    }

    pub(crate) fn from_str(s: &str) -> Self {
        match s {
            "github" => AuthProviders::GitHub,
            // "telegram" => AuthProviders::TeleGram,
            // "google" => AuthProviders::Google,
            // "discord" => AuthProviders::Discord,
            // "slack" => AuthProviders::Slack,
            _ => panic!("Invalid auth provider {}", s),
        }
    }
}

pub fn secret_key() -> String {
    match std::env::var("SECRET_KEY") {
        Ok(secret) => secret,
        Err(_e) => {
            println!("WARN: SECRET_KEY not set");
            // TODO: Need to change this approach later
            "FASTN_TEMP_SECRET".to_string()
        }
    }
}

/// will fetch out the decrypted user data from cookies
/// and return it as string
/// if no cookie wrt to platform found it throws an error
pub async fn get_user_data_from_cookies(
    platform: &str,
    requested_field: &str,
    cookies: &std::collections::HashMap<String, String>,
) -> fastn_core::Result<Option<String>> {
    let ud_encrypted = cookies.get(platform).ok_or_else(|| {
        fastn_core::Error::GenericError(format!(
            "user detail not found for platform {} in the cookies",
            platform
        ))
    });
    match ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                match fastn_core::auth::AuthProviders::from_str(platform) {
                    fastn_core::auth::AuthProviders::GitHub => {
                        let github_ud: github::UserDetail =
                            serde_json::from_str(ud_decrypted.as_str())?;
                        return match requested_field {
                            "username" | "user_name" | "user-name" => Ok(Some(github_ud.user_name)),
                            "token" => Ok(Some(github_ud.token)),
                            _ => Err(fastn_core::Error::GenericError(format!(
                                "invalid field {} requested for platform {}",
                                requested_field, platform
                            ))),
                        };
                    }
                }
            }
        }
        Err(err) => {
            // Debug out the error and return None
            let error_msg = format!("User data error: {}", err);
            dbg!(&error_msg);
        }
    };
    Ok(None)
}

// TODO: rename the method later
// bridge between fastn_core to auth to check
pub async fn get_auth_identities(
    cookies: &std::collections::HashMap<String, String>,
    identities: &[fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    let mut matched_identities: Vec<fastn_core::user_group::UserIdentity> = vec![];

    let github_ud_encrypted = cookies
        .get(fastn_core::auth::AuthProviders::GitHub.as_str())
        .ok_or_else(|| {
            fastn_core::Error::GenericError(
                "github user detail not found in the cookies".to_string(),
            )
        });
    match github_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(github_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let github_ud: github::UserDetail =
                    serde_json::from_str(github_ud_decrypted.as_str())?;
                matched_identities.extend(github::matched_identities(github_ud, identities).await?);
            }
        }
        Err(err) => {
            // TODO: What to do with this error
            format!("{}{}", "github user detail not found in the cookies", err);
        }
    };

    // TODO: which API to from which platform based on identity
    // identity can be github-*, discord-*, and etc...
    //let matched_identities = github::matched_identities(token.as_str(), identities).await?;

    //TODO: Call discord::matched_identities
    //TODO: Call google::matched_identities
    //TODO: Call twitter::matched_identities
    Ok(matched_identities)
}
