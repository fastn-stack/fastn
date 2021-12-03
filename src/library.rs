pub struct Library {}

impl ftd::p2::Library for Library {
    fn get(&self, name: &str) -> Option<String> {
        if name == "fpm" {
            return Some(fpm::fpm_ftd().to_string());
        }
        if let Ok(v) = std::fs::read_to_string(format!("./{}.ftd", name)) {
            Some(v)
        } else {
            std::fs::read_to_string(format!("./.packages/{}.ftd", name)).ok()
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
            "ft-toc" => fpm::toc::processor(section, doc),
            t => unimplemented!("$processor$: {} is not implemented yet", t),
        }
    }
}
