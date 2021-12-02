#[derive(Debug)]
pub struct Document {
    pub id: String,
    pub document: String,
    pub base_path: String,
    pub depth: usize,
}

pub fn process_dir(directory: String, depth: usize, base_path: String) -> Vec<Document> {
    let mut documents: Vec<Document> = vec![];
    process_dir_(&mut documents, directory, depth, base_path);
    return documents;

    fn process_dir_(
        documents: &mut Vec<Document>,
        directory: String,
        depth: usize,
        base_path: String,
    ) -> u32 {
        let mut count: u32 = 0;
        for entry in std::fs::read_dir(&directory).expect("Panic! Unable to process the directory")
        {
            let e = entry.expect("Panic: Doc not found");
            let md = std::fs::metadata(e.path()).expect("Doc Metadata evaluation failed");
            let doc_path = e
                .path()
                .to_str()
                .expect("Directory path is expected")
                .to_string();

            if depth == 0 && doc_path.as_str().ends_with("FPM.ftd") {
                // pass the FPM.ftd file at the base level
            } else if md.is_dir() {
                // Iterate the children
                let id = doc_path.split('/').last();
                if id.is_some() && [".history", ".build", ".packages"].contains(&id.unwrap()) {
                    // ignore .history and .build directory
                    continue;
                }
                count += process_dir_(
                    documents,
                    doc_path,
                    depth + 1,
                    base_path.as_str().to_string(),
                );
            } else if doc_path.as_str().ends_with(".ftd") {
                // process the document
                let doc = std::fs::read_to_string(doc_path).expect("cant read file");
                let id = e.path().clone();
                let id = id.to_str().expect(">>>").split('/');
                let len = id.clone().count();

                documents.push(Document {
                    id: id
                        .skip(len - (depth + 1))
                        .take_while(|_| true)
                        .collect::<Vec<&str>>()
                        .join("/")
                        .to_string(),
                    document: doc,
                    base_path: base_path.as_str().to_string(),
                    depth,
                });
                count += 1;
            }
        }
        count
    }
}
