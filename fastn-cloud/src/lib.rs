extern crate self as fastn_cloud;

use std::io;

mod create;
mod http;
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

pub async fn create() -> Result<(), fastn_cloud::CreateError> {
    let root = fastn_cloud::utils::build_dir();
    if !root.exists() {
        return Err(CreateError::BuildDirNotFound(
            "Run `fastn build` to create a .build directory before running this".to_string(),
        ));
    }
    let resp = fastn_cloud::create::create_package(root.as_path())
        .await
        .unwrap();
    println!("{}", root);
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
