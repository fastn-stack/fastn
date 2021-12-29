use crate::utils::HasElements;

mod http;
mod sqlite;
mod toc;

pub struct Library {
    pub config: fpm::Config,
    pub markdown: Option<(String, String)>,
    pub document_id: String,
    pub translated_data: fpm::TranslationData,
}

impl ftd::p2::Library for Library {
    fn get(&self, name: &str) -> Option<String> {
        if name == "fpm" {
            return Some(construct_fpm_base(self));
        }

        if let Ok(v) = std::fs::read_to_string(self.config.root.join(format!("{}.ftd", name))) {
            return Some(v);
        }

        if let Ok(original_path) = self.config.original_path() {
            if let Ok(v) = std::fs::read_to_string(original_path.join(format!("{}.ftd", name))) {
                return Some(v);
            }
        }

        let package_path = self.config.root.join(".packages");
        if let Ok(v) = std::fs::read_to_string(package_path.join(format!("{}.ftd", name))) {
            return Some(v);
        }
        // Check for Aliases of the packages
        for (alias, package) in self.config.aliases().ok()? {
            if name.starts_with(&alias) {
                // Non index document
                let non_alias_name = name.replacen(&alias, package.name.as_str(), 1);
                if let Ok(v) = std::fs::read_to_string(
                    package_path.join(format!("{}.ftd", non_alias_name.as_str())),
                ) {
                    return Some(v);
                } else {
                    // Index document check for the alias
                    if let Ok(v) = std::fs::read_to_string(
                        package_path.join(format!("{}/index.ftd", non_alias_name.as_str())),
                    ) {
                        return Some(v);
                    }
                }
            }
        }

        return std::fs::read_to_string(package_path.join(format!("{}/index.ftd", name))).ok();

        fn construct_fpm_base(lib: &Library) -> String {
            let mut fpm_base = format!(
                indoc::indoc! {"
                        {fpm_base}
                        
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
    
                        "},
                fpm_base = fpm::fpm_ftd().to_string(),
                document_id = lib.document_id,
            );
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

            if let Ok(no_of_doc) =
                futures::executor::block_on(fpm::utils::get_no_of_document(&lib.config.root))
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

            if let Some(ref language) = lib.config.package.language {
                fpm_base = format!(
                    indoc::indoc! {"
                        {fpm_base}
                        
                        -- language: {language} 
    
                        "},
                    fpm_base = fpm_base,
                    language = language,
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
                        let mut translation_status_list = "".to_string();

                        for (file, status) in translation_status {
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
                        }

                        fpm_base = format!(
                            indoc::indoc! {"
                                {fpm_base}
                                
                                -- record status-data:
                                string file:
                                string status:
                                
                                -- status-data list status:
        
                                {translation_status_list}
                                
                            "},
                            fpm_base = fpm_base,
                            translation_status_list = translation_status_list
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
                            translation_status_list = format!(
                                indoc::indoc! {"
                                    {list}
                                    
                                    -- status:
                                    language: {language}
                                    url: {url}
                                    never-marked: {never_marked}
                                    missing: {missing}
                                    out-dated: {out_dated}
                                    upto-date: {upto_date}
                                    
                                "},
                                list = translation_status_list,
                                language = language,
                                url = url,
                                never_marked = status.never_marked,
                                missing = status.missing,
                                out_dated = status.out_dated,
                                upto_date = status.upto_date
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
                        lang
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

                        -- record language-toc-item:
                        caption title:
                        string url:
                        language-toc-item list children:

                        -- language-toc-item list language-toc:
                        
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
