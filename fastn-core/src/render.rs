/// `fastn_core::render()` renders a single ftd file that is part of current fastn package.
/// It returns `fastn_core::Result` of rendered HTML as `String`.
#[allow(dead_code)]
pub async fn render(
    config: &fastn_core::Config,
    id: &str,
    base_url: &str,
) -> fastn_core::Result<String> {
    let file = config.get_file_by_id(id, &config.package).await?;
    let asset_documents = config.get_assets().await?;

    let main = match file {
        fastn_core::File::Ftd(f) => f,
        _ => unreachable!(),
    };

    let new_main = fastn_core::Document {
        content: config
            .package
            .get_prefixed_body(main.content.as_str(), &main.id, true),
        id: main.id.to_owned(),
        parent_path: main.parent_path.to_owned(),
        package_name: main.package_name.to_owned(),
    };

    return get_html(config, &new_main, base_url, &asset_documents).await;

    async fn get_html(
        config: &fastn_core::Config,
        main: &fastn_core::Document,
        base_url: &str,
        asset_documents: &std::collections::HashMap<String, String>,
    ) -> fastn_core::Result<String> {
        let lib = fastn_core::Library {
            config: config.clone(),
            markdown: None,
            document_id: main.id.clone(),
            translated_data: Default::default(),
            asset_documents: asset_documents.to_owned(),
            base_url: base_url.to_string(),
        };

        let main_ftd_doc = match fastn_core::doc::parse(
            main.id_with_package().as_str(),
            main.content.as_str(),
            &lib,
            base_url,
            None,
        )
        .await
        {
            Ok(v) => v,
            Err(e) => {
                return Err(fastn_core::Error::PackageError {
                    message: format!("failed to parse {:?}", &e),
                });
            }
        };
        let doc_title = match &main_ftd_doc.title() {
            Some(x) => x.original.clone(),
            _ => main.id.as_str().to_string(),
        };
        let ftd_doc = main_ftd_doc.to_rt("main", &main.id);
        Ok(fastn_core::utils::replace_markers_2021(
            fastn_core::ftd_html(),
            config,
            main.id_to_path().as_str(),
            doc_title.as_str(),
            base_url,
            &ftd_doc,
        ))
    }
}
