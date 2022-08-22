pub async fn create_cr() -> actix_web::Result<actix_web::HttpResponse> {
    let config = match fpm::Config::read(None, false).await {
        Ok(config) => config,
        Err(err) => {
            return fpm::apis::error(
                err.to_string(),
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    };
    match fpm::commands::create_cr::create_cr(&config).await {
        Ok(cr_number) => {
            #[derive(serde::Serialize)]
            struct CreateCRResponse {
                url: String,
            }
            let url = format!("-/{}/-/about/", cr_number);
            fpm::apis::success(CreateCRResponse { url })
        }
        Err(err) => fpm::apis::error(
            err.to_string(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}
