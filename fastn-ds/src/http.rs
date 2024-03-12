pub(crate) struct ResponseBuilder {}

impl ResponseBuilder {
    pub async fn from_reqwest(response: reqwest::Response) -> fastn_ds::HttpResponse {
        let status = response.status();

        // Remove `Connection` as per
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
        let mut response_builder = actix_web::HttpResponse::build(status);
        for header in response
            .headers()
            .iter()
            .filter(|(h, _)| *h != "connection")
        {
            response_builder.insert_header(header);
        }

        let content = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return actix_web::HttpResponse::from(actix_web::error::ErrorInternalServerError(
                    fastn_ds::HttpError::HttpError(e),
                ))
            }
        };
        response_builder.body(content)
    }
}
