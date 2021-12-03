#[derive(Debug)]
pub struct Document {
    pub id: String,
    pub document: String,
    pub base_path: String,
    pub depth: usize,
}

pub async fn process_dir(directory: &str) -> fpm::Result<Vec<Document>> {
    let mut documents: Vec<Document> = vec![];
    let directory = std::path::PathBuf::from(directory);

    process_dir_(&mut documents, &directory, 0, &directory).await?;
    return Ok(documents);

    #[async_recursion::async_recursion]
    async fn process_dir_(
        documents: &mut Vec<Document>,
        directory: &std::path::Path,
        depth: usize,
        base_path: &std::path::Path,
    ) -> fpm::Result<()> {
        let mut r = tokio::fs::read_dir(&directory).await?;
        while let Some(entry) = r.next_entry().await? {
            let doc_path = entry.path();
            let md = tokio::fs::metadata(&doc_path).await?;

            if md.is_dir() {
                // Iterate the children
                let id = doc_path.to_str().unwrap_or_default().split('/').last();
                if id.is_some() && [".history", ".build", ".packages"].contains(&id.unwrap()) {
                    // ignore .history and .build directory
                    continue;
                }
                process_dir_(documents, &doc_path, depth + 1, base_path).await?;
            } else if doc_path.to_str().unwrap_or_default().ends_with(".ftd") {
                // process the document
                let doc = tokio::fs::read_to_string(&doc_path).await?;
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
