struct ArtifactBaseRequest {
    token: String,
    url: String,
    user_agent: String,
    options: Option<ArtifactOptions>,
    response: Option<reqwest::Response>,
}

struct ArtifactOptions {
    duration: Option<u64>,
    tag: Option<String>,
}

impl ArtifactBaseRequest {
    fn new(
        token: String,
        url: String,
        user_agent: String,
        options: Option<ArtifactOptions>,
    ) -> Self {
        ArtifactBaseRequest {
            token,
            url,
            user_agent,
            options,
            response: None,
        }
    }

    fn get_headers(&self, method: &str) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();

        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", self.token)).unwrap(),
        );

        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_str(&self.user_agent).unwrap(),
        );

        if method == "PUT" {
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                reqwest::header::HeaderValue::from_static("application/octet-stream"),
            );

            if let Some(options) = &self.options {
                if let Some(duration) = options.duration {
                    headers.insert(
                        reqwest::header::HeaderName::from_static("x-artifact-duration"),
                        reqwest::header::HeaderValue::from(duration),
                    );
                }

                if let Some(tag) = &options.tag {
                    headers.insert(
                        reqwest::header::HeaderName::from_static("x-artifact-tag"),
                        reqwest::header::HeaderValue::from_str(tag).unwrap(),
                    );
                }
            }
        }

        if let Ok(ci_name) = std::env::var("CI") {
            headers.insert(
                reqwest::header::HeaderName::from_static("x-artifact-client-ci"),
                reqwest::header::HeaderValue::from_str(&ci_name).unwrap(),
            );
        }

        let is_tty = atty::is(atty::Stream::Stdout);
        headers.insert(
            reqwest::header::HeaderName::from_static("x-artifact-client-interactive"),
            reqwest::header::HeaderValue::from_str(if is_tty { "1" } else { "0" }).unwrap(),
        );

        headers
    }

    async fn fetch(&mut self, method: &str) -> fastn_core::Result<()> {
        let client = reqwest::Client::new();

        let response = match method {
            "GET" => client.get(&self.url),
            "HEAD" => client.head(&self.url),
            "PUT" => client.put(&self.url),
            _ => todo!(),
        }
        .headers(self.get_headers(method))
        .send()
        .await?;

        self.response = Some(response);
        Ok(())
    }
}
