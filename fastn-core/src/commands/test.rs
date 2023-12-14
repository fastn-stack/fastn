pub(crate) const TEST_FOLDER: &str = "_tests";
pub(crate) const TEST_FILE_EXTENSION: &str = ".test.ftd";

pub async fn test(
    config: &fastn_core::Config,
    only_id: Option<&str>,
    base_url: &str,
    headless: bool,
) -> fastn_core::Result<()> {
    if !headless {
        return fastn_core::usage_error(
            "Currently headless mode is only suuported, use: --headless flag".to_string(),
        );
    }
    let ftd_documents = config.get_test_files().await?;

    for document in ftd_documents {
        read_ftd_test_file(document, config).await?;
    }

    Ok(())
}

impl fastn_core::Config {
    /**
    Returns the list of all test files with extension of `<file name>.test.ftd`
    **/
    pub(crate) async fn get_test_files(&self) -> fastn_core::Result<Vec<fastn_core::Document>> {
        use itertools::Itertools;
        let package = &self.package;
        let path = self.get_root_for_package(package);
        let all_files = self.get_all_test_file_paths()?;
        let documents = fastn_core::paths_to_files(package.name.as_str(), all_files, &path).await?;
        let mut tests = documents
            .into_iter()
            .filter_map(|file| match file {
                fastn_core::File::Ftd(ftd_document)
                    if ftd_document
                        .id
                        .ends_with(fastn_core::commands::test::TEST_FILE_EXTENSION) =>
                {
                    Some(ftd_document)
                }
                _ => None,
            })
            .collect_vec();
        tests.sort_by_key(|v| v.id.to_string());

        Ok(tests)
    }

    pub(crate) fn get_all_test_file_paths(&self) -> fastn_core::Result<Vec<camino::Utf8PathBuf>> {
        let path = self
            .get_root_for_package(&self.package)
            .join(fastn_core::commands::test::TEST_FOLDER);
        let mut ignore_paths = ignore::WalkBuilder::new(&path);
        Ok(ignore_paths
            .build()
            .flatten()
            .map(|x| camino::Utf8PathBuf::from_path_buf(x.into_path()).unwrap()) //todo: improve error message
            .collect::<Vec<camino::Utf8PathBuf>>())
    }
}

async fn read_ftd_test_file(
    ftd_document: fastn_core::Document,
    config: &fastn_core::Config,
) -> fastn_core::Result<()> {
    /*let parsed_document = ftd::interpreter::ParsedDocument::parse(
        ftd_document.id.as_str(),
        ftd_document.content.as_str(),
    )?;
    dbg!(&parsed_document.name, &parsed_document.ast);*/
    let req = fastn_core::http::Request::default();
    let base_url = "/";
    let mut req_config =
        fastn_core::RequestConfig::new(config, &req, ftd_document.id.as_str(), base_url);
    req_config.current_document = Some(ftd_document.id.to_string());
    /* let resp = fastn_core::package::package_doc::process_ftd(
        &mut req_config,
        doc,
        base_url,
        build_static_files,
        false,
        file_path.as_str(),
    )
        .await;*/

    let main_ftd_doc = fastn_core::doc::interpret_helper(
        ftd_document.id_with_package().as_str(),
        ftd_document.content.as_str(),
        &mut req_config,
        base_url,
        false,
        0,
    )
    .await?;
    dbg!(&main_ftd_doc.tree);
    Ok(())
}

pub fn test_fastn_ftd() -> &'static str {
    include_str!("../../test_fastn.ftd")
}
