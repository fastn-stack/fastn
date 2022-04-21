use itertools::Itertools;

pub(crate) async fn build_version(
    config: &fpm::Config,
    _file: Option<&str>,
    base_url: &str,
    skip_failed: bool,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<()> {
    let versioned_documents = config.get_versions(&config.package).await?;
    let mut documents = std::collections::BTreeMap::new();
    for key in versioned_documents.keys().sorted() {
        let (version_str, doc) = versioned_documents[key].to_owned();
        documents.extend(doc.iter().map(|v| (v.get_id(), v.to_owned())));
        if key.eq(&semver::Version::new(0, 0, 0)) {
            continue;
        }
        for doc in documents.values() {
            let mut doc = doc.clone();
            let id = doc.get_id();
            if id.eq("FPM.ftd") {
                continue;
            }
            doc.set_id(format!("{}/{}", version_str, id).as_str());
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
                Some(id),
            )
            .await?;
        }
    }

    /*while let Some(doc) = versioned_documents.get(&index) {
        documents.extend(doc.iter().map(|v| (v.get_id(), v.to_owned())));
        if index.eq(&0) {
            index += 1;
            continue;
        }
        for doc in documents.values() {
            let mut doc = doc.clone();
            let id = doc.get_id();
            if id.eq("FPM.ftd") {
                continue;
            }
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
                Some(id),
            )
            .await?;
        }
        index += 1;
    }*/

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
            None,
        )
        .await?;
    }
    Ok(())
}
