#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct Version {
    pub major: u64,
    pub minor: Option<u64>,
    pub original: String,
}

impl PartialOrd for Version {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ord::cmp(self, rhs))
    }
}

impl Ord for Version {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        if self.major.eq(&rhs.major) {
            let lhs_minor = self.minor.unwrap_or(0);
            let rhs_minor = rhs.minor.unwrap_or(0);
            return lhs_minor.cmp(&rhs_minor);
        }
        self.major.cmp(&rhs.major)
    }
}

/*#[allow(dead_code)]
pub(crate) async fn build_version(
    config: &fastn_core::Config,
    _file: Option<&str>,
    base_url: &str,
    _skip_failed: bool,
    _asset_documents: &std::collections::HashMap<String, String>,
) -> fastn_core::Result<()> {
    use itertools::Itertools;

    let versioned_documents = config.get_versions(&config.package).await?;
    let mut documents = std::collections::BTreeMap::new();
    for key in versioned_documents.keys().sorted() {
        let doc = versioned_documents[key].to_owned();
        documents.extend(doc.iter().map(|v| {
            (
                v.get_id().to_string(),
                (key.original.to_string(), v.to_owned()),
            )
        }));
        if key.eq(&fastn_core::Version::base()) {
            continue;
        }
        for (version, doc) in documents.values() {
            let mut doc = doc.clone();
            let id = doc.get_id();
            if id.eq("FASTN.ftd") {
                continue;
            }
            let new_id = format!("{}/{}", key.original, id);
            if !key.original.eq(version) && !fastn_core::Version::base().original.eq(version) {
                if let fastn_core::File::Ftd(_) = doc {
                    let original_id = format!("{}/{}", version, id);
                    let original_file_rel_path = if original_id.contains("index.ftd") {
                        original_id.replace("index.ftd", "index.html")
                    } else {
                        original_id.replace(
                            ".ftd",
                            format!("{}index.html", std::path::MAIN_SEPARATOR).as_str(),
                        )
                    };
                    let original_file_path =
                        config.ds.root().join(".build").join(original_file_rel_path);
                    let file_rel_path = if new_id.contains("index.ftd") {
                        new_id.replace("index.ftd", "index.html")
                    } else {
                        new_id.replace(
                            ".ftd",
                            format!("{}index.html", std::path::MAIN_SEPARATOR).as_str(),
                        )
                    };
                    let new_file_path = config.ds.root().join(".build").join(file_rel_path);
                    let original_content = config.ds.read_to_string(&original_file_path).await?;
                    let from_pattern = format!("<base href=\"{}{}/\">", base_url, version);
                    let to_pattern = format!("<base href=\"{}{}/\">", base_url, key.original);
                    config
                        .ds
                        .write_content(
                            &new_file_path,
                            original_content
                                .replace(from_pattern.as_str(), to_pattern.as_str())
                                .into_bytes(),
                        )
                        .await?;
                    continue;
                }
            }
            doc.set_id(new_id.as_str());

            todo!()

            // fastn_core::process_file(
            //     config,
            //     &config.package,
            //     &doc,
            //     None,
            //     None,
            //     Default::default(),
            //     format!("{}{}/", base_url, key.original).as_str(),
            //     skip_failed,
            //     asset_documents,
            //     Some(id),
            //     false,
            // )
            // .await?;
        }
    }

    todo!()
    // for (_, doc) in documents.values() {
    //     fastn_core::process_file(
    //         config,
    //         &config.package,
    //         doc,
    //         None,
    //         None,
    //         Default::default(),
    //         base_url,
    //         skip_failed,
    //         asset_documents,
    //         None,
    //         false,
    //     )
    //     .await?;
    // }
    // Ok(())
}*/
