fn construct_fastn_cli_variables(_lib: &fastn_core::Library) -> String {
    format!(
        indoc::indoc! {"
        -- fastn.build-info info:
        cli-version: {cli_version}
        cli-git-commit-hash: {cli_git_commit_hash}
        cli-created-on: {cli_created_on}
        build-created-on: {build_created_on}
        ftd-version: {ftd_version}
    "},
        cli_version = if fastn_core::utils::is_test() {
            "FASTN_CLI_VERSION"
        } else {
            env!("CARGO_PKG_VERSION")
        },
        cli_git_commit_hash = if fastn_core::utils::is_test() {
            "FASTN_CLI_GIT_HASH"
        } else {
            option_env!("GITHUB_SHA").unwrap_or("unknown-sha")
        },
        cli_created_on = if fastn_core::utils::is_test() {
            "FASTN_CLI_BUILD_TIMESTAMP"
        } else {
            // TODO: calculate this in github action and pass it, vergen is too heave a dependency
            option_env!("FASTN_CLI_BUILD_TIMESTAMP").unwrap_or("0")
        },
        ftd_version = if fastn_core::utils::is_test() {
            "FTD_VERSION"
        } else {
            ""
            // TODO
        },
        build_created_on = if fastn_core::utils::is_test() {
            String::from("BUILD_CREATE_TIMESTAMP")
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_string()
        }
    )
}

pub(crate) async fn get2022_(lib: &fastn_core::Library) -> String {
    #[allow(clippy::format_in_format_args)]
    let mut fastn_base = format!(
        indoc::indoc! {"
            {fastn_base}
            {capital_fastn}

            {build_info}

            -- string document-name: {document_id}
            -- string package-title: {title}
            -- string package-name: {package_name}
            -- string home-url: {home_url}
        "},
        fastn_base = fastn_package::old_fastn::fastn_ftd_2021(),
        capital_fastn = capital_fastn(lib),
        build_info = construct_fastn_cli_variables(lib),
        document_id = lib.document_id,
        title = lib.config.config.package.name,
        package_name = lib.config.config.package.name,
        home_url = format!("https://{}", lib.config.config.package.name),
    );

    if let Ok(number_of_documents) = futures::executor::block_on(
        fastn_core::utils::get_number_of_documents(&lib.config.config),
    ) {
        fastn_base = format!(
            indoc::indoc! {"
                {fastn_base}
                
                -- number-of-documents: {number_of_documents}    
            "},
            fastn_base = fastn_base,
            number_of_documents = number_of_documents,
        );
    }

    if let Some((ref filename, ref content)) = lib.markdown {
        fastn_base = format!(
            indoc::indoc! {"
                {fastn_base}
                
                -- string markdown-filename: {filename}                        
                -- string markdown-content:
    
                {content}
            "},
            fastn_base = fastn_base,
            filename = filename,
            content = content,
        );
    }

    fastn_base
}

pub(crate) async fn get2022(lib: &fastn_core::Library2022) -> String {
    let lib = fastn_core::Library {
        config: lib.clone(),
        markdown: lib.markdown.clone(),
        document_id: lib.document_id.clone(),
        translated_data: lib.translated_data.clone(),
        asset_documents: Default::default(),
        base_url: lib.base_url.clone(),
    };
    get2022_(&lib).await
}

fn capital_fastn(lib: &fastn_core::Library) -> String {
    let mut s = format!(
        indoc::indoc! {"
            -- package-data package: {package_name}
        "},
        package_name = lib.config.config.package.name,
    );

    if let Some(ref zip) = lib.config.config.package.zip {
        s.push_str(format!("zip: {zip}").as_str());
    }

    if let Some(ref favicon) = lib.config.config.package.favicon {
        s.push_str(format!("\nfavicon: {favicon}").as_str());
    }

    s
}
