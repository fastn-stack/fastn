pub(crate) mod amazon;
pub(crate) mod apple;
pub(crate) mod baidu;
pub(crate) mod bitbucket;
pub(crate) mod config;
pub(crate) mod digitalocean;
pub(crate) mod discord;
pub(crate) mod doorkeeper;
pub(crate) mod dropbox;
pub(crate) mod facebook;
pub(crate) mod github;
pub(crate) mod gitlab;
pub(crate) mod gmail;
pub(crate) mod google;
pub(crate) mod instagram;
pub(crate) mod linkedin;
pub(crate) mod microsoft;
pub(crate) mod okta;
pub(crate) mod pintrest;
pub(crate) mod processor;
pub(crate) mod routes;
pub(crate) mod slack;
pub(crate) mod telegram;
pub(crate) mod tiktok;
pub(crate) mod twitch;
pub(crate) mod twitter;
pub(crate) mod wechat;
pub(crate) mod yahoo;
pub(crate) mod zoho;

pub mod utils;
#[derive(Debug)]
pub(crate) enum AuthProviders {
    GitHub,
    TeleGram,
    Google,
    Discord,
    Slack,
    Amazon,
    Apple,
    Baidu,
    BitBucket,
    DigitalOcean,
    DoorKeeper,
    DropBox,
    Facebook,
    GitLab,
    Instagram,
    LinkedIn,
    Microsoft,
    Okta,
    Pintrest,
    TikTok,
    Twitch,
    Twitter,
    WeChat,
    Yahoo,
    Zoho,
    Gmail,
}

impl AuthProviders {
    const AUTH_ITER: [AuthProviders; 26] = [
        AuthProviders::GitHub,
        AuthProviders::TeleGram,
        AuthProviders::Google,
        AuthProviders::Discord,
        AuthProviders::Slack,
        AuthProviders::Amazon,
        AuthProviders::Apple,
        AuthProviders::Baidu,
        AuthProviders::BitBucket,
        AuthProviders::DigitalOcean,
        AuthProviders::DoorKeeper,
        AuthProviders::DropBox,
        AuthProviders::Facebook,
        AuthProviders::GitLab,
        AuthProviders::Instagram,
        AuthProviders::LinkedIn,
        AuthProviders::Microsoft,
        AuthProviders::Okta,
        AuthProviders::Pintrest,
        AuthProviders::TikTok,
        AuthProviders::Twitch,
        AuthProviders::Twitter,
        AuthProviders::WeChat,
        AuthProviders::Yahoo,
        AuthProviders::Zoho,
        AuthProviders::Gmail,
    ];
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            AuthProviders::GitHub => "github",
            AuthProviders::TeleGram => "telegram",
            AuthProviders::Google => "google",
            AuthProviders::Discord => "discord",
            AuthProviders::Slack => "slack",
            AuthProviders::Amazon => "amazon",
            AuthProviders::Apple => "apple",
            AuthProviders::Baidu => "baidu",
            AuthProviders::BitBucket => "bitbucket",
            AuthProviders::DigitalOcean => "digitalocean",
            AuthProviders::DoorKeeper => "doorkeeper",
            AuthProviders::DropBox => "dropbox",
            AuthProviders::Facebook => "facebook",
            AuthProviders::GitLab => "gitlab",
            AuthProviders::Instagram => "instagram",
            AuthProviders::LinkedIn => "linkedin",
            AuthProviders::Microsoft => "microsoft",
            AuthProviders::Okta => "okta",
            AuthProviders::Pintrest => "pintrest",
            AuthProviders::TikTok => "tiktok",
            AuthProviders::Twitch => "twitch",
            AuthProviders::Twitter => "twitter",
            AuthProviders::WeChat => "wechat",
            AuthProviders::Yahoo => "yahoo",
            AuthProviders::Zoho => "zoho",
            AuthProviders::Gmail => "gmail",
        }
    }
}

pub fn secret_key() -> String {
    match std::env::var("SECRET_KEY") {
        Ok(secret) => secret,
        Err(_e) => {
            println!("WARN: SECRET_KEY not set");
            // TODO: Need to change this approach later
            "FPM_TEMP_SECRET".to_string()
        }
    }
}

// TODO: rename the method later
// bridge between fpm to auth to check
pub async fn get_auth_identities(
    cookies: &std::collections::HashMap<String, String>,
    identities: &[fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    let mut matched_identities: Vec<fpm::user_group::UserIdentity> = vec![];

    let github_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::GitHub.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("github user detail not found in the cookies".to_string())
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
    let telegram_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::TeleGram.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("telegram user detail not found in the cookies".to_string())
        });
    match telegram_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(telegram_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let telegram_ud: telegram::UserDetail =
                    serde_json::from_str(telegram_ud_decrypted.as_str())?;
                matched_identities
                    .extend(telegram::matched_identities(telegram_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "telegram user detail not found in the cookies", err);
        }
    };
    let discord_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Discord.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("discord user detail not found in the cookies".to_string())
        });
    match discord_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(discord_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let discord_ud: discord::UserDetail =
                    serde_json::from_str(discord_ud_decrypted.as_str())?;
                matched_identities
                    .extend(discord::matched_identities(discord_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "discord user detail not found in the cookies", err);
        }
    };
    let twitter_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Twitter.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("twitter user detail not found in the cookies".to_string())
        });

    match twitter_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(twitter_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let twitter_ud: twitter::UserDetail =
                    serde_json::from_str(twitter_ud_decrypted.as_str())?;
                matched_identities
                    .extend(twitter::matched_identities(twitter_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "twitter user detail not found in the cookies", err);
        }
    };
    let amazon_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Amazon.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("amazon user detail not found in the cookies".to_string())
        });
    match amazon_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(amazon_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let amazon_ud: amazon::UserDetail =
                    serde_json::from_str(amazon_ud_decrypted.as_str())?;
                matched_identities.extend(amazon::matched_identities(amazon_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "amazon user detail not found in the cookies", err);
        }
    };
    let facebook_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Facebook.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("facebook user detail not found in the cookies".to_string())
        });
    match facebook_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(facebook_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let facebook_ud: facebook::UserDetail =
                    serde_json::from_str(facebook_ud_decrypted.as_str())?;
                matched_identities
                    .extend(facebook::matched_identities(facebook_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "facebook user detail not found in the cookies", err);
        }
    };
    let gmail_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Gmail.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("gmail user detail not found in the cookies".to_string())
        });
    match gmail_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(gmail_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let gmail_ud: gmail::UserDetail =
                    serde_json::from_str(gmail_ud_decrypted.as_str())?;
                matched_identities.extend(gmail::matched_identities(gmail_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "gmail user detail not found in the cookies", err);
        }
    };
    let slack_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Slack.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("slack user detail not found in the cookies".to_string())
        });
    match slack_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(slack_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let slack_ud: slack::UserDetail =
                    serde_json::from_str(slack_ud_decrypted.as_str())?;
                matched_identities.extend(slack::matched_identities(slack_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "slack user detail not found in the cookies", err);
        }
    };
    let apple_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Apple.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("apple user detail not found in the cookies".to_string())
        });
    match apple_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(apple_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let apple_ud: apple::UserDetail =
                    serde_json::from_str(apple_ud_decrypted.as_str())?;
                matched_identities.extend(apple::matched_identities(apple_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "apple user detail not found in the cookies", err);
        }
    };
    let baidu_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Baidu.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("baidu user detail not found in the cookies".to_string())
        });
    match baidu_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(baidu_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let baidu_ud: baidu::UserDetail =
                    serde_json::from_str(baidu_ud_decrypted.as_str())?;
                matched_identities.extend(baidu::matched_identities(baidu_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "baidu user detail not found in the cookies", err);
        }
    };
    let bitbucket_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::BitBucket.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("bitbucket user detail not found in the cookies".to_string())
        });
    match bitbucket_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(bitbucket_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let bitbucket_ud: bitbucket::UserDetail =
                    serde_json::from_str(bitbucket_ud_decrypted.as_str())?;
                matched_identities
                    .extend(bitbucket::matched_identities(bitbucket_ud, identities).await?);
            }
        }
        Err(err) => {
            format!(
                "{}{}",
                "bitbucket user detail not found in the cookies", err
            );
        }
    };
    let digitalocean_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::DigitalOcean.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError(
                "digitalocean user detail not found in the cookies".to_string(),
            )
        });
    match digitalocean_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(digitalocean_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let digitalocean_ud: digitalocean::UserDetail =
                    serde_json::from_str(digitalocean_ud_decrypted.as_str())?;
                matched_identities
                    .extend(digitalocean::matched_identities(digitalocean_ud, identities).await?);
            }
        }
        Err(err) => {
            format!(
                "{}{}",
                "digitalocean user detail not found in the cookies", err
            );
        }
    };
    let doorkeeper_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::DoorKeeper.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("doorkeeper user detail not found in the cookies".to_string())
        });
    match doorkeeper_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(doorkeeper_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let doorkeeper_ud: doorkeeper::UserDetail =
                    serde_json::from_str(doorkeeper_ud_decrypted.as_str())?;
                matched_identities
                    .extend(doorkeeper::matched_identities(doorkeeper_ud, identities).await?);
            }
        }
        Err(err) => {
            format!(
                "{}{}",
                "doorkeeper user detail not found in the cookies", err
            );
        }
    };
    let dropbox_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::DropBox.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("DropBox user detail not found in the cookies".to_string())
        });
    match dropbox_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(dropbox_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let dropbox_ud: dropbox::UserDetail =
                    serde_json::from_str(dropbox_ud_decrypted.as_str())?;
                matched_identities
                    .extend(dropbox::matched_identities(dropbox_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "dropbox user detail not found in the cookies", err);
        }
    };
    let gitlab_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::GitLab.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("GitLab user detail not found in the cookies".to_string())
        });
    match gitlab_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(gitlab_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let gitlab_ud: gitlab::UserDetail =
                    serde_json::from_str(gitlab_ud_decrypted.as_str())?;
                matched_identities.extend(gitlab::matched_identities(gitlab_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "GitLab user detail not found in the cookies", err);
        }
    };
    let instagram_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Instagram.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("Instagram user detail not found in the cookies".to_string())
        });
    match instagram_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(instagram_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let instagram_ud: instagram::UserDetail =
                    serde_json::from_str(instagram_ud_decrypted.as_str())?;
                matched_identities
                    .extend(instagram::matched_identities(instagram_ud, identities).await?);
            }
        }
        Err(err) => {
            format!(
                "{}{}",
                "Instagram user detail not found in the cookies", err
            );
        }
    };
    let linkedin_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::LinkedIn.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("LinkedIn user detail not found in the cookies".to_string())
        });
    match linkedin_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(linkedin_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let linkedin_ud: linkedin::UserDetail =
                    serde_json::from_str(linkedin_ud_decrypted.as_str())?;
                matched_identities
                    .extend(linkedin::matched_identities(linkedin_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "LinkedIn user detail not found in the cookies", err);
        }
    };
    let microsoft_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Microsoft.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("Microsoft user detail not found in the cookies".to_string())
        });
    match microsoft_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(microsoft_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let microsoft_ud: microsoft::UserDetail =
                    serde_json::from_str(microsoft_ud_decrypted.as_str())?;
                matched_identities
                    .extend(microsoft::matched_identities(microsoft_ud, identities).await?);
            }
        }
        Err(err) => {
            format!(
                "{}{}",
                "Microsoft user detail not found in the cookies", err
            );
        }
    };
    let okta_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Okta.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("Okta user detail not found in the cookies".to_string())
        });
    match okta_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(okta_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let okta_ud: okta::UserDetail = serde_json::from_str(okta_ud_decrypted.as_str())?;
                matched_identities.extend(okta::matched_identities(okta_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "Okta user detail not found in the cookies", err);
        }
    };
    let pintrest_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Pintrest.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("Pintrest user detail not found in the cookies".to_string())
        });
    match pintrest_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(pintrest_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let pintrest_ud: pintrest::UserDetail =
                    serde_json::from_str(pintrest_ud_decrypted.as_str())?;
                matched_identities
                    .extend(pintrest::matched_identities(pintrest_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "Pintrest user detail not found in the cookies", err);
        }
    };
    let tiktok_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::TikTok.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("TikTok user detail not found in the cookies".to_string())
        });
    match tiktok_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(tiktok_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let tiktok_ud: tiktok::UserDetail =
                    serde_json::from_str(tiktok_ud_decrypted.as_str())?;
                matched_identities.extend(tiktok::matched_identities(tiktok_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "TikTok user detail not found in the cookies", err);
        }
    };
    let twitch_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Twitch.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("Twitch user detail not found in the cookies".to_string())
        });
    match twitch_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(twitch_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let twitch_ud: twitch::UserDetail =
                    serde_json::from_str(twitch_ud_decrypted.as_str())?;
                matched_identities.extend(twitch::matched_identities(twitch_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "Twitch user detail not found in the cookies", err);
        }
    };
    let twitter_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Twitter.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("Twitter user detail not found in the cookies".to_string())
        });
    match twitter_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(twitter_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let twitter_ud: twitter::UserDetail =
                    serde_json::from_str(twitter_ud_decrypted.as_str())?;
                matched_identities
                    .extend(twitter::matched_identities(twitter_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "Twitter user detail not found in the cookies", err);
        }
    };
    let wechat_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::WeChat.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("WeChat user detail not found in the cookies".to_string())
        });
    match wechat_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(wechat_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let wechat_ud: wechat::UserDetail =
                    serde_json::from_str(wechat_ud_decrypted.as_str())?;
                matched_identities.extend(wechat::matched_identities(wechat_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "WeChat user detail not found in the cookies", err);
        }
    };
    let yahoo_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Yahoo.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("Yahoo user detail not found in the cookies".to_string())
        });
    match yahoo_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(yahoo_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let yahoo_ud: yahoo::UserDetail =
                    serde_json::from_str(yahoo_ud_decrypted.as_str())?;
                matched_identities.extend(yahoo::matched_identities(yahoo_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "Yahoo user detail not found in the cookies", err);
        }
    };
    let zoho_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Zoho.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("Zoho user detail not found in the cookies".to_string())
        });
    match zoho_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(zoho_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let zoho_ud: zoho::UserDetail = serde_json::from_str(zoho_ud_decrypted.as_str())?;
                matched_identities.extend(zoho::matched_identities(zoho_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "Zoho user detail not found in the cookies", err);
        }
    };
    let google_ud_encrypted = cookies
        .get(fpm::auth::AuthProviders::Google.as_str())
        .ok_or_else(|| {
            fpm::Error::GenericError("Google user detail not found in the cookies".to_string())
        });
    match google_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(google_ud_decrypted) = utils::decrypt_str(encrypt_str).await {
                let google_ud: google::UserDetail =
                    serde_json::from_str(google_ud_decrypted.as_str())?;
                matched_identities.extend(google::matched_identities(google_ud, identities).await?);
            }
        }
        Err(err) => {
            format!("{}{}", "Google user detail not found in the cookies", err);
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
