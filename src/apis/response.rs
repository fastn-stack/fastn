pub(crate) fn success(data: impl serde::Serialize) -> actix_web::Result<actix_web::HttpResponse> {
    #[derive(serde::Serialize)]
    struct SuccessResponse<T: serde::Serialize> {
        data: T,
        success: bool,
    }

    let data = serde_json::to_string(&SuccessResponse {
        data,
        success: true,
    })?;

    Ok(actix_web::HttpResponse::Ok()
        .content_type(actix_web::http::header::ContentType::json())
        .status(actix_web::http::StatusCode::OK)
        .body(data))
}

pub(crate) fn error<T: Into<String>>(
    message: T,
    status: actix_web::http::StatusCode,
) -> actix_web::Result<actix_web::HttpResponse> {
    #[derive(serde::Serialize, Debug)]
    struct ErrorResponse {
        message: String,
        success: bool,
    }

    let resp = ErrorResponse {
        message: message.into(),
        success: false,
    };

    Ok(actix_web::HttpResponse::Ok()
        .content_type(actix_web::http::header::ContentType::json())
        .status(status)
        .body(serde_json::to_string(&resp)?))
}
