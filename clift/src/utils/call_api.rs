pub async fn call_api(
    mut request_builder: reqwest::RequestBuilder,
    token: &clift::utils::UpdateToken,
) -> reqwest::Result<reqwest::Response> {
    match token {
        clift::utils::UpdateToken::SiteToken(clift::utils::SiteToken(token)) => {
            request_builder = request_builder.header("X-FIFTHTRY-SITE-WRITE-TOKEN", token);
        }
        clift::utils::UpdateToken::GithubToken(token) => {
            request_builder = request_builder
                .header(
                    "X-FIFTHTRY-GH-ACTIONS-ID-TOKEN-REQUEST-TOKEN",
                    token.token.clone(),
                )
                .header(
                    "X-FIFTHTRY-GH-ACTIONS-ID-TOKEN-REQUEST-URL",
                    token.url.clone(),
                );
        }
    }
    request_builder.send().await
}
