mod http;
mod sqlite;
mod toc;

pub struct Library {
    pub config: fpm::Config,
    pub markdown: Option<(String, String)>,
    pub document_id: String,
    pub diff: Option<String>,
}

impl ftd::p2::Library for Library {
    fn get(&self, name: &str) -> Option<String> {
        if name == "fpm" {
            let fpm_base = {
                let mut fpm_base = format!(
                    indoc::indoc! {"
                        {fpm_base}
                        
                        -- string document-title: {document_id}
    
                        "},
                    fpm_base = fpm::fpm_ftd().to_string(),
                    document_id = self.document_id,
                );
                if let Some(ref diff) = self.diff {
                    fpm_base = format!(
                        indoc::indoc! {"
                        {fpm_base}
                        
                        -- string diff: 
                        
                        {diff}
    
                        "},
                        fpm_base = fpm_base,
                        diff = diff,
                    );
                }
                fpm_base
            };
            return Some(match self.markdown.as_ref() {
                Some((filename, content)) => format!(
                    indoc::indoc! {"
                        {fpm_base}
                        
                        -- string markdown-filename: {filename}
                        
                        -- string markdown-content:
    
                        {content}
                    "},
                    fpm_base = fpm_base,
                    filename = filename,
                    content = content
                ),
                _ => fpm_base,
            });
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
