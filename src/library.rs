pub struct Library {}

impl ftd::p2::Library for Library {
    fn get(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(format!("./examples/{}.ftd", name)).ok()
    }
}
