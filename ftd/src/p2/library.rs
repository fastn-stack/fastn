pub trait Library {
    fn get(&self, name: &str) -> Option<String>;
    fn get_with_result(&self, name: &str) -> crate::p1::Result<String> {
        match self.get(name) {
            Some(v) => Ok(v),
            None => crate::e(format!("library not found: {}", name)),
        }
    }
}

#[derive(Default)]
pub struct TestLibrary {
    pub libs: std::collections::HashMap<String, String>,
}

impl Library for TestLibrary {
    fn get(&self, name: &str) -> Option<String> {
        self.libs.get(name).map(ToString::to_string)
    }
}
