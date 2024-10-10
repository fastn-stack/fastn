fn endpoint() -> String {
    clift::api::endpoint("initiate-upload")
}

#[derive(serde::Serialize)]
pub enum InitiateUploadRequest {
    Folder {
        site: String,
        files: Vec<ContentToUpload>,
        folder: String,
        dry_run: bool,
    },
    File {
        site: String,
        file: ContentToUpload,
        dry_run: bool,
    },
}

impl InitiateUploadRequest {
    pub fn get_site(&self) -> String {
        match self {
            InitiateUploadRequest::Folder { site, .. }
            | InitiateUploadRequest::File { site, .. } => site.clone(),
        }
    }
    pub fn is_dry_run(&self) -> bool {
        match self {
            InitiateUploadRequest::Folder { dry_run, .. }
            | InitiateUploadRequest::File { dry_run, .. } => *dry_run,
        }
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct InitiateUploadResponse {
    pub new_files: Vec<String>,
    pub updated_files: Vec<String>,
    #[serde(default)]
    pub deleted_files: Vec<String>,
    pub upload_session_id: i64,
    pub tejar_file_id: Option<i64>,
    pub pre_signed_request: Option<PreSignedRequest>,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct PreSignedRequest {
    pub url: String,
    pub method: String,
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ContentToUpload {
    pub file_name: String,   // name of the file
    pub sha256_hash: String, // hash of the file
    pub file_size: usize,    // size of the file
}

#[derive(Debug, thiserror::Error)]
pub enum InitiateUploadError {
    #[error("cant call api: {0}")]
    CantCallAPI(#[from] reqwest::Error),
    #[error("cant read body during error: {0}")]
    CantReadBodyDuringError(reqwest::Error),
    #[error("got error from api: {0}")]
    APIError(String),
    #[error("cant parse json: {0}")]
    CantParseJson(#[from] serde_json::Error),
    #[error("got failure from ft: {0:?}")]
    GotFailure(std::collections::HashMap<String, String>),
}

pub async fn initiate_upload(
    to_upload: clift::api::InitiateUploadRequest,
    update_token: &clift::utils::UpdateToken,
) -> Result<InitiateUploadResponse, InitiateUploadError> {
    let response = clift::utils::call_api(
        reqwest::Client::new()
            .post(clift::api::initiate_upload::endpoint())
            .json(&to_upload),
        update_token,
    )
    .await
    .map_err(InitiateUploadError::CantCallAPI)?;

    if !response.status().is_success() {
        return Err(InitiateUploadError::APIError(
            response
                .text()
                .await
                .map_err(InitiateUploadError::CantReadBodyDuringError)?,
        ));
    }

    let json: clift::api::ApiResponse<InitiateUploadResponse> = response.json().await?;

    if !json.success {
        // TODO: remove unwrap
        return Err(InitiateUploadError::GotFailure(json.errors.unwrap()));
    }

    Ok(json.data.unwrap()) // TODO: remove unwrap
}
