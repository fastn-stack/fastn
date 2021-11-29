pub struct Library {}

impl ftd::p2::Library for Library {
    fn get(&self, name: &str) -> Option<String> {
        if name == "fpm" {
            return Some(include_str!("../fpm.ftd").to_string());
        }
        std::fs::read_to_string(format!("./{}.ftd", name)).ok()
    }
}
