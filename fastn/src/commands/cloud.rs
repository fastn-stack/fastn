pub(crate) const PUBLISH_STATIC: &str = "publish-static";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("PublishStaticError: {}", _0)]
    PublishStaticError(#[from] PublishStaticError),
}

#[derive(thiserror::Error, Debug)]
pub enum PublishStaticError {
    #[error("PublishStaticUploadError: {}", _0)]
    Upload(#[from] fastn_cloud::UploadError),
}

pub(crate) async fn handle() -> Result<bool, Error> {
    Ok(handle_publish_static().await?)
}

pub(crate) async fn handle_publish_static() -> Result<bool, PublishStaticError> {
    fastn_cloud::upload().await?;
    Ok(true)
}
