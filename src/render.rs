/// `fpm::render()` renders a single ftd file that is part of current FPM package.
/// It returns `fpm::Result` of rendered HTML as `String`.
#[allow(dead_code)]
pub async fn render(config: &fpm::Config, file: &str, base_url: &str) -> fpm::Result<String> {
    use itertools::Itertools;

    let file = fpm::get_documents(config, &config.package)
        .await?
        .into_iter()
        .find(|v| v.get_id().eq(file))
        .ok_or_else(|| fpm::Error::UsageError {
            message: format!("No such file found: {}", file),
        })?;
    let dependencies = if let Some(package) = config.package.translation_of.as_ref() {
        let mut deps = package
            .get_flattened_dependencies()
            .into_iter()
            .unique_by(|dep| dep.package.name.clone())
            .collect_vec();
        deps.extend(
            config
                .package
                .get_flattened_dependencies()
                .into_iter()
                .unique_by(|dep| dep.package.name.clone()),
        );
        deps
    } else {
        config
            .package
            .get_flattened_dependencies()
            .into_iter()
            .unique_by(|dep| dep.package.name.clone())
            .collect_vec()
    };
    let mut asset_documents = std::collections::HashMap::new();
    asset_documents.insert(
        config.package.name.clone(),
        config.package.get_assets_doc(config, base_url).await?,
    );
    for dep in &dependencies {
        asset_documents.insert(
            dep.package.name.clone(),
            dep.package.get_assets_doc(config, base_url).await?,
        );
    }
    match file {
        fpm::File::Ftd(main_doc) => {
            fpm::commands::build::get_ftd(config, &main_doc, base_url, &asset_documents).await
        }
        _ => unimplemented!(),
    }
}
