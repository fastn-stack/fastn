pub mod commit_upload;
pub mod initiate_upload;

pub use commit_upload::{CommitUploadError, CommitUploadRequest, commit_upload};
pub use initiate_upload::{
    ContentToUpload, InitiateUploadError, InitiateUploadRequest, InitiateUploadResponse,
    PreSignedRequest, initiate_upload,
};

pub const ENDPOINT: &str = "https://www.fifthtry.com";

#[derive(serde::Deserialize)]
pub struct ApiResponse<T> {
    pub data: Option<T>,
    pub errors: Option<std::collections::HashMap<String, String>>,
    pub success: bool,
}

pub fn endpoint(name: &str) -> String {
    format!(
        "{prefix}/ft2/api/{name}/",
        prefix = std::env::var("DEBUG_API_FIFTHTRY_COM")
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_else(|_| clift::api::ENDPOINT)
    )
}
