mod http;
mod sqlite;
mod toc;

#[derive(Default)]
pub struct Library {
    pub markdown: Option<(String, String)>,
}

impl ftd::p2::Library for Library {
    fn get(&self, name: &str) -> Option<String> {
        if name == "fpm" {
            let fpm_base = fpm::fpm_ftd().to_string();
            Some(match self.markdown.as_ref() {
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
            })
        } else if let Ok(v) = std::fs::read_to_string(format!("./{}.ftd", name)) {
            Some(v)
        } else if let Ok(v) = std::fs::read_to_string(format!("./.packages/{}.ftd", name)) {
            Some(v)
        } else if let Ok(v) = std::fs::read_to_string(format!("./.packages/{}/index.ftd", name)) {
            Some(v)
        } else if let Ok(v) = std::fs::read_to_string(format!("./.packages/.tmp/{}.ftd", name)) {
            Some(v)
        } else {
            std::fs::read_to_string(format!("./.packages/.tmp/{}/index.ftd", name)).ok()
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
            "package-query" => fpm::library::sqlite::processor(section, doc),
            t => unimplemented!("$processor$: {} is not implemented yet", t),
        }
    }
}
