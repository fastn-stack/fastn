mod http;
mod sqlite;
mod toc;

pub struct Library {
    pub file_name: Option<String>,
    pub markdown_content: Option<String>,
}

impl ftd::p2::Library for Library {
    fn get(&self, name: &str) -> Option<String> {
        if name == "fpm" {
            // append the vars in the string
            let fpm_base = fpm::fpm_ftd().to_string();
            let fpm_base = if self.file_name.is_some() && self.markdown_content.is_some() {
                format!(
                    indoc::indoc! {"
                    {main_doc}
                    
                    -- string markdown-filename: {md_filename}
                    
                    -- string markdown-content:

                    {md_content}
                    "},
                    main_doc = fpm_base,
                    md_filename = self.file_name.as_ref().unwrap(),
                    md_content = self.markdown_content.as_ref().unwrap()
                )
            } else {
                fpm_base
            };
            Some(fpm_base)
        } else if let Ok(v) = std::fs::read_to_string(format!("./{}.ftd", name)) {
            Some(v)
        } else if let Ok(v) = std::fs::read_to_string(format!("./.packages/{}.ftd", name)) {
            Some(v)
        } else {
            std::fs::read_to_string(format!("./.packages/{}/index.ftd", name)).ok()
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
