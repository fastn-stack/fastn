pub(crate) mod cloud;
pub(crate) const PUBLISH_STATIC: &str = "publish-static";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("PublishStaticError: {}", _0)]
    PublishStaticError(#[from] PublishStaticError),
}

#[derive(thiserror::Error, Debug)]
pub enum PublishStaticError {
    #[error("PublishStaticCreateError: {}", _0)]
    PublishStaticCreateError(#[from] fastn_cloud::CreateError),
    #[error("PublishStaticUpdateError: {}", _0)]
    PublishStaticUpdateError(#[from] fastn_cloud::UpdateError),
}

pub(crate) async fn handle(matches: &clap::ArgMatches) -> Result<bool, Error> {
    // TODO: Handle Dynamic Commands
    Ok(handle_publish_static(matches).await?)
}

pub(crate) async fn handle_publish_static(
    matches: &clap::ArgMatches,
) -> Result<bool, PublishStaticError> {
    match matches.subcommand() {
        Some(("create", _matches)) => {
            fastn_cloud::create().await?;
            Ok(true)
        }
        Some(("update", _matches)) => {
            fastn_cloud::update().await?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
