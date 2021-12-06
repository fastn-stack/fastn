mod http;
mod sqlite;
mod toc;

pub struct Library {}

impl ftd::p2::Library for Library {
    fn get(&self, name: &str) -> Option<String> {
        if name == "fpm" {
            return Some(fpm::fpm_ftd().to_string());
        }
        if let Ok(v) = std::fs::read_to_string(format!("./{}.ftd", name)) {
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
