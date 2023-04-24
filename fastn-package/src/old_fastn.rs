#[derive(Default)]
pub struct FastnLibrary {}

pub fn fastn_ftd() -> &'static str {
    include_str!("../fastn.ftd")
}

impl FastnLibrary {
    pub fn get(&self, name: &str, _doc: &ftd::ftd2021::p2::TDoc) -> Option<String> {
        if name == "fastn" {
            Some(format!(
                "{}\n\n-- optional package-data package:\n",
                fastn_ftd()
            ))
        } else {
            // Note: currently we do not allow users to import other modules from FASTN.ftd
            eprintln!("FASTN.ftd can only import `fastn` module");
            None
        }
    }

    pub fn get_with_result(
        &self,
        name: &str,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<String> {
        match self.get(name, doc) {
            Some(v) => Ok(v),
            None => ftd::ftd2021::p2::utils::e2(format!("library not found: {}", name), "", 0),
        }
    }
}
