#[derive(Debug)]
pub struct Document {
    pub id: String,
    pub document: String,
    pub base_path: String,
    pub depth: usize,
}

pub fn process_dir(directory: &str) -> Vec<Document> {
    let mut documents: Vec<Document> = vec![];
    let directory = std::path::PathBuf::from(directory);

    process_dir_(&mut documents, &directory, 0, &directory);
    return documents;

    fn process_dir_(
        documents: &mut Vec<Document>,
        directory: &std::path::Path,
        depth: usize,
        base_path: &std::path::Path,
    ) -> u32 {
        let mut count: u32 = 0;
        for entry in std::fs::read_dir(&directory).expect("Panic! Unable to process the directory")
        {
            let doc_path = entry.expect("Panic: Doc not found").path();
            let md = std::fs::metadata(&doc_path).expect("Doc Metadata evaluation failed");

            if md.is_dir() {
                // Iterate the children
                let id = doc_path.to_str().unwrap().split('/').last();
                if id.is_some() && [".history", ".build", ".packages"].contains(&id.unwrap()) {
                    // ignore .history and .build directory
                    continue;
                }
                count += process_dir_(documents, &doc_path, depth + 1, base_path);
            } else if doc_path.to_str().unwrap_or_default().ends_with(".ftd") {
                // process the document
                let doc = std::fs::read_to_string(&doc_path).expect("cant read file");
                let id = doc_path.to_str().expect(">>>").split('/');
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
                count += 1;
            }
        }
        count
    }
}
