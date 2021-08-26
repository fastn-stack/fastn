pub trait Library {
    fn get(&self, name: &str) -> Option<String>;
    fn get_with_result(&self, name: &str) -> crate::p1::Result<String> {
        match self.get(name) {
            Some(v) => Ok(v),
            None => crate::e(format!("library not found: {}", name)),
        }
    }
}

pub struct TestLibrary {}

impl Library for TestLibrary {
    fn get(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(format!("./tests/{}.ftd", name)).ok()
    }
}
