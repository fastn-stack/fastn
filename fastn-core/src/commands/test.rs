pub(crate) const TEST_FOLDER: &str = "_tests";
pub(crate) const TEST_FILE_EXTENSION: &str = ".test.ftd";

// mandatory test parameters
pub(crate) const TEST_TITLE_HEADER: &str = "title";
pub(crate) const TEST_URL_HEADER: &str = "url";

// optional test parameters
pub(crate) const QUERY_PARAMS_HEADER: &str = "query-params";
pub(crate) const QUERY_PARAMS_HEADER_KEY: &str = "key";
pub(crate) const QUERY_PARAMS_HEADER_VALUE: &str = "value";
pub(crate) const POST_BODY_HEADER: &str = "body";
pub(crate) const TEST_CONTENT_HEADER: &str = "test";
pub(crate) const HTTP_REDIRECT_HEADER: &str = "http-redirect";
pub(crate) const HTTP_STATUS_HEADER: &str = "http-status";
pub(crate) const HTTP_LOCATION_HEADER: &str = "http-location";

pub async fn test(
    config: &fastn_core::Config,
    only_id: Option<&str>,
    _base_url: &str,
    headless: bool,
    script: bool,
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
        println!("Running test file: {}", document.id.magenta());
        read_ftd_test_file(document, config, script).await?;
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

    pub(crate) fn get_test_directory_path(&self) -> camino::Utf8PathBuf {
        self.get_root_for_package(&self.package)
            .join(fastn_core::commands::test::TEST_FOLDER)
    }
}

async fn read_ftd_test_file(
    ftd_document: fastn_core::Document,
    config: &fastn_core::Config,
    script: bool,
) -> fastn_core::Result<()> {
    let req = fastn_core::http::Request::default();
    let mut saved_cookies: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
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
    let mut instruction_number = 1;
    for instruction in main_ftd_doc.tree {
        if !execute_instruction(
            &instruction,
            &doc,
            config,
            &mut saved_cookies,
            script,
            instruction_number,
        )
        .await?
        {
            break;
        }
        instruction_number += 1;
    }
    Ok(())
}

async fn execute_instruction(
    instruction: &ftd::interpreter::Component,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::Config,
    saved_cookies: &mut std::collections::HashMap<String, String>,
    script: bool,
    instruction_number: i64,
) -> fastn_core::Result<bool> {
    match instruction.name.as_str() {
        "fastn#get" => {
            execute_get_instruction(
                instruction,
                doc,
                config,
                saved_cookies,
                script,
                instruction_number,
            )
            .await
        }
        "fastn#post" => {
            execute_post_instruction(
                instruction,
                doc,
                config,
                saved_cookies,
                script,
                instruction_number,
            )
            .await
        }
        t => fastn_core::usage_error(format!(
            "Unknown instruction {}, line number: {}",
            t, instruction.line_number
        )),
    }
}

async fn execute_post_instruction(
    instruction: &ftd::interpreter::Component,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::Config,
    saved_cookies: &mut std::collections::HashMap<String, String>,
    script: bool,
    _instruction_number: i64,
) -> fastn_core::Result<bool> {
    let property_values = instruction.get_interpreter_property_value_of_all_arguments(doc);

    // Mandatory test parameters --------------------------------
    let url = get_value_ok(TEST_URL_HEADER, &property_values, instruction.line_number)?
        .to_string(doc, false)?
        .unwrap();
    let title = get_value_ok(TEST_TITLE_HEADER, &property_values, instruction.line_number)?
        .to_string(doc, false)?
        .unwrap();

    // Optional test parameters --------------------------------
    let mut other_params: ftd::Map<String> = ftd::Map::new();

    if let Some(test_content) =
        get_optional_value_string(TEST_CONTENT_HEADER, &property_values, doc)?
    {
        other_params.insert(TEST_CONTENT_HEADER.to_string(), test_content);
    }

    if let Some(post_body) = get_optional_value_string(POST_BODY_HEADER, &property_values, doc)? {
        other_params.insert(POST_BODY_HEADER.to_string(), post_body);
    }

    if let Some(http_status) = get_optional_value_string(HTTP_STATUS_HEADER, &property_values, doc)?
    {
        other_params.insert(HTTP_STATUS_HEADER.to_string(), http_status);
    }

    if let Some(http_location) =
        get_optional_value_string(HTTP_LOCATION_HEADER, &property_values, doc)?
    {
        other_params.insert(HTTP_LOCATION_HEADER.to_string(), http_location);
    }

    if let Some(http_redirect) =
        get_optional_value_string(HTTP_REDIRECT_HEADER, &property_values, doc)?
    {
        other_params.insert(HTTP_REDIRECT_HEADER.to_string(), http_redirect);
    }

    assert_optional_headers(&other_params)?;

    get_post_response_for_id(
        url.as_str(),
        title.as_str(),
        other_params,
        config,
        saved_cookies,
        script,
    )
    .await
}

async fn get_post_response_for_id(
    id: &str,
    title: &str,
    optional_params: ftd::Map<String>,
    config: &fastn_core::Config,
    saved_cookies: &mut std::collections::HashMap<String, String>,
    _script: bool,
) -> fastn_core::Result<bool> {
    use actix_web::body::MessageBody;
    use colored::Colorize;

    println!("Test: {}  ", title.yellow());
    let req_body = optional_params
        .get(POST_BODY_HEADER)
        .cloned()
        .unwrap_or_default();
    let post_body = actix_web::web::Bytes::copy_from_slice(req_body.as_bytes());
    let mut request = fastn_core::http::Request::default();
    request.path = id.to_string();
    request.set_method("post");
    request.set_body(post_body);
    request.insert_header(reqwest::header::CONTENT_TYPE, "application/json");
    request.set_cookies(&saved_cookies);
    let response = fastn_core::commands::serve::serve_helper(config, request, true).await?;
    update_cookies(saved_cookies, &response);

    let (response_status_code, response_location) = assert_response(&response, &optional_params)?;
    let test = optional_params.get(TEST_CONTENT_HEADER);
    if let Some(test_content) = test {
        let body = response.into_body().try_into_bytes().unwrap(); // Todo: Throw error
        let response_body = format!(
            "fastn.http_response = {}",
            std::str::from_utf8(&body).unwrap()
        );
        // Todo: Throw error
        let fastn_test_js = fastn_js::fastn_test_js();
        let fastn_assertion_headers =
            fastn_js::fastn_assertion_headers(response_status_code, response_location.as_str());
        let fastn_js = fastn_js::all_js_without_test_and_ftd_langugage_js();
        let test_string = format!(
            "{fastn_js}\n\n{response_body}\n{fastn_assertion_headers}\n{fastn_test_js}\n\
            {test_content}\nfastn.test_result"
        );
        let test_result = fastn_js::run_test(test_string.as_str());
        if test_result.iter().any(|v| !(*v)) {
            println!("{}", "Test Failed".red());
            return Ok(false);
        }
    }
    println!("{}", "Test Passed".green());
    Ok(true)
}

async fn execute_get_instruction(
    instruction: &ftd::interpreter::Component,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::Config,
    saved_cookies: &mut std::collections::HashMap<String, String>,
    script: bool,
    instruction_number: i64,
) -> fastn_core::Result<bool> {
    let property_values = instruction.get_interpreter_property_value_of_all_arguments(doc);

    // Mandatory test parameters --------------------------------
    let url = get_value_ok(TEST_URL_HEADER, &property_values, instruction.line_number)?
        .to_string(doc, false)?
        .unwrap();
    let title = get_value_ok(TEST_TITLE_HEADER, &property_values, instruction.line_number)?
        .to_string(doc, false)?
        .unwrap();

    // Optional test parameters --------------------------------
    let mut optional_params: ftd::Map<String> = ftd::Map::new();

    if let Some(query_params) = get_optional_value_list(QUERY_PARAMS_HEADER, &property_values, doc)?
    {
        let mut query_strings = vec![];
        for query in query_params.iter() {
            if let ftd::interpreter::Value::Record { fields, .. } = query {
                let resolved_key = fields
                    .get(QUERY_PARAMS_HEADER_KEY)
                    .unwrap()
                    .clone()
                    .resolve(doc, 0)?
                    .to_string(doc, false)?
                    .unwrap();
                let resolved_value = fields
                    .get(QUERY_PARAMS_HEADER_VALUE)
                    .unwrap()
                    .clone()
                    .resolve(doc, 0)?
                    .to_string(doc, false)?
                    .unwrap();
                let query_key_value =
                    format!("{}={}", resolved_key.as_str(), resolved_value.as_str());
                query_strings.push(query_key_value);
            }
        }
        if !query_strings.is_empty() {
            let query_string = query_strings.join("&").to_string();
            optional_params.insert(QUERY_PARAMS_HEADER.to_string(), query_string);
        }
    }

    if let Some(test_content) =
        get_optional_value_string(TEST_CONTENT_HEADER, &property_values, doc)?
    {
        optional_params.insert(TEST_CONTENT_HEADER.to_string(), test_content);
    }

    if let Some(http_status) = get_optional_value_string(HTTP_STATUS_HEADER, &property_values, doc)?
    {
        optional_params.insert(HTTP_STATUS_HEADER.to_string(), http_status);
    }

    if let Some(http_location) =
        get_optional_value_string(HTTP_LOCATION_HEADER, &property_values, doc)?
    {
        optional_params.insert(HTTP_LOCATION_HEADER.to_string(), http_location);
    }

    if let Some(http_redirect) =
        get_optional_value_string(HTTP_REDIRECT_HEADER, &property_values, doc)?
    {
        optional_params.insert(HTTP_REDIRECT_HEADER.to_string(), http_redirect);
    }

    assert_optional_headers(&optional_params)?;

    get_js_for_id(
        url.as_str(),
        title.as_str(),
        optional_params,
        config,
        saved_cookies,
        script,
        doc.name,
        instruction_number,
    )
    .await
}

async fn get_js_for_id(
    id: &str,
    title: &str,
    optional_params: ftd::Map<String>,
    config: &fastn_core::Config,
    saved_cookies: &mut std::collections::HashMap<String, String>,
    script: bool,
    doc_name: &str,
    instruction_number: i64,
) -> fastn_core::Result<bool> {
    use actix_web::body::MessageBody;
    use colored::Colorize;

    println!("Test: {}  ", title.yellow());
    let mut request = fastn_core::http::Request::default();
    request.path = id.to_string();
    if let Some(query_string) = optional_params.get(QUERY_PARAMS_HEADER) {
        request.set_query_string(query_string.as_str());
    }
    request.set_method("get");
    request.set_cookies(&saved_cookies);
    let response = fastn_core::commands::serve::serve_helper(config, request, true).await?;
    update_cookies(saved_cookies, &response);

    let (response_status_code, response_location) = assert_response(&response, &optional_params)?;
    let test = optional_params.get(TEST_CONTENT_HEADER);
    if let Some(test_content) = test {
        let body = response.into_body().try_into_bytes().unwrap(); // Todo: Throw error
        let response_body = format!(
            "fastn.http_response = {}",
            std::str::from_utf8(&body).unwrap()
        );
        let fastn_test_js = fastn_js::fastn_test_js();
        let fastn_assertion_headers =
            fastn_js::fastn_assertion_headers(response_status_code, response_location.as_str());
        let fastn_js = fastn_js::all_js_without_test_and_ftd_langugage_js();
        let test_string = format!(
            "{fastn_js}\n{response_body}\n{fastn_assertion_headers}\n{fastn_test_js}\n\
            {test_content}\nfastn.test_result"
        );
        if script {
            let mut test_file_name = doc_name.to_string();
            if let Some((_, file_name)) = test_file_name.trim_end_matches('/').rsplit_once('/') {
                test_file_name = file_name.to_string();
            }
            fastn_js::generate_script_file(
                test_string.as_str(),
                config.get_test_directory_path(),
                test_file_name
                    .replace(".test", format!(".t{}.test", instruction_number).as_str())
                    .as_str(),
            );
            println!("{}", "Script file created".green());
            return Ok(true);
        }
        let test_result = fastn_js::run_test(test_string.as_str());
        if test_result.iter().any(|v| !(*v)) {
            println!("{}", "Test Failed".red());
            return Ok(false);
        }
    }
    println!("{}", "Test Passed".green());
    Ok(true)
}

fn update_cookies(
    saved_cookies: &mut std::collections::HashMap<String, String>,
    response: &actix_web::HttpResponse,
) {
    for ref c in response.cookies() {
        saved_cookies.insert(c.name().to_string(), c.value().to_string());
    }
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

fn get_optional_value(
    key: &str,
    property_values: &ftd::Map<ftd::interpreter::PropertyValue>,
) -> Option<ftd::interpreter::Value> {
    if let Some(property_value) = property_values.get(key) {
        return match property_value {
            ftd::interpreter::PropertyValue::Value { value, .. } => Some(value.clone()),
            _ => unimplemented!(),
        };
    }
    None
}

fn get_optional_value_list(
    key: &str,
    property_values: &ftd::Map<ftd::interpreter::PropertyValue>,
    doc: &ftd::interpreter::TDoc<'_>,
) -> ftd::interpreter::Result<Option<Vec<ftd::interpreter::Value>>> {
    let value = get_optional_value(key, property_values);
    if let Some(ref value) = value {
        return value.to_list(doc, false);
    }
    Ok(None)
}

fn get_optional_value_string(
    key: &str,
    property_values: &ftd::Map<ftd::interpreter::PropertyValue>,
    doc: &ftd::interpreter::TDoc<'_>,
) -> ftd::interpreter::Result<Option<String>> {
    let value = get_optional_value(key, property_values);
    if let Some(ref value) = value {
        return value.to_string(doc, false);
    }
    Ok(None)
}

pub fn test_fastn_ftd() -> &'static str {
    include_str!("../../test_fastn.ftd")
}

pub fn assert_optional_headers(
    optional_test_parameters: &ftd::Map<String>,
) -> fastn_core::Result<bool> {
    if (optional_test_parameters.contains_key(HTTP_STATUS_HEADER)
        || optional_test_parameters.contains_key(HTTP_LOCATION_HEADER))
        && optional_test_parameters.contains_key(HTTP_REDIRECT_HEADER)
    {
        return fastn_core::usage_error(format!(
            "Use either [{} and {}] or [{}] both not allowed.",
            HTTP_STATUS_HEADER, HTTP_LOCATION_HEADER, HTTP_REDIRECT_HEADER
        ));
    }
    Ok(true)
}

pub fn assert_response(
    response: &fastn_core::http::Response,
    params: &ftd::Map<String>,
) -> fastn_core::Result<(u16, String)> {
    if let Some(redirection_url) = params.get(HTTP_REDIRECT_HEADER) {
        return assert_redirect(response, redirection_url);
    }

    assert_location_and_status(response, params)
}

pub fn assert_redirect(
    response: &fastn_core::http::Response,
    redirection_location: &str,
) -> fastn_core::Result<(u16, String)> {
    let response_status_code = response.status().as_u16();
    if !response.status().is_redirection() {
        return fastn_core::assert_error(format!(
            "Invalid redirect status code {:?}",
            response.status().as_u16()
        ));
    }

    let response_location = get_response_location(response)?.unwrap_or_default();
    if !response_location.eq(redirection_location) {
        return fastn_core::assert_error(format!(
            "HTTP redirect location mismatch. Expected \"{:?}\", Found \"{:?}\"",
            redirection_location, response_location
        ));
    }

    Ok((response_status_code, response_location))
}

pub fn assert_location_and_status(
    response: &fastn_core::http::Response,
    params: &ftd::Map<String>,
) -> fastn_core::Result<(u16, String)> {
    // By default, we are expecting status 200 if not http-status is not passed
    let default_status_code = "200".to_string();
    let response_status_code = response.status().as_u16();
    let response_status_code_string = response_status_code.to_string();
    let expected_status_code = params
        .get(HTTP_STATUS_HEADER)
        .unwrap_or(&default_status_code);

    if !response_status_code_string.eq(expected_status_code) {
        return fastn_core::assert_error(format!(
            "HTTP status code mismatch. Expected {}, Found {}",
            expected_status_code, response_status_code
        ));
    }

    let response_location = get_response_location(response)?.unwrap_or_default();
    let expected_location = params.get(HTTP_LOCATION_HEADER);

    if let Some(expected_location) = expected_location {
        if !expected_location.eq(response_location.as_str()) {
            return fastn_core::assert_error(format!(
                "HTTP Location mismatch. Expected \"{:?}\", Found \"{:?}\"",
                expected_location, response_location
            ));
        }
    }

    Ok((response_status_code, response_location))
}

pub fn get_response_location(
    response: &fastn_core::http::Response,
) -> fastn_core::Result<Option<String>> {
    if let Some(redirect_location) = response.headers().get("Location") {
        return if let Ok(location) = redirect_location.to_str() {
            Ok(Some(location.to_string()))
        } else {
            fastn_core::generic_error("Failed to convert 'Location' header to string".to_string())
        };
    }
    return Ok(None);
}
