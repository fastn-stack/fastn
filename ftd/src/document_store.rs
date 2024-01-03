pub trait DocumentStore: std::fmt::Debug + Clone {
    fn read(&self, path: &str, user_id: Option<u32>) -> ftd::interpreter::Result<Vec<u8>>;
    fn write(&self, path: &str, data: &[u8], user_id: Option<u32>) -> ftd::interpreter::Result<()>;
}

#[derive(Clone, Debug)]
struct FSStore {
    root: String,
}

impl FSStore {
    pub fn new(root: String) -> Self {
        Self { root }
    }
    fn path(&self, path: &str) -> String {
        format!("{}/{}", self.root, path)
    }
}

impl DocumentStore for FSStore {
    fn read(&self, path: &str, user_id: Option<u32>) -> ftd::interpreter::Result<Vec<u8>> {
        use std::io::Read;

        let mut file = std::fs::File::open(self.path(path))?;
        let mut contents = vec![];
        file.read_to_end(&mut contents)?;
        Ok(contents)
    }

    fn write(&self, path: &str, data: &[u8], user_id: Option<u32>) -> ftd::interpreter::Result<()> {
        use std::io::Write;

        let mut file = std::fs::File::create(self.path(path))?;
        file.write_all(data)?;
        Ok(())
    }
}
