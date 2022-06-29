#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
pub struct EditRequest {
    pub url: String,
    pub value: String,
    pub path: String,
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
pub struct EditResponse {
    pub path: String,
}

pub async fn edit(
    req: actix_web::web::Json<EditRequest>,
) -> actix_web::Result<actix_web::HttpResponse> {
    match edit_worker(req.0).await {
        Ok(data) => fpm::apis::success(data),
        Err(err) => fpm::apis::error(
            err.to_string(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

pub(crate) async fn edit_worker(request: EditRequest) -> fpm::Result<EditResponse> {
    let mut config = fpm::Config::read2(None, false).await?;
    let file_name = config
        .get_file_path_and_resolve(request.path.as_str())
        .await?;
    fpm::utils::update(
        &config.root,
        file_name.as_str(),
        request.value.into_bytes().as_slice(),
    )
    .await?;
    Ok(EditResponse { path: request.path })
}
