pub(crate) const TEST_FOLDER: &str = "_tests";
pub(crate) const TEST_FILE_EXTENSION: &str = ".test.ftd";

pub async fn test(
    config: &fastn_core::Config,
    only_id: Option<&str>,
    base_url: &str,
    headless: bool,
) -> fastn_core::Result<()> {
    let test_folder = config.root.join(TEST_FOLDER);
    let mut documents = std::collections::BTreeMap::from_iter(
        config
            .get_test_files(&config.package)
            .await?
            .into_iter()
            .map(|v| (v.get_id().to_string(), v)),
    );
    Ok(())
}
