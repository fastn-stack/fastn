// TODO: This has be set while creating the Discord OAuth Application
pub const CALLBACK_URL: &str = "/auth/discord/callback/";
pub const AUTH_URL: &str = "https://discord.com/oauth2/authorize";
pub const TOKEN_URL: &str = "https://discord.com/api/oauth2/token";
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserDetail {
    pub token: String,
    pub user_name: String,
    pub user_id: String,
}
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DiscordAuthReq {
    pub client_secret: String,
    pub client_id: String,
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
}
pub(crate) enum DiscordScopes {
    Identify,
    Guilds,
    GuildsMembersRead,
}

impl DiscordScopes {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            DiscordScopes::Identify => "identify",
            DiscordScopes::Guilds => "guilds",
            DiscordScopes::GuildsMembersRead => "guilds.members.read",
        }
    }
}
// route: /auth/login/
pub async fn login(req: actix_web::HttpRequest) -> fpm::Result<fpm::http::Response> {
    // Discord will be redirect to this url after login process completed

    let redirect_url: String = format!(
        "{}://{}{}",
        req.connection_info().scheme(),
        req.connection_info().host(),
        CALLBACK_URL
    );
    let client_id = match std::env::var("DISCORD_CLIENT_ID") {
        Ok(id) => id,
        Err(_e) => {
            return Err(fpm::Error::APIResponseError(
                "WARN: FPM_TEMP_DISCORD_CLIENT_ID not set.".to_string(),
            ));
            // TODO: Need to change this approach later
            //"FPM_TEMP_DISCORD_CLIENT_ID".to_string()
        }
    };
    let discord_auth_url = format!(
        "{}{}{}{}{}{}{} {} {}",
        AUTH_URL,
        "?client_id=",
        client_id,
        "&redirect_uri=",
        redirect_url,
        "&response_type=code&scope=",
        DiscordScopes::Identify.as_str(),
        DiscordScopes::Guilds.as_str(),
        DiscordScopes::GuildsMembersRead.as_str()
    );
    // send redirect to /auth/discord/callback/
    Ok(actix_web::HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, discord_auth_url))
        .finish())
}

// route: /auth/discord/callback/
// In this API we are accessing
// the token and setting it to cookies
pub async fn callback(req: actix_web::HttpRequest) -> fpm::Result<actix_web::HttpResponse> {
    #[derive(Debug, serde::Deserialize)]
    pub struct QueryParams {
        pub code: String,
    }

    let query = actix_web::web::Query::<QueryParams>::from_query(req.query_string())?.0;
    let redirect_url = format!(
        "{}://{}{}",
        req.connection_info().scheme(),
        req.connection_info().host(),
        CALLBACK_URL
    );
    let discord_auth =
        apis::discord_token(TOKEN_URL, redirect_url.as_str(), query.code.as_str()).await;
    match discord_auth {
        Ok(access_token) => {
            dbg!(&access_token);
            let (user_name, user_id) = apis::user_details(&access_token).await?;
            let user_detail_obj: UserDetail = UserDetail {
                token: access_token.to_owned(),
                user_name,
                user_id,
            };
            let user_detail_str = serde_json::to_string(&user_detail_obj)?;

            return Ok(actix_web::HttpResponse::Found()
                .cookie(
                    actix_web::cookie::Cookie::build(
                        fpm::auth::AuthProviders::Discord.as_str(),
                        fpm::auth::utils::encrypt_str(&user_detail_str).await,
                    )
                    .domain(fpm::auth::utils::domain(req.connection_info().host()))
                    .path("/")
                    .permanent()
                    .finish(),
                )
                .append_header((actix_web::http::header::LOCATION, "/".to_string()))
                .finish());
        }
        Err(err) => Ok(actix_web::HttpResponse::InternalServerError().body(err.to_string())),
    }
}
// it returns identities which matches to given input
pub async fn matched_identities(
    ud: UserDetail,
    identities: &[fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    let discord_identities = identities
        .iter()
        .filter(|identity| identity.key.starts_with("discord"))
        .collect::<Vec<&fpm::user_group::UserIdentity>>();

    if discord_identities.is_empty() {
        return Ok(vec![]);
    }

    let mut matched_identities = vec![];
    // matched_user_servers
    matched_identities.extend(matched_user_servers(&ud, discord_identities.as_slice()).await?);
    // matched_thread_members
    matched_identities.extend(matched_thread_members(&ud, discord_identities.as_slice()).await?);
    //matched_event_members
    matched_identities.extend(matched_event_members(&ud, discord_identities.as_slice()).await?);
    //matched_member_permission
    matched_identities.extend(matched_member_permission(&ud, discord_identities.as_slice()).await?);
    Ok(matched_identities)
}
pub async fn matched_user_servers(
    ud: &UserDetail,
    identities: &[&fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    use itertools::Itertools;

    let user_servers = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("discord-server") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if user_servers.is_empty() {
        return Ok(vec![]);
    }
    let user_joined_servers = apis::user_servers(ud.token.as_str()).await?;
    // filter the user joined servers with input
    Ok(user_joined_servers
        .into_iter()
        .filter(|user_server| user_servers.contains(&user_server.as_str()))
        .map(|repo| fpm::user_group::UserIdentity {
            key: "discord-server".to_string(),
            value: repo,
        })
        .collect())
}
pub async fn matched_thread_members(
    ud: &UserDetail,
    identities: &[&fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut user_joined_threads: Vec<String> = vec![];
    let user_threads = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("discord-thread") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if user_threads.is_empty() {
        return Ok(vec![]);
    }
    for thread in user_threads.iter() {
        let thread_member_list: Vec<String> = apis::thread_members(thread).await?;
        if thread_member_list.contains(&ud.user_id) {
            user_joined_threads.push(thread.to_string());
        }
        // TODO:
        // Return Error if user thread does not exist
    }
    // filter the user joined threads with input
    Ok(user_joined_threads
        .into_iter()
        .map(|thread| fpm::user_group::UserIdentity {
            key: "discord-thread".to_string(),
            value: thread,
        })
        .collect())
}
pub async fn matched_event_members(
    ud: &UserDetail,
    identities: &[&fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut user_joined_events: Vec<String> = vec![];
    let user_events = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("discord-event") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if user_events.is_empty() {
        return Ok(vec![]);
    }
    for event in user_events.iter() {
        let event_member_list: Vec<String> = apis::event_members(event).await?;
        if event_member_list.contains(&ud.user_id) {
            user_joined_events.push(event.to_string());
        }
        // TODO:
        // Return Error if event member does not exist
    }
    // filter the user joined events with input
    Ok(user_joined_events
        .into_iter()
        .map(|event| fpm::user_group::UserIdentity {
            key: "discord-event".to_string(),
            value: event,
        })
        .collect())
}
pub async fn matched_member_permission(
    ud: &UserDetail,
    identities: &[&fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    use itertools::Itertools;
    let user_roles = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("discord-permission") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if user_roles.is_empty() {
        return Ok(vec![]);
    }
    // filter the user assigned roles with input
    let member_role_list = apis::member_roles(ud.user_id.as_str()).await?;
    Ok(member_role_list
        .into_iter()
        .filter(|user_role| user_roles.contains(&user_role.as_str()))
        .map(|permission| fpm::user_group::UserIdentity {
            key: "discord-permission".to_string(),
            value: permission,
        })
        .collect())
}
pub mod apis {
    #[derive(serde::Deserialize)]
    pub struct DiscordAuthResp {
        pub access_token: String,
    }
    // TODO: API to get user detail.
    // API Docs: https://discord.com/developers/docs/getting-started
    //API EndPoints: https://github.com/GregTCLTK/Discord-Api-Endpoints/blob/master/Endpoints.md
    // TODO: It can be stored in the request cookies
    pub async fn user_details(token: &str) -> fpm::Result<(String, String)> {
        // API Docs: https://discord.com/api/users/@me
        #[derive(serde::Deserialize)]
        struct UserDetails {
            username: String,
            id: String,
        }
        let user_obj: UserDetails = fpm::auth::utils::get_api(
            "https://discord.com/api/users/@me",
            format!("{} {}", "Bearer", token).as_str(),
        )
        .await?;

        Ok((user_obj.username, user_obj.id))
    }

    //This API will only be used to get access token for discord
    pub async fn discord_token(url: &str, redirect_url: &str, code: &str) -> fpm::Result<String> {
        let client_id = match std::env::var("DISCORD_CLIENT_ID") {
            Ok(id) => id,
            Err(_e) => {
                return Err(fpm::Error::APIResponseError(
                    "WARN: DISCORD_CLIENT_ID not set.".to_string(),
                ));
            }
        };
        let client_secret = match std::env::var("DISCORD_CLIENT_SECRET") {
            Ok(secret) => secret,
            Err(_e) => {
                return Err(fpm::Error::APIResponseError(
                    "WARN: DISCORD_SECRET not set.".to_string(),
                ));
            }
        };
        let mut map: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
        map.insert("client_secret", client_secret.as_str());
        map.insert("client_id", client_id.as_str());
        map.insert("grant_type", "authorization_code");
        map.insert("code", code);
        map.insert("redirect_uri", redirect_url);

        let response = reqwest::Client::new().post(url).form(&map).send().await?;

        if !response.status().eq(&reqwest::StatusCode::OK) {
            return Err(fpm::Error::APIResponseError(format!(
                "DISCORD-API-ERROR: {}, Error: {}",
                url,
                response.text().await?
            )));
        }
        let auth_obj = response.json::<DiscordAuthResp>().await?;
        Ok(auth_obj.access_token)
    }
    pub async fn user_servers(token: &str) -> fpm::Result<Vec<String>> {
        // API Docs: https://discord.com/api/users/@me/guilds
        // TODO: Handle paginated response

        #[derive(Debug, serde::Deserialize)]
        struct UserGuilds {
            name: String,
        }
        let user_server_list: Vec<UserGuilds> = fpm::auth::utils::get_api(
            format!("{}?limit=100", "https://discord.com/api/users/@me/guilds").as_str(),
            format!("{} {}", "Bearer", token).as_str(),
        )
        .await?;
        Ok(user_server_list.into_iter().map(|x| x.name).collect())
    }
    pub async fn thread_members(thread_id: &str) -> fpm::Result<Vec<String>> {
        // API Docs: https://discord.com/api/channels/{thread-id}/thread-members
        // TODO: Handle paginated response
        let discord_bot_id = match std::env::var("DISCORD_BOT_ID") {
            Ok(id) => id,
            Err(_e) => {
                return Err(fpm::Error::APIResponseError(
                    "WARN: DISCORD_BOT_ID not set.".to_string(),
                ));
            }
        };
        #[derive(Debug, serde::Deserialize)]
        struct ThreadMembers {
            user_id: String,
        }
        let thread_member_list: Vec<ThreadMembers> = fpm::auth::utils::get_api(
            format!(
                "{}{}{}?limit=100",
                "https://discord.com/api/channels/", thread_id, "/thread-members"
            )
            .as_str(),
            format!("{} {}", "Bot", discord_bot_id).as_str(),
        )
        .await?;
        Ok(thread_member_list.into_iter().map(|x| x.user_id).collect())
    }
    pub async fn event_members(event_id: &str) -> fpm::Result<Vec<String>> {
        // API Docs: https://discord.com/api/guilds/{guild-id}/scheduled-events/{event-id}/users
        // TODO: Handle paginated response
        let discord_bot_id = match std::env::var("DISCORD_BOT_ID") {
            Ok(id) => id,
            Err(_e) => {
                return Err(fpm::Error::APIResponseError(
                    "WARN: DISCORD_BOT_ID not set.".to_string(),
                ));
            }
        };
        let discord_guild_id = match std::env::var("DISCORD_GUILD_ID") {
            Ok(id) => id,
            Err(_e) => {
                return Err(fpm::Error::APIResponseError(
                    "WARN: DISCORD_GUILD_ID not set.".to_string(),
                ));
            }
        };
        #[derive(Debug, serde::Deserialize)]
        struct EventMembers {
            user: EventMemberObj,
        }
        #[derive(Debug, serde::Deserialize)]
        struct EventMemberObj {
            id: String,
        }
        let event_member_list: Vec<EventMembers> = fpm::auth::utils::get_api(
            format!(
                "{}{}{}{}{}?limit=100",
                "https://discord.com/api/guilds/",
                discord_guild_id,
                "/scheduled-events/",
                event_id,
                "/users"
            )
            .as_str(),
            format!("{} {}", "Bot", discord_bot_id).as_str(),
        )
        .await?;
        Ok(event_member_list.into_iter().map(|x| x.user.id).collect())
    }
    pub async fn member_roles(user_id: &str) -> fpm::Result<Vec<String>> {
        // API Docs: https://discord.com/api/guilds/{guild-id}/members/{user-id}
        let discord_bot_id = match std::env::var("DISCORD_BOT_ID") {
            Ok(id) => id,
            Err(_e) => {
                return Err(fpm::Error::APIResponseError(
                    "WARN: DISCORD_BOT_ID not set.".to_string(),
                ));
            }
        };
        let discord_guild_id = match std::env::var("DISCORD_GUILD_ID") {
            Ok(id) => id,
            Err(_e) => {
                return Err(fpm::Error::APIResponseError(
                    "WARN: DISCORD_GUILD_ID not set.".to_string(),
                ));
            }
        };
        #[derive(Debug, serde::Deserialize)]
        struct MemberRoles {
            roles: Vec<String>,
        }
        let member_roles: MemberRoles = fpm::auth::utils::get_api(
            format!(
                "{}{}{}{}",
                "https://discord.com/api/guilds/", discord_guild_id, "/members/", user_id
            )
            .as_str(),
            format!("{} {}", "Bot", discord_bot_id).as_str(),
        )
        .await?;
        Ok(member_roles.roles)
    }
}
