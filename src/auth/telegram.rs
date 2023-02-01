// TODO: This has be set while creating the Telegram OAuth Application
pub const CALLBACK_URL: &str = "/auth/telegram/callback/";
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserDetail {
    pub user_id: String,
    pub user_name: String,
    pub token: String,
}
// route: /auth/login/

pub async fn login(req: actix_web::HttpRequest) -> fastn::Result<fastn::http::Response> {
    // This method will be called to open telegram login dialogue
    let redirect_url: String = format!(
        "{}://{}{}",
        req.connection_info().scheme(),
        req.connection_info().host(),
        CALLBACK_URL
    );
    let login_widget_url = "https://telegram.org/js/telegram-widget.js?21";

    let telegram_body = format!(
        r#"{}"{}"{}"{}"{}"{}"{}"#,
        "<html>
            <head><title>FTD</title></head>
            <body><script async src=",
        login_widget_url,
        " data-telegram-login=",
        match std::env::var("TELEGRAM_BOT_NAME") {
            Ok(val) => val,
            Err(e) => format!("{}{}", "TELEGRAM_BOT_NAME not set in env ", e),
        },
        r#"  data-size="medium" data-userpic="false" data-request-access="write" data-auth-url="#,
        redirect_url,
        "></script></body></html>"
    );

    Ok(actix_web::HttpResponse::Ok().body(telegram_body))
}

// route: /auth/telegram/callback/
// In this API we are accessing
// the token and setting it to cookies
pub async fn token(req: actix_web::HttpRequest) -> fastn::Result<actix_web::HttpResponse> {
    #[derive(Debug, serde::Deserialize)]
    pub struct QueryParams {
        pub id: String,
        pub first_name: String,
        pub last_name: String,
        pub username: String,
        pub auth_date: String,
        pub hash: String,
    }
    let query = actix_web::web::Query::<QueryParams>::from_query(req.query_string())?.0;
    let user_detail_obj: UserDetail = UserDetail {
        user_id: query.id,
        token: query.hash,
        user_name: query.username,
    };
    let user_detail_str = serde_json::to_string(&user_detail_obj)?;
    return Ok(actix_web::HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build(
                fastn::auth::AuthProviders::TeleGram.as_str(),
                fastn::auth::utils::encrypt_str(&user_detail_str).await,
            )
            .domain(fastn::auth::utils::domain(req.connection_info().host()))
            .path("/")
            .permanent()
            .secure(true)
            .finish(),
        )
        .append_header((actix_web::http::header::LOCATION, "/".to_string()))
        .finish());
}
// it returns identities which matches to given input
pub async fn matched_identities(
    ud: UserDetail,
    identities: &[fastn::user_group::UserIdentity],
) -> fastn::Result<Vec<fastn::user_group::UserIdentity>> {
    let telegram_identities = identities
        .iter()
        .filter(|identity| identity.key.starts_with("telegram"))
        .collect::<Vec<&fastn::user_group::UserIdentity>>();

    if telegram_identities.is_empty() {
        return Ok(vec![]);
    }

    let mut matched_identities = vec![];
    // matched_telegram_admin
    matched_identities.extend(matched_telegram_admin(&ud, telegram_identities.as_slice()).await?);
    matched_identities
        .extend(matched_telegram_group_member(&ud, telegram_identities.as_slice()).await?);
    matched_identities
        .extend(matched_telegram_channel_member(&ud, telegram_identities.as_slice()).await?);
    Ok(matched_identities)
}

pub async fn matched_telegram_admin(
    ud: &UserDetail,
    identities: &[&fastn::user_group::UserIdentity],
) -> fastn::Result<Vec<fastn::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut matched_groups: Vec<String> = vec![];
    let group_list = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("telegram-admin") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();
    if group_list.is_empty() {
        return Ok(vec![]);
    }

    for group_name in group_list.iter() {
        let group_administrator_list: Vec<String> = apis::group_administrators(group_name).await?;
        if group_administrator_list.contains(&ud.user_name) {
            matched_groups.push(group_name.to_string());
        }
        // TODO:
        // Return Error if group administrator does not exist
    }
    // filter the user joined teams with input
    Ok(matched_groups
        .into_iter()
        .map(|group_name| fastn::user_group::UserIdentity {
            key: "telegram-admin".to_string(),
            value: group_name,
        })
        .collect())
}
pub async fn matched_telegram_group_member(
    ud: &UserDetail,
    identities: &[&fastn::user_group::UserIdentity],
) -> fastn::Result<Vec<fastn::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut matched_groups: Vec<String> = vec![];
    let group_list = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("telegram-group") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();
    if group_list.is_empty() {
        return Ok(vec![]);
    }

    for group_name in group_list.iter() {
        let group_member: String = apis::get_member(group_name, ud.user_id.as_str()).await?;
        dbg!(&group_member);
        if group_member.eq(&ud.user_name) {
            matched_groups.push(group_name.to_string());
        }
        // TODO:
        // Return Error if group administrator does not exist
    }
    // filter the user joined teams with input
    Ok(matched_groups
        .into_iter()
        .map(|group_name| fastn::user_group::UserIdentity {
            key: "telegram-group".to_string(),
            value: group_name,
        })
        .collect())
}
pub async fn matched_telegram_channel_member(
    ud: &UserDetail,
    identities: &[&fastn::user_group::UserIdentity],
) -> fastn::Result<Vec<fastn::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut matched_groups: Vec<String> = vec![];
    let group_list = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("telegram-channel") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();
    if group_list.is_empty() {
        return Ok(vec![]);
    }

    for group_name in group_list.iter() {
        let group_member: String = apis::get_member(group_name, ud.user_id.as_str()).await?;
        dbg!(&group_member);
        if group_member.eq(&ud.user_name) {
            matched_groups.push(group_name.to_string());
        }
        // TODO:
        // Return Error if group administrator does not exist
    }
    // filter the user joined teams with input
    Ok(matched_groups
        .into_iter()
        .map(|group_name| fastn::user_group::UserIdentity {
            key: "telegram-channel".to_string(),
            value: group_name,
        })
        .collect())
}

pub mod apis {
    #[derive(Debug, serde::Deserialize)]
    pub struct TelegramAdminResp {
        pub ok: bool,
        pub result: Vec<TelegramUser>,
    }
    #[derive(Debug, serde::Deserialize)]
    pub struct TelegramUser {
        pub user: TelegramUserObj,
    }
    #[derive(Debug, serde::Deserialize)]
    pub struct TelegramUserObj {
        pub username: String,
    }
    #[derive(Debug, serde::Deserialize)]
    pub struct TelegramMemberResp {
        pub ok: bool,
        pub result: TelegramUser,
    }
    // TODO: API to get bot informations
    // API Docs: https://core.telegram.org/bots

    pub async fn group_administrators(group_name: &str) -> fastn::Result<Vec<String>> {
        // API Docs: https://api.telegram.org/bot{telegram-bot-token}/getChatAdministrators?chat_id="@group_name"
        // TODO: Handle paginated response

        let group_administrator: TelegramAdminResp = group_administrator_api(
            format!(
                "{}{}/GetChatAdministrators?chat_id={}",
                "https://api.telegram.org/bot",
                match std::env::var("TELEGRAM_BOT_TOKEN") {
                    Ok(val) => val,
                    Err(e) => format!("{}{}", "TELEGRAM_BOT_TOKEN not set in env ", e),
                },
                group_name
            )
            .as_str(),
        )
        .await?;
        Ok(group_administrator
            .result
            .into_iter()
            .map(|x| x.user.username)
            .collect())
    }
    pub async fn get_member(group_name: &str, user_id: &str) -> fastn::Result<String> {
        // API Docs: https://api.telegram.org/bot{telegram-bot-token}/getChatMember?chat_id="@group_name"&user_id="user_id"
        // TODO: Handle paginated response

        let member: TelegramMemberResp = get_api(
            format!(
                "{}{}/GetChatMember?chat_id={}&user_id={}",
                "https://api.telegram.org/bot",
                match std::env::var("TELEGRAM_BOT_TOKEN") {
                    Ok(val) => val,
                    Err(e) => format!("{}{}", "TELEGRAM_BOT_TOKEN not set in env ", e),
                },
                group_name,
                user_id
            )
            .as_str(),
        )
        .await?;
        Ok(member.result.user.username)
    }

    pub async fn group_administrator_api(url: &str) -> fastn::Result<TelegramAdminResp> {
        let response = reqwest::Client::new()
            .get(url)
            .header(reqwest::header::ACCEPT, "application/json")
            .header(
                reqwest::header::USER_AGENT,
                reqwest::header::HeaderValue::from_static("fastn"),
            )
            .send()
            .await?;

        if !response.status().eq(&reqwest::StatusCode::OK) {
            return Err(fastn::Error::APIResponseError(format!(
                "Telegram API ERROR: {}",
                url
            )));
        }

        Ok(response.json::<TelegramAdminResp>().await?)
    }
    pub async fn get_api(url: &str) -> fastn::Result<TelegramMemberResp> {
        let response = reqwest::Client::new()
            .get(url)
            .header(reqwest::header::ACCEPT, "application/json")
            .header(
                reqwest::header::USER_AGENT,
                reqwest::header::HeaderValue::from_static("fastn"),
            )
            .send()
            .await?;

        if !response.status().eq(&reqwest::StatusCode::OK) {
            return Err(fastn::Error::APIResponseError(format!(
                "Telegram API ERROR: {}",
                url
            )));
        }

        Ok(response.json::<TelegramMemberResp>().await?)
    }
}

pub mod utils {

    // Lazy means a value which initialize at the first time access
    // we have to access it before using it and make sure to use it while starting a server
    // TODO: they should be configured with auth feature flag
    // if feature flag auth is enabled Make sure that before accessing in the API these variable
    // are set
}
