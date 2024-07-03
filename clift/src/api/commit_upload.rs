fn endpoint() -> String {
    clift::api::endpoint("commit-upload")
}

#[derive(serde::Serialize)]
pub struct CommitUploadRequest {
    site: String,
    upload_session_id: i64,
    tejar_file_id: Option<i64>,
}

#[derive(Debug, thiserror::Error)]
pub enum CommitUploadError {
    #[error("cant call api: {0}")]
    CantCallAPI(#[from] reqwest::Error),
}

pub async fn commit_upload(
    site_slug: &str,
    data: &clift::api::InitiateUploadResponse,
    update_token: &clift::utils::UpdateToken,
) -> Result<(), CommitUploadError> {
    let response = clift::utils::call_api(
        reqwest::Client::new()
            .post(clift::api::commit_upload::endpoint())
            .json(&CommitUploadRequest {
                site: site_slug.to_string(),
                upload_session_id: data.upload_session_id,
                tejar_file_id: data.tejar_file_id,
            }),
        update_token,
    )
    .await?;

    if !response.status().is_success() {
        todo!("response.text(): {:?}", response.text().await)
    }

    Ok(())
}
