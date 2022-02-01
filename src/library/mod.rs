use crate::utils::HasElements;

mod http;
mod include;
mod new_toc;
mod sqlite;
mod toc;

#[derive(Debug)]
pub struct Library {
    pub config: fpm::Config,
    pub markdown: Option<(String, String)>,
    pub document_id: String,
    pub translated_data: fpm::TranslationData,
    pub current_package: std::sync::Arc<std::sync::Mutex<Vec<fpm::Package>>>,
}

impl ftd::p2::Library for Library {
    fn get(&self, name: &str) -> Option<String> {
        if name == "fpm" {
            return Some(construct_fpm_base(self));
        }

        if name == "fpm-ui" {
            return Some(construct_fpm_ui(self));
        }

        if name == "fpm-lib" {
            return Some(fpm::fpm_lib_ftd().to_string());
        }

        if let Ok(mut packages) = self.current_package.lock() {
            let mut new_packages = packages.clone();
            while let Some(current_package) = new_packages.last() {
                if let Some((v, current_packages)) =
                    get_data_from_package(name, current_package, self, &new_packages)
                {
                    *packages = current_packages;
                    drop(new_packages);
                    return Some(v);
                }
                new_packages.pop();
            }
        }

        return None;

        fn get_data_from_package(
            name: &str,
            package: &fpm::Package,
            lib: &Library,
            current_packages: &[fpm::Package],
        ) -> Option<(String, Vec<fpm::Package>)> {
            let mut current_packages = current_packages.to_owned();
            let path = if package.name.eq(&lib.config.package.name) {
                lib.config.root.clone()
            } else {
                lib.config
                    .root
                    .join(".packages")
                    .join(package.name.as_str())
            };

            if let Ok(v) = std::fs::read_to_string(path.join(format!("{}.ftd", name))) {
                return Some((v, current_packages));
            }

            if let Some(o) = package.translation_of.as_ref() {
                let original_path = lib.config.root.join(".packages").join(o.name.as_str());
                if let Ok(v) = std::fs::read_to_string(original_path.join(format!("{}.ftd", name)))
                {
                    current_packages.push(o.clone());
                    return Some((v, current_packages));
                }
                if let Some((v, current_packages)) =
                    get_data_from_dependency(name, o, lib, current_packages.clone())
                {
                    return Some((v, current_packages));
                }
            }

            return get_data_from_dependency(name, package, lib, current_packages);

            fn get_data_from_dependency(
                name: &str,
                package: &fpm::Package,
                lib: &Library,
                mut current_packages: Vec<fpm::Package>,
            ) -> Option<(String, Vec<fpm::Package>)> {
                // Check for Aliases of the packages
                for (alias, package) in package.aliases().ok()? {
                    if name.starts_with(&alias) || name.starts_with(package.name.as_str()) {
                        // Non index document
                        let package_path = lib.config.root.join(".packages");
                        let non_alias_name = name.replacen(&alias, package.name.as_str(), 1);
                        if let Ok(v) = std::fs::read_to_string(
                            package_path.join(format!("{}.ftd", non_alias_name.as_str())),
                        ) {
                            current_packages.push(package.clone());
                            return Some((v, current_packages));
                        } else {
                            // Index document check for the alias
                            if let Ok(v) = std::fs::read_to_string(
                                package_path.join(format!("{}/index.ftd", non_alias_name.as_str())),
                            ) {
                                current_packages.push(package.clone());
                                return Some((v, current_packages));
                            }
                        }
                    }
                }
                None
            }
        }

        fn construct_fpm_ui(lib: &Library) -> String {
            let lang = match lib.config.package.language {
                Some(ref lang) => realm_lang::Language::from_2_letter_code(lang)
                    .unwrap_or(realm_lang::Language::English),
                None => realm_lang::Language::English,
            };

            let primary_lang = match lib.config.package.translation_of.as_ref() {
                Some(ref package) => match package.language {
                    Some(ref lang) => realm_lang::Language::from_2_letter_code(lang)
                        .unwrap_or(realm_lang::Language::English),
                    None => lang,
                },
                None => lang,
            };

            let current_document_last_modified_on =
                futures::executor::block_on(fpm::utils::get_current_document_last_modified_on(
                    &lib.config,
                    lib.document_id.as_str(),
                ));

            format!(
                indoc::indoc! {"
                    -- record ui-data:
                    string last-modified-on:
                    string never-synced:
                    string show-translation-status:
                    string other-available-languages:
                    string current-language:
                    string translation-not-available:
                    string unapproved-heading:
                    string show-unapproved-version:
                    string show-latest-version:
                    string show-outdated-version:
                    string out-dated-heading:
                    string out-dated-body:
                    string language-detail-page:
                    string language-detail-page-body:
                    string total-number-of-documents:
                    string document:
                    string status:
                    string missing:
                    string never-marked:
                    string out-dated:
                    string upto-date:
                    string welcome-fpm-page:
                    string welcome-fpm-page-subtitle:
                    string language:


                    -- ui-data ui:
                    last-modified-on: {last_modified_on}
                    never-synced: {never_synced}
                    show-translation-status: {show_translation_status}
                    other-available-languages: {other_available_languages}
                    current-language: {current_language}
                    translation-not-available: {translation_not_available}
                    unapproved-heading: {unapproved_heading}
                    show-unapproved-version: {show_unapproved_version}
                    show-latest-version: {show_latest_version}
                    show-outdated-version: {show_outdated_version}
                    out-dated-heading: {out_dated_heading}
                    out-dated-body: {out_dated_body}
                    language-detail-page: {language_detail_page}
                    language-detail-page-body: {language_detail_page_body}
                    total-number-of-documents: {total_number_of_documents}
                    document: {document}
                    status: {status}
                    missing: {missing}
                    never-marked: {never_marked}
                    out-dated: {out_dated}
                    upto-date: {upto_date}
                    welcome-fpm-page: {welcome_fpm_page}
                    welcome-fpm-page-subtitle: {welcome_fpm_page_subtitle}
                    language: {language}
                "},
                language = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "language",
                    &current_document_last_modified_on
                ),
                welcome_fpm_page_subtitle = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "welcome-fpm-page-subtitle",
                    &current_document_last_modified_on
                ),
                welcome_fpm_page = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "welcome-fpm-page",
                    &current_document_last_modified_on
                ),
                upto_date = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "upto-date",
                    &current_document_last_modified_on
                ),
                out_dated = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "out-dated",
                    &current_document_last_modified_on
                ),
                never_marked = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "never-marked",
                    &current_document_last_modified_on
                ),
                missing = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "missing",
                    &current_document_last_modified_on
                ),
                status = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "status",
                    &current_document_last_modified_on
                ),
                document = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "document",
                    &current_document_last_modified_on
                ),
                total_number_of_documents = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "total-number-of-documents",
                    &current_document_last_modified_on
                ),
                language_detail_page_body = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "language-detail-page-body",
                    &current_document_last_modified_on
                ),
                last_modified_on = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "last-modified-on",
                    &current_document_last_modified_on
                ),
                never_synced = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "never-synced",
                    &current_document_last_modified_on
                ),
                show_translation_status = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "show-translation-status",
                    &current_document_last_modified_on
                ),
                other_available_languages = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "other-available-languages",
                    &current_document_last_modified_on
                ),
                current_language = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "current-language",
                    &current_document_last_modified_on
                ),
                translation_not_available = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "translation-not-available",
                    &current_document_last_modified_on
                ),
                unapproved_heading = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "unapproved-heading",
                    &current_document_last_modified_on
                ),
                show_unapproved_version = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "show-unapproved-version",
                    &current_document_last_modified_on
                ),
                show_latest_version = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "show-latest-version",
                    &current_document_last_modified_on
                ),
                show_outdated_version = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "show-outdated-version",
                    &current_document_last_modified_on
                ),
                out_dated_heading = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "out-dated-heading",
                    &current_document_last_modified_on
                ),
                out_dated_body = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "out-dated-body",
                    &current_document_last_modified_on
                ),
                language_detail_page = fpm::i18n::translation::search(
                    &lang,
                    &primary_lang,
                    "language-detail-page",
                    &current_document_last_modified_on
                ),
            )
        }

        fn construct_fpm_base(lib: &Library) -> String {
            let mut fpm_base = format!(
                indoc::indoc! {"
                        {fpm_base}
                        {fpm_ui}
                        
                        -- boolean mobile: true
                        -- boolean dark-mode: false
                        -- boolean system-dark-mode: false
                        -- boolean follow-system-dark-mode: true
                        -- string document-id: {document_id}
                        -- optional string diff:
                        -- optional string last-marked-on:
                        -- optional string original-latest:
                        -- optional string translated-latest:
                        -- optional string last-marked-on-rfc3339:
                        -- optional string original-latest-rfc3339:
                        -- optional string translated-latest-rfc3339:
                        -- optional string language:
                        -- optional string number-of-documents:
                        -- optional string last-modified-on:
                        -- optional string current-document-last-modified-on:
                        -- string translation-status-url: {home_url}
                        -- string title: {title}
                        -- string package-name: {package_name}
                        -- optional string package-zip:
                        -- string home-url: {home_url}

                        -- record language-toc-item:
                        caption title:
                        string url:
                        language-toc-item list children:

                        -- language-toc-item list language-toc:
    
                        "},
                fpm_base = fpm::fpm_ftd(),
                fpm_ui = construct_fpm_ui(lib),
                document_id = lib.document_id,
                title = lib.config.package.name,
                package_name = lib.config.package.name,
                home_url = format!("//{}", lib.config.package.name)
            );

            if let Some(ref zip) = lib.config.package.zip {
                fpm_base = format!(
                    indoc::indoc! {"
                        {fpm_base}
                        
                        -- package-zip: {package_zip}
    
                        "},
                    fpm_base = fpm_base,
                    package_zip = zip,
                );
            }

            if let Some(ref diff) = lib.translated_data.diff {
                fpm_base = format!(
                    indoc::indoc! {"
                        {fpm_base}
                        
                        -- diff: 
                        
                        {diff}
    
                        "},
                    fpm_base = fpm_base,
                    diff = diff,
                );
            }

            if lib.config.package.translation_of.is_some()
                || lib.config.package.translations.has_elements()
            {
                fpm_base = format!(
                    indoc::indoc! {"
                        {fpm_base}
                        
                        -- translation-status-url: //{package_name}/FPM/translation-status
    
                        "},
                    fpm_base = fpm_base,
                    package_name = lib.config.package.name
                );
            }

            if let Ok(no_of_doc) =
                futures::executor::block_on(fpm::utils::get_no_of_document(&lib.config))
            {
                fpm_base = format!(
                    indoc::indoc! {"
                        {fpm_base}
                        
                        -- number-of-documents: {number_of_documents}
    
                        "},
                    fpm_base = fpm_base,
                    number_of_documents = no_of_doc,
                );
            }

            if let Some(last_modified_on) =
                futures::executor::block_on(fpm::utils::get_last_modified_on(&lib.config.root))
            {
                fpm_base = format!(
                    indoc::indoc! {"
                        {fpm_base}
                        
                        -- last-modified-on: {last_modified_on}
    
                        "},
                    fpm_base = fpm_base,
                    last_modified_on = last_modified_on,
                );
            }

            if let Some(last_modified_on) =
                futures::executor::block_on(fpm::utils::get_current_document_last_modified_on(
                    &lib.config,
                    lib.document_id.as_str(),
                ))
            {
                fpm_base = format!(
                    indoc::indoc! {"
                        {fpm_base}
                        
                        -- current-document-last-modified-on: {last_modified_on}
    
                        "},
                    fpm_base = fpm_base,
                    last_modified_on = last_modified_on,
                );
            }

            if let Some(ref language) = lib.config.package.language {
                fpm_base = format!(
                    indoc::indoc! {"
                        {fpm_base}
                        
                        -- language: {language} 
    
                        "},
                    fpm_base = fpm_base,
                    language = fpm::utils::language_to_human(language),
                );
            }

            if let Some(ref last_marked_on) = lib.translated_data.last_marked_on {
                let rfc3339 = fpm::utils::nanos_to_rfc3339(last_marked_on);
                fpm_base = format!(
                    indoc::indoc! {"
                        {fpm_base}
                        
                        -- last-marked-on: {last_marked_on}

                        -- last-marked-on-rfc3339: {rfc3339}
    
                        "},
                    fpm_base = fpm_base,
                    last_marked_on = last_marked_on,
                    rfc3339 = rfc3339
                );
            }
            if let Some(ref original_latest) = lib.translated_data.original_latest {
                let rfc3339 = fpm::utils::nanos_to_rfc3339(original_latest);
                fpm_base = format!(
                    indoc::indoc! {"
                        {fpm_base}
                        
                        -- original-latest: {original_latest}

                        -- original-latest-rfc3339: {rfc3339}
    
                        "},
                    fpm_base = fpm_base,
                    original_latest = original_latest,
                    rfc3339 = rfc3339
                );
            }
            if let Some(ref translated_latest) = lib.translated_data.translated_latest {
                let rfc3339 = fpm::utils::nanos_to_rfc3339(translated_latest);
                fpm_base = format!(
                    indoc::indoc! {"
                        {fpm_base}
                        
                        -- translated-latest: {translated_latest}

                        -- translated-latest-rfc3339: {rfc3339}
    
                        "},
                    fpm_base = fpm_base,
                    translated_latest = translated_latest,
                    rfc3339 = rfc3339
                );
            }
            if let Some((filename, content)) = lib.markdown.as_ref() {
                fpm_base = format!(
                    indoc::indoc! {"
                        {fpm_base}
                        
                        -- string markdown-filename: {filename}
                        
                        -- string markdown-content:
    
                        {content}
                    "},
                    fpm_base = fpm_base,
                    filename = filename,
                    content = content
                );
            }

            if let Ok(original_path) = lib.config.original_path() {
                if let Ok(original_snapshots) =
                    futures::executor::block_on(fpm::snapshot::get_latest_snapshots(&original_path))
                {
                    if let Ok(translation_status) =
                        fpm::commands::translation_status::get_translation_status(
                            &original_snapshots,
                            &lib.config.root,
                        )
                    {
                        let mut never_marked_files = "".to_string();
                        let mut missing_files = "".to_string();
                        let mut outdated_files = "".to_string();
                        let mut upto_date_files = "".to_string();
                        let mut translation_status_list = "".to_string();

                        for (file, status) in translation_status.iter() {
                            translation_status_list = format!(
                                indoc::indoc! {"
                                    {list}
                                    
                                    -- status:
                                    file: {file}
                                    status: {status}
                                    
                                "},
                                list = translation_status_list,
                                file = file,
                                status = status.as_str()
                            );

                            match status {
                                fpm::commands::translation_status::TranslationStatus::Missing => {
                                    missing_files = format!(
                                        indoc::indoc! {"
                                            {list}
                                            
                                            -- missing-files: {file}
                                            
                                        "},
                                        list = missing_files,
                                        file = file,
                                    );
                                },
                                fpm::commands::translation_status::TranslationStatus::NeverMarked => {
                                    never_marked_files = format!(
                                        indoc::indoc! {"
                                            {list}
                                            
                                            -- never-marked-files: {file}
                                            
                                        "},
                                        list = never_marked_files,
                                        file = file,
                                    );
                                },
                                fpm::commands::translation_status::TranslationStatus::Outdated => {
                                    outdated_files = format!(
                                        indoc::indoc! {"
                                            {list}
                                            
                                            -- outdated-files: {file}
                                            
                                        "},
                                        list = outdated_files,
                                        file = file,
                                    );
                                }
                                fpm::commands::translation_status::TranslationStatus::UptoDate => {
                                    upto_date_files = format!(
                                        indoc::indoc! {"
                                            {list}
                                            
                                            -- upto-date-files: {file}
                                            
                                        "},
                                        list = upto_date_files,
                                        file = file,
                                    );
                                }
                            }
                        }

                        fpm_base = format!(
                            indoc::indoc! {"
                                {fpm_base}
                                
                                -- record status-data:
                                string file:
                                string status:
                                
                                -- status-data list status:
        
                                {translation_status_list}

                                -- string list missing-files:
                                
                                {missing_files}

                                -- string list never-marked-files:
                                
                                {never_marked_files}

                                -- string list outdated-files:
                                
                                {outdated_files}

                                -- string list upto-date-files:
                                
                                {upto_date_files}

                                
                            "},
                            fpm_base = fpm_base,
                            translation_status_list = translation_status_list,
                            missing_files = missing_files,
                            never_marked_files = never_marked_files,
                            outdated_files = outdated_files,
                            upto_date_files = upto_date_files
                        );
                    }
                }
            }

            if lib.config.package.translations.has_elements() {
                let mut translation_status_list = "".to_string();
                for translation in lib.config.package.translations.iter() {
                    if let Some(ref status) = translation.translation_status {
                        if let Some(ref language) = translation.language {
                            let url =
                                format!("https://{}/FPM/translation-status/", translation.name);
                            let status = {
                                let mut status_data = format!(
                                    indoc::indoc! {"
                                        -- status:
                                        language: {language}
                                        url: {url}
                                        never-marked: {never_marked}
                                        missing: {missing}
                                        out-dated: {out_dated}
                                        upto-date: {upto_date}
                                    "},
                                    language = language,
                                    url = url,
                                    never_marked = status.never_marked,
                                    missing = status.missing,
                                    out_dated = status.out_dated,
                                    upto_date = status.upto_date
                                );
                                if let Some(ref last_modified_on) = status.last_modified_on {
                                    status_data = format!(
                                        indoc::indoc! {"
                                            {status}last-modified-on: {last_modified_on}
                                        "},
                                        status = status_data,
                                        last_modified_on = last_modified_on
                                    );
                                }
                                status_data
                            };
                            translation_status_list = format!(
                                indoc::indoc! {"
                                    {list}
                                    
                                    {status}
                                    
                                "},
                                list = translation_status_list,
                                status = status
                            );
                        }
                    }

                    fpm_base = format!(
                        indoc::indoc! {"
                        {fpm_base}
                        
                        -- record status-data:
                        string language:
                        string url:
                        integer never-marked:
                        integer missing:
                        integer out-dated:
                        integer upto-date:
                        optional string last-modified-on:
                        
                        -- status-data list status:

                        {translation_status_list}
                        
                    "},
                        fpm_base = fpm_base,
                        translation_status_list = translation_status_list
                    );
                }
            }

            let other_language_packages =
                if let Some(translation_of) = lib.config.package.translation_of.as_ref() {
                    let mut other_language_packages = translation_of
                        .translations
                        .iter()
                        .collect::<Vec<&fpm::Package>>();
                    other_language_packages.insert(0, translation_of);
                    other_language_packages
                } else {
                    lib.config
                        .package
                        .translations
                        .iter()
                        .collect::<Vec<&fpm::Package>>()
                };

            if other_language_packages.has_elements() {
                let mut languages = "".to_string();
                let doc_id = if lib.document_id.eq("index.ftd") {
                    "".to_string()
                } else {
                    lib.document_id.replace(".ftd", "/")
                };

                for lang_package in other_language_packages {
                    let language = if let Some(ref lang) = lang_package.language {
                        fpm::utils::language_to_human(lang)
                    } else {
                        continue;
                    };

                    let domain = if lang_package.name.ends_with('/') {
                        format!("https://{}{}", lang_package.name, doc_id)
                    } else {
                        format!("https://{}/{}", lang_package.name, doc_id)
                    };

                    languages = format!(
                        indoc::indoc! {"
                        {languages}
                        - {domain}
                          {language}

                        "},
                        languages = languages,
                        domain = domain,
                        language = language
                    );
                }

                if !languages.trim().is_empty() {
                    fpm_base = format!(
                        indoc::indoc! {"
                        {fpm_base}
                        
                        -- language-toc:
                        $processor$: toc

                        {languages}

                        "},
                        fpm_base = fpm_base,
                        languages = languages
                    );
                }
            }

            fpm_base
        }
    }

    fn process(
        &self,
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<ftd::Value> {
        match section
            .header
            .str(doc.name, section.line_number, "$processor$")?
        {
            "toc" => fpm::library::toc::processor(section, doc),
            "http" => fpm::library::http::processor(section, doc),
            "package-query" => fpm::library::sqlite::processor(section, doc, &self.config),
            "toc-v2" => fpm::library::new_toc::processor(section, doc, &self.config),
            "include" => fpm::library::include::processor(section, doc, &self.config),
            t => unimplemented!("$processor$: {} is not implemented yet", t),
        }
    }
}

#[derive(Default)]
pub struct FPMLibrary {}

impl ftd::p2::Library for FPMLibrary {
    fn get(&self, name: &str) -> Option<String> {
        if name == "fpm" {
            return Some(fpm::fpm_ftd().to_string());
        } else {
            std::fs::read_to_string(format!("./{}.ftd", name)).ok()
        }
    }

    fn process(
        &self,
        _section: &ftd::p1::Section,
        _doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<ftd::Value> {
        unimplemented!("not implemented yet");
    }
}
