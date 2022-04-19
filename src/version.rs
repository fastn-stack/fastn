pub(crate) async fn build_version(
    config: &fpm::Config,
    _file: Option<&str>,
    base_url: &str,
    skip_failed: bool,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<()> {
    let versioned_documents = config.get_versions(&config.package).await?;
    let mut index = 1;
    let mut documents = std::collections::BTreeMap::new();
    loop {
        if let Some(doc) = versioned_documents.get(&index) {
            documents.extend(doc.into_iter().map(|v| (v.get_id(), v.to_owned())));
        } else {
            break;
        }
        for doc in documents.values() {
            let mut doc = doc.clone();
            let id = doc.get_id();
            doc.set_id(format!("v{}/{}", index, id).as_str());
            fpm::process_file(
                config,
                &config.package,
                &doc,
                None,
                None,
                Default::default(),
                base_url,
                skip_failed,
                asset_documents,
            )
            .await?;
        }
        index += 1;
    }
    for doc in documents.values() {
        fpm::process_file(
            config,
            &config.package,
            doc,
            None,
            None,
            Default::default(),
            base_url,
            skip_failed,
            asset_documents,
        )
        .await?;
    }
    Ok(())
}
