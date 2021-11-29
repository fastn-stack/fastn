pub struct Library {}

impl ftd::p2::Library for Library {
    fn get(&self, name: &str) -> Option<String> {
        if name == "fpm" {
            // return Some("".to_string());
            return Some(std::fs::read_to_string("fpm.ftd").unwrap());
        }
        std::fs::read_to_string(format!("./{}.ftd", name)).ok()
    }
}
