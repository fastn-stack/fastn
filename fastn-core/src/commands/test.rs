pub(crate) const TEST_FOLDER: &str = "_tests";
pub(crate) const TEST_FILE_EXTENSION: &str = ".test.ftd";

pub async fn test(
    config: &fastn_core::Config,
    only_id: Option<&str>,
    _base_url: &str,
    headless: bool,
) -> fastn_core::Result<()> {
    use colored::Colorize;

    if !headless {
        return fastn_core::usage_error(
            "Currently headless mode is only suuported, use: --headless flag".to_string(),
        );
    }
    let ftd_documents = config.get_test_files().await?;

    for document in ftd_documents {
        if let Some(id) = only_id {
            if !document.id.contains(id) {
                continue;
            }
        }
        println!("Running test in {}", document.id.yellow());
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
        Ok(ignore::WalkBuilder::new(path)
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
    let req = fastn_core::http::Request::default();
    let base_url = "/";
    let mut req_config =
        fastn_core::RequestConfig::new(config, &req, ftd_document.id.as_str(), base_url);
    req_config.current_document = Some(ftd_document.id.to_string());
    let main_ftd_doc = fastn_core::doc::interpret_helper(
        ftd_document.id_with_package().as_str(),
        ftd_document.content.as_str(),
        &mut req_config,
        base_url,
        false,
        0,
    )
    .await?;

    let doc = ftd::interpreter::TDoc::new(
        &main_ftd_doc.name,
        &main_ftd_doc.aliases,
        &main_ftd_doc.data,
    );

    for instruction in main_ftd_doc.tree {
        if !execute_instruction(&instruction, &doc, config).await? {
            break;
        }
    }
    Ok(())
}

async fn execute_instruction(
    instruction: &ftd::interpreter::Component,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::Config,
) -> fastn_core::Result<bool> {
    match instruction.name.as_str() {
        "fastn#get" => execute_get_instruction(instruction, doc, config).await,
        "fastn#post" => todo!(),
        t => fastn_core::usage_error(format!(
            "Unknown instruction {}, line number: {}",
            t, instruction.line_number
        )),
    }
}

async fn execute_get_instruction(
    instruction: &ftd::interpreter::Component,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::Config,
) -> fastn_core::Result<bool> {
    let property_values = instruction.get_interpreter_property_value_of_all_arguments(doc);
    let url = get_value_ok("url", &property_values, instruction.line_number)?
        .to_string(doc)?
        .unwrap();
    let title = get_value_ok("title", &property_values, instruction.line_number)?
        .to_string(doc)?
        .unwrap();
    let test = get_value_ok("test", &property_values, instruction.line_number)?
        .to_string(doc)?
        .unwrap();

    get_js_for_id(url.as_str(), test.as_str(), title.as_str(), config).await
}

async fn get_js_for_id(
    id: &str,
    test: &str,
    title: &str,
    config: &fastn_core::Config,
) -> fastn_core::Result<bool> {
    use actix_web::body::MessageBody;
    use colored::Colorize;

    print!("{}:  ", title.yellow());
    let mut request = fastn_core::http::Request::default();
    request.path = id.to_string();
    let response = fastn_core::commands::serve::serve_helper(config, request, true).await?;
    let body = response.into_body().try_into_bytes().unwrap(); // Todo: Throw error
    let body_str = std::str::from_utf8(&body).unwrap(); // Todo: Throw error
    let fastn_test_js = fastn_js::fastn_test_js();
    let test_string = format!("{body_str}\n{fastn_test_js}\n{test}\nfastn.test_result");
    let test_result = fastn_js::run_test(test_string.as_str());
    if test_result.iter().any(|v| !(*v)) {
        println!("{}", "Test Failed".red());
        return Ok(false);
    }
    println!("{}", "Test Passed".green());
    Ok(true)
}

fn get_value_ok(
    key: &str,
    property_values: &ftd::Map<ftd::interpreter::PropertyValue>,
    line_number: usize,
) -> fastn_core::Result<ftd::interpreter::Value> {
    get_value(key, property_values).ok_or(fastn_core::Error::NotFound(format!(
        "Key '{}' not found, line number: {}",
        key, line_number
    )))
}

fn get_value(
    key: &str,
    property_values: &ftd::Map<ftd::interpreter::PropertyValue>,
) -> Option<ftd::interpreter::Value> {
    let property_value = property_values.get(key)?;
    match property_value {
        ftd::interpreter::PropertyValue::Value { value, .. } => Some(value.clone()),
        _ => unimplemented!(),
    }
}

pub fn test_fastn_ftd() -> &'static str {
    include_str!("../../test_fastn.ftd")
}
