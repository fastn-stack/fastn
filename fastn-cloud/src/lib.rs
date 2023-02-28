extern crate self as fastn_cloud;

use std::io;

mod create;
mod http;
mod upload;
mod utils;

#[derive(thiserror::Error, Debug)]
pub enum CreateError {
    #[error("BuildDirNotFound: {}", _0)]
    BuildDirNotFound(String),
    #[error("TejarCreateError: {}", _0)]
    TejarCreateError(#[from] tejar::error::CreateError),
    #[error("StdIOError: {}", _0)]
    StdIOError(#[from] io::Error),
    #[error("FastnCoreError: {}", _0)]
    FastnCoreError(#[from] fastn_core::Error),
    #[error("HttpPOSTCreateError: {}", _0)]
    HttpPOSTCreateError(#[from] fastn_cloud::http::PostError),
}

#[derive(thiserror::Error, Debug)]
pub enum UpdateError {}

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
    HTTPPostError(#[from] fastn_cloud::http::PostError),
    #[error("StdIOError: {}", _0)]
    StdIOError(#[from] io::Error),
}

pub async fn create() -> Result<(), fastn_cloud::CreateError> {
    let build_dir = fastn_cloud::utils::build_dir();
    if !build_dir.exists() {
        return Err(CreateError::BuildDirNotFound(
            "Run `fastn build` to create a .build directory before running this".to_string(),
        ));
    }
    fastn_cloud::create::create_package(build_dir.as_path())
        .await
        .unwrap();
    println!("{}", build_dir);
    // call fastn build
    // read the content of the .build folder
    // pass this content to tejar and create two files LIST and DATA
    // call /api/create/ by passing the content of the LIST and META
    // call /api/upload-new-package by passing the missing entries and DATA
    // save package key and at home folder
    // show the subdomain to user or open browser directly
    println!("publish-static create called");
    Ok(())
}

pub async fn update() -> Result<(), UpdateError> {
    println!("publish-static update called");
    Ok(())
}

pub async fn upload() -> Result<(), UploadError> {
    let build_dir = fastn_cloud::utils::build_dir();
    if !build_dir.exists() {
        return Err(UploadError::BuildDirNotFound(
            "Run `fastn build` to create a .build directory before running this".to_string(),
        ));
    }

    let cw_id_path = fastn_cloud::utils::cw_id();
    if !cw_id_path.exists() {
        return Err(UploadError::CwIdNotFound);
    }
    let cw_id = tokio::fs::read_to_string(cw_id_path.as_path())
        .await
        .map_err(|_e| UploadError::CwIdReadError)?;

    let sid_path = fastn_cloud::utils::sid();
    if !sid_path.exists() {
        return Err(UploadError::SidNotFound);
    }

    let sid = tokio::fs::read_to_string(sid_path.as_path())
        .await
        .map_err(|_e| UploadError::SidReadError)?;

    fastn_cloud::upload::upload(build_dir.as_path(), sid.trim(), cw_id.trim()).await?;
    println!("publish-static upload called");
    Ok(())
}
