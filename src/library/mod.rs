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
            if let Some(ref last_marked_on) = lib.translated_data.last_marked_on {
                let time = std::time::SystemTime::UNIX_EPOCH
                    + std::time::Duration::from_nanos(*last_marked_on as u64);
                let rfc3339 = chrono::DateTime::<chrono::Utc>::from(time).to_rfc3339();
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
                let time = std::time::SystemTime::UNIX_EPOCH
                    + std::time::Duration::from_nanos(*original_latest as u64);
                let rfc3339 = chrono::DateTime::<chrono::Utc>::from(time).to_rfc3339();
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
                let time = std::time::SystemTime::UNIX_EPOCH
                    + std::time::Duration::from_nanos(*translated_latest as u64);
                let rfc3339 = chrono::DateTime::<chrono::Utc>::from(time).to_rfc3339();
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
