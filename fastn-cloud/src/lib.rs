extern crate self as fastn_cloud;

mod http;
mod upload;
mod utils;

#[derive(thiserror::Error, Debug)]
pub enum UploadError {
    #[error("BuildDirNotFound: {}", _0)]
    BuildDirNotFound(String),
    #[error("cw-id not found")]
    CwIdNotFound,
    #[error("cw-id read error")]
    CwIdReadError,
    #[error("sid not found")]
    SidNotFound,
    #[error("sid read error")]
    SidReadError,
    #[error("TejarCreateError: {}", _0)]
    TejarCreateError(#[from] tejar::error::CreateError),
    #[error("HTTPPostError: {}", _0)]
    HTTPPostError(#[from] fastn_cloud::http::Error),
    #[error("StdIOError: {}", _0)]
    StdIOError(#[from] std::io::Error),
    #[error("SidParseError: {}", _0)]
    SidParseError(#[from] serde_json::Error),
    #[error("TejarReadError: {}", _0)]
    TejarRead(#[from] tejar::error::ReadError),
    #[error("FromPathBufError: {}", _0)]
    FromPathBufError(#[from] camino::FromPathBufError),
}

pub async fn upload() -> Result<(), UploadError> {
    let current_dir: camino::Utf8PathBuf = std::env::current_dir()?.canonicalize()?.try_into()?;
    let ds = fastn_ds::DocumentStore::new(current_dir);
    let build_dir = fastn_cloud::utils::build_dir(&ds);
    if !build_dir.exists() {
        return Err(UploadError::BuildDirNotFound(
            "Run `fastn build` to create a .build directory before running this".to_string(),
        ));
    }

    let cw_id_path = fastn_cloud::utils::cw_id(&ds);
    if !cw_id_path.exists() {
        return Err(UploadError::CwIdNotFound);
    }
    let _cw_id = ds
        .read_to_string(&cw_id_path)
        .await
        .map_err(|_e| UploadError::CwIdReadError)?;

    let sid_path = fastn_cloud::utils::sid(&ds);
    if !sid_path.exists() {
        return Err(UploadError::SidNotFound);
    }

    let _sid = ds
        .read_to_string(&sid_path)
        .await
        .map_err(|_e| UploadError::SidReadError)?;

    // Todo: Revive this
    // fastn_cloud::upload::upload(&ds, build_dir, sid.trim(), cw_id.trim()).await?;
    println!("publish-static upload done");
    Ok(())
}
