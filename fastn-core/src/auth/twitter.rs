// TODO: This has be set while creating the Discord OAuth Application
pub const CALLBACK_URL: &str = "/auth/twitter/callback/";
pub const AUTH_URL: &str = "https://twitter.com/i/oauth2/authorize";
pub const TOKEN_URL: &str = "https://api.twitter.com/2/oauth2/token";
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserDetail {
    pub token: String,
    pub user_name: String,
    pub user_id: String,
}

pub(crate) enum TwitterScopes {
    ReadTweet,
    WriteTweet,
    ModerateTweet,
    ReadUsers,
    ReadFollows,
    WriteFollows,
    AccessOffline,
    ReadSpace,
    ReadMute,
    WriteMute,
    ReadLike,
    WriteLike,
    ReadBlock,
    WriteBlock,
    ReadBookmark,
    WriteBookmark,
    WriteList,
    ReadList,
}

impl TwitterScopes {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            TwitterScopes::ReadTweet => "tweet.read",
            TwitterScopes::WriteTweet => "tweet.write",
            TwitterScopes::ModerateTweet => "tweet.moderate.write",
            TwitterScopes::ReadUsers => "users.read",
            TwitterScopes::ReadFollows => "follows.read",
            TwitterScopes::WriteFollows => "follows.write",
            TwitterScopes::AccessOffline => "offline.access",
            TwitterScopes::ReadSpace => "space.read",
            TwitterScopes::ReadMute => "mute.read",
            TwitterScopes::WriteMute => "mute.write",
            TwitterScopes::ReadLike => "like.read",
            TwitterScopes::WriteLike => "like.write",
            TwitterScopes::ReadBlock => "block.read",
            TwitterScopes::WriteBlock => "block.write",
            TwitterScopes::ReadBookmark => "bookmark.read",
            TwitterScopes::WriteBookmark => "bookmark.write",
            TwitterScopes::WriteList => "list.read",
            TwitterScopes::ReadList => "list.write",
        }
    }
}
// route: /auth/login/
pub async fn login(req: actix_web::HttpRequest) -> fastn_core::Result<fastn_core::http::Response> {
    // Twitter will be redirect to this url after login process completed

    let redirect_url: String = format!(
        "{}://{}{}",
        req.connection_info().scheme(),
        req.connection_info().host(),
        CALLBACK_URL
    );
    let client_id = match std::env::var("TWITTER_CLIENT_ID") {
        Ok(id) => id,
        Err(_e) => {
            return Err(fastn_core::Error::APIResponseError(
                "WARN: FASTN_TEMP_TWITTER_CLIENT_ID not set.".to_string(),
            ));
            // TODO: Need to change this approach later
            //"FASTN_TEMP_TWITTER_CLIENT_ID".to_string()
        }
    };
    let twitter_auth_url = format!(
        "{}{}{}{}{}{}{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
        AUTH_URL,
        "?client_id=",
        client_id,
        "&redirect_uri=",
        redirect_url,
        "&response_type=code&state=state&code_challenge=challenge&code_challenge_method=plain&scope=",
        TwitterScopes::ReadTweet.as_str(),
        TwitterScopes::WriteTweet.as_str(),
        TwitterScopes::ModerateTweet.as_str(),
        TwitterScopes::ReadUsers.as_str(),
        TwitterScopes::ReadFollows.as_str(),
        TwitterScopes::WriteFollows.as_str(),
        TwitterScopes::AccessOffline.as_str(),
        TwitterScopes::ReadSpace.as_str(),
        TwitterScopes::ReadMute.as_str(),
        TwitterScopes::WriteMute.as_str(),
        TwitterScopes::ReadLike.as_str(),
        TwitterScopes::WriteLike.as_str(),
        TwitterScopes::ReadBlock.as_str(),
        TwitterScopes::WriteBlock.as_str(),
        TwitterScopes::ReadBookmark.as_str(),
        TwitterScopes::WriteBookmark.as_str(),
        TwitterScopes::WriteList.as_str(),
        TwitterScopes::ReadList.as_str(),
    );
    // send redirect to /auth/twitter/callback/
    Ok(actix_web::HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, twitter_auth_url))
        .finish())
}
// route: /auth/twitter/callback/
// In this API we are accessing
// the token and setting it to cookies
pub async fn callback(req: actix_web::HttpRequest) -> fastn_core::Result<actix_web::HttpResponse> {
    #[derive(Debug, serde::Deserialize)]
    pub struct QueryParams {
        pub code: String,
        pub state: String,
    }

    let query = actix_web::web::Query::<QueryParams>::from_query(req.query_string())?.0;
    let redirect_url = format!(
        "{}://{}{}",
        req.connection_info().scheme(),
        req.connection_info().host(),
        CALLBACK_URL
    );
    let twitter_auth =
        apis::twitter_token(TOKEN_URL, redirect_url.as_str(), query.code.as_str()).await;
    match twitter_auth {
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
                        fastn_core::auth::AuthProviders::Twitter.as_str(),
                        fastn_core::auth::utils::encrypt_str(&user_detail_str).await,
                    )
                    .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
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
pub async fn matched_identities(
    ud: UserDetail,
    identities: &[fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    let twitter_identities = identities
        .iter()
        .filter(|identity| identity.key.starts_with("twitter"))
        .collect::<Vec<&fastn_core::user_group::UserIdentity>>();

    if twitter_identities.is_empty() {
        return Ok(vec![]);
    }

    let mut matched_identities = vec![];
    // matched_liking_member
    matched_identities.extend(matched_liking_member(&ud, twitter_identities.as_slice()).await?);
    // matched_member_followers
    matched_identities.extend(matched_member_followers(&ud, twitter_identities.as_slice()).await?);
    // matched_member_followings
    matched_identities.extend(matched_member_followings(&ud, twitter_identities.as_slice()).await?);
    // matched_member_followings
    matched_identities.extend(matched_retweet_member(&ud, twitter_identities.as_slice()).await?);
    // matched_space_buyers
    matched_identities.extend(matched_space_buyers(&ud, twitter_identities.as_slice()).await?);
    Ok(matched_identities)
}
pub async fn matched_liking_member(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut member_liked_tweets: Vec<String> = vec![];
    let tweet_ids = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("twitter-liking") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if tweet_ids.is_empty() {
        return Ok(vec![]);
    }
    for tweet_id in tweet_ids.iter() {
        let tweet_liking_member: Vec<String> = apis::liking_members(&ud.token, tweet_id).await?;
        if tweet_liking_member.contains(&ud.user_id) {
            member_liked_tweets.push(tweet_id.to_string());
        }
        // TODO:
        // Return Error if tweet liking does not exist
    }
    // filter the user liked tweets with input
    Ok(member_liked_tweets
        .into_iter()
        .map(|tweet_id| fastn_core::user_group::UserIdentity {
            key: "twitter-liking".to_string(),
            value: tweet_id,
        })
        .collect())
}
pub async fn matched_retweet_member(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut member_retweeted_tweet: Vec<String> = vec![];
    let tweet_ids = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("twitter-retweet") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if tweet_ids.is_empty() {
        return Ok(vec![]);
    }
    for tweet_id in tweet_ids.iter() {
        let tweet_retweeted_member: Vec<String> =
            apis::tweet_retweeted_members(&ud.token, tweet_id).await?;
        if tweet_retweeted_member.contains(&ud.user_id) {
            member_retweeted_tweet.push(tweet_id.to_string());
        }
        // TODO:
        // Return Error if tweet liking does not exist
    }
    // filter the user liked tweets with input
    Ok(member_retweeted_tweet
        .into_iter()
        .map(|tweet_id| fastn_core::user_group::UserIdentity {
            key: "twitter-retweet".to_string(),
            value: tweet_id,
        })
        .collect())
}
//This method will be used to find given user twitter followers.
pub async fn matched_member_followers(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut followed_member_list: Vec<String> = vec![];
    let member_names = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("twitter-followers") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if member_names.is_empty() {
        return Ok(vec![]);
    }
    for member_name in member_names.iter() {
        let user_id = apis::user_details_by_name(&ud.token, member_name).await?;
        let member_followers: Vec<String> =
            apis::member_followers(&ud.token, user_id.as_str()).await?;
        if member_followers.contains(&ud.user_id) {
            followed_member_list.push(member_name.to_string());
        }
        // TODO:
        // Return Error if member follower does not exist
    }
    // filter the user followers with input
    Ok(followed_member_list
        .into_iter()
        .map(|member_id| fastn_core::user_group::UserIdentity {
            key: "twitter-followers".to_string(),
            value: member_id,
        })
        .collect())
}
pub async fn matched_member_followings(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut following_member_list: Vec<String> = vec![];
    let member_names = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("twitter-follows") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if member_names.is_empty() {
        return Ok(vec![]);
    }
    for member_name in member_names.iter() {
        let user_id = apis::user_details_by_name(&ud.token, member_name).await?;
        let member_followings: Vec<String> =
            apis::member_followings(&ud.token, user_id.as_str()).await?;
        if member_followings.contains(&ud.user_id) {
            following_member_list.push(member_name.to_string());
        }
        // TODO:
        // Return Error if member following does not exist
    }
    // filter the user following with input
    Ok(following_member_list
        .into_iter()
        .map(|member_id| fastn_core::user_group::UserIdentity {
            key: "twitter-follows".to_string(),
            value: member_id,
        })
        .collect())
}
pub async fn matched_space_buyers(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut space_allowed_list: Vec<String> = vec![];
    let space_ids = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("twitter-space") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if space_ids.is_empty() {
        return Ok(vec![]);
    }
    for space_id in space_ids.iter() {
        let space_ticket_buyers: Vec<String> =
            apis::space_ticket_buyers(&ud.token, space_id).await?;
        if space_ticket_buyers.contains(&ud.user_id) {
            space_allowed_list.push(space_id.to_string());
        }
        // TODO:
        // Return Error if spaces does not exist
    }
    // filter the user spaces with input
    Ok(space_allowed_list
        .into_iter()
        .map(|space_id| fastn_core::user_group::UserIdentity {
            key: "twitter-space".to_string(),
            value: space_id,
        })
        .collect())
}
pub mod apis {
    #[derive(serde::Deserialize)]
    pub struct TwitterAuthResp {
        pub access_token: String,
    }
    #[derive(serde::Deserialize)]
    pub struct DataObj {
        pub data: Vec<UserDetail>,
    }
    #[derive(serde::Deserialize)]
    pub struct UserDetail {
        pub id: String,
    }
    // API Docs: https://developer.twitter.com/en/docs/authentication/guides/v2-authentication-mapping

    //This API will only be used to get access token for discord
    pub async fn twitter_token(url: &str, redirect_url: &str, code: &str) -> fastn_core::Result<String> {
        let client_id = match std::env::var("TWITTER_CLIENT_ID") {
            Ok(id) => id,
            Err(_e) => {
                return Err(fastn_core::Error::APIResponseError(
                    "WARN: TWITTER_CLIENT_ID not set.".to_string(),
                ));
            }
        };

        let mut map: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();

        map.insert("client_id", client_id.as_str());

        map.insert("code", code);
        map.insert("redirect_uri", redirect_url);
        map.insert("grant_type", "authorization_code");
        map.insert("code_verifier", "challenge");

        let response = reqwest::Client::new().post(url).form(&map).send().await?;

        if !response.status().eq(&reqwest::StatusCode::OK) {
            return Err(fastn_core::Error::APIResponseError(format!(
                "TWITTER-API-ERROR: {}, Error: {}",
                url,
                response.text().await?
            )));
        }
        let auth_obj = response.json::<TwitterAuthResp>().await?;
        Ok(auth_obj.access_token)
    }

    // TODO: API to get user detail.
    // TODO: It can be stored in the request cookies
    pub async fn user_details(token: &str) -> fastn_core::Result<(String, String)> {
        // API Docs: https://api.twitter.com/2/users/me
        #[derive(serde::Deserialize)]
        struct DataObj {
            data: UserDetail,
        }
        #[derive(serde::Deserialize)]
        struct UserDetail {
            username: String,
            id: String,
        }
        let user_obj: DataObj = fastn_core::auth::utils::get_api(
            "https://api.twitter.com/2/users/me",
            format!("{} {}", "Bearer", token).as_str(),
        )
        .await?;

        Ok((user_obj.data.username, user_obj.data.id))
    }
    // TODO: API to get user detail by name.
    // TODO: It can be stored in the request cookies
    pub async fn user_details_by_name(token: &str, username: &str) -> fastn_core::Result<String> {
        // API Docs: https://api.twitter.com/2/users/by/username/{username}
        #[derive(serde::Deserialize)]
        struct DataObj {
            data: UserDetail,
        }
        #[derive(serde::Deserialize)]
        struct UserDetail {
            id: String,
        }
        let user_obj: DataObj = fastn_core::auth::utils::get_api(
            format!(
                "{}{}",
                "https://api.twitter.com/2/users/by/username/", username
            )
            .as_str(),
            format!("{} {}", "Bearer", token).as_str(),
        )
        .await?;
        Ok(user_obj.data.id)
    }
    pub async fn liking_members(token: &str, tweet_id: &str) -> fastn_core::Result<Vec<String>> {
        // API Docs: https://api.twitter.com/2/tweets/{tweet-id}/liking_users?max_results=100
        // TODO: Handle paginated response

        let liking_member_list: DataObj = fastn_core::auth::utils::get_api(
            format!(
                "{}{}{}?max_results=100",
                "https://api.twitter.com/2/tweets/", tweet_id, "/liking_users"
            )
            .as_str(),
            format!("{} {}", "Bearer", token).as_str(),
        )
        .await?;
        Ok(liking_member_list.data.into_iter().map(|x| x.id).collect())
    }
    pub async fn tweet_retweeted_members(
        token: &str,
        tweet_id: &str,
    ) -> fastn_core::Result<Vec<String>> {
        // API Docs: https://api.twitter.com/2/tweets/{tweet-id}/retweeted_by?max_results=100
        // TODO: Handle paginated response

        let retweeted_member_list: DataObj = fastn_core::auth::utils::get_api(
            format!(
                "{}{}{}?max_results=100",
                "https://api.twitter.com/2/tweets/", tweet_id, "/retweeted_by"
            )
            .as_str(),
            format!("{} {}", "Bearer", token).as_str(),
        )
        .await?;
        Ok(retweeted_member_list
            .data
            .into_iter()
            .map(|x| x.id)
            .collect())
    }
    pub async fn member_followers(token: &str, member_id: &str) -> fastn_core::Result<Vec<String>> {
        // API Docs: https://api.twitter.com/2/users/{user-id}/followers?max_results=100
        // TODO: Handle paginated response

        let member_follower_list: DataObj = fastn_core::auth::utils::get_api(
            format!(
                "{}{}{}?max_results=100",
                "https://api.twitter.com/2/users/", member_id, "/followers"
            )
            .as_str(),
            format!("{} {}", "Bearer", token).as_str(),
        )
        .await?;
        Ok(member_follower_list
            .data
            .into_iter()
            .map(|x| x.id)
            .collect())
    }
    pub async fn member_followings(token: &str, member_id: &str) -> fastn_core::Result<Vec<String>> {
        // API Docs: https://api.twitter.com/2/users/{user-id}/following?max_results=100
        // TODO: Handle paginated response

        let member_following_list: DataObj = fastn_core::auth::utils::get_api(
            format!(
                "{}{}{}?max_results=100",
                "https://api.twitter.com/2/users/", member_id, "/following"
            )
            .as_str(),
            format!("{} {}", "Bearer", token).as_str(),
        )
        .await?;
        Ok(member_following_list
            .data
            .into_iter()
            .map(|x| x.id)
            .collect())
    }
    pub async fn space_ticket_buyers(token: &str, space_id: &str) -> fastn_core::Result<Vec<String>> {
        // API Docs: https://api.twitter.com/2/spaces/{space_id}/buyers?max_results=100
        // TODO: Handle paginated response

        let space_ticket_buyers_list: DataObj = fastn_core::auth::utils::get_api(
            format!(
                "{}{}{}?max_results=100",
                "https://api.twitter.com/2/spaces/", space_id, "/buyers"
            )
            .as_str(),
            format!("{} {}", "Bearer", token).as_str(),
        )
        .await?;
        Ok(space_ticket_buyers_list
            .data
            .into_iter()
            .map(|x| x.id)
            .collect())
    }
}
