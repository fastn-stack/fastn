#[derive(thiserror::Error, Debug)]
pub enum InitialisePackageError {}

#[async_trait::async_trait]
pub trait PackageInitializer {}

pub async fn initialize_packages(_: impl PackageInitializer) -> Result<(), InitialisePackageError> {
    Ok(())
}
