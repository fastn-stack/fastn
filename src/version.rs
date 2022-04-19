pub(crate) async fn build_version(
    config: &fpm::Config,
    file: Option<&str>,
    base_url: &str,
    skip_failed: bool,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<()> {
    // let documents = std::collections::BTreeMap::from_iter(
    //     config
    //         .get_files(&config.package)
    //         .await?
    //         .into_iter()
    //         .map(|v| (v.get_id(), v)),
    // );
    let versioned_documents = config.get_versions(&config.package).await?;
    dbg!(&versioned_documents);
    Ok(())
}
