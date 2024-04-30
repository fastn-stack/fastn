pub mod commit_upload;
pub mod initiate_upload;

pub use commit_upload::{commit_upload, CommitUploadError, CommitUploadRequest};
pub use initiate_upload::{
    initiate_upload, ContentToUpload, InitiateUploadError, InitiateUploadRequest,
    InitiateUploadResponse, PreSignedRequest,
};

pub const ENDPOINT: &str = "https://api.fifthtry.com";

#[derive(serde::Deserialize)]
pub struct ApiResponse<T> {
    pub data: Option<T>,
    pub errors: Option<std::collections::HashMap<String, String>>,
    pub success: bool,
}

pub fn endpoint(name: &str) -> String {
    if let Ok(url) = std::env::var("DEBUG_API_FIFTHTRY_COM") {
        println!("using debug api, only use this for testing");
        return format!("{}/ft2/api/{name}/", url);
    }
    format!("{}/ft2/api/{name}/", clift::api::ENDPOINT)
}
