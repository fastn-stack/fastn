#[derive(Debug)]
pub struct Document {
    pub id: String,
    pub document: String,
    pub base_path: String,
    pub depth: usize,
}

pub fn process_dir(directory: &str) -> fpm::Result<Vec<Document>> {
    let mut documents: Vec<Document> = vec![];
    let directory = std::path::PathBuf::from(directory);

    process_dir_(&mut documents, &directory, 0, &directory)?;
    return Ok(documents);

    fn process_dir_(
        documents: &mut Vec<Document>,
        directory: &std::path::Path,
        depth: usize,
        base_path: &std::path::Path,
    ) -> fpm::Result<()> {
        for entry in std::fs::read_dir(&directory)? {
            let doc_path = entry?.path();
            let md = std::fs::metadata(&doc_path)?;

            if md.is_dir() {
                // Iterate the children
                let id = doc_path.to_str().unwrap_or_default().split('/').last();
                if id.is_some() && [".history", ".build", ".packages"].contains(&id.unwrap()) {
                    // ignore .history and .build directory
                    continue;
                }
                process_dir_(documents, &doc_path, depth + 1, base_path)?;
            } else if doc_path.to_str().unwrap_or_default().ends_with(".ftd") {
                // process the document
                let doc = std::fs::read_to_string(&doc_path)?;
                let id = doc_path.to_str().unwrap_or_default().split('/');
                let len = id.clone().count();

                documents.push(Document {
                    id: id
                        .skip(len - (depth + 1))
                        .take_while(|_| true)
                        .collect::<Vec<&str>>()
                        .join("/")
                        .to_string(),
                    document: doc,
                    base_path: base_path.to_str().unwrap_or_default().to_string(),
                    depth,
                });
            }
        }
        Ok(())
    }
}
