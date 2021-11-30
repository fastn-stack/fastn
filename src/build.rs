pub async fn build() {
    let (_fpm_config, base_dir) = fpm::check().await;

    std::fs::create_dir_all(format!("{}/.build", base_dir.as_str()).as_str())
        .expect("failed to create build folder");

    process_dir(base_dir.clone(), 0, base_dir);
}

pub fn process_dir(directory: String, depth: usize, base_path: String) -> u32 {
    let mut count: u32 = 0;
    for entry in std::fs::read_dir(&directory).expect("Panic! Unable to process the directory") {
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
            count += process_dir(doc_path, depth + 1, base_path.as_str().to_string());
        } else if doc_path.as_str().ends_with(".ftd") {
            // process the document
            let doc = std::fs::read_to_string(doc_path).expect("cant read file");
            let id = e.path().clone();
            let id = id.to_str().expect(">>>").split('/');
            let len = id.clone().count();

            write(
                id.skip(len - (depth + 1))
                    .take_while(|_| true)
                    .collect::<Vec<&str>>()
                    .join("/")
                    .as_str(),
                doc,
                base_path.as_str().to_string(),
                depth,
            );
            count += 1;
        }
    }
    count
}

fn write(id: &str, doc: String, base_path: String, depth: usize) {
    use std::io::Write;

    let lib = fpm::Library {};
    let b = match ftd::p2::Document::from(id, &*doc, &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse {}: {:?}", id, &e);
            return;
        }
    };
    if !(depth == 0 && id == "index.ftd") {
        std::fs::create_dir_all(format!(
            "{}/.build/{}",
            base_path.as_str(),
            id.replace(".ftd", "")
        ))
        .expect("failed to create directory folder for doc");
    }
    let new_file_path = format!(
        "{}/.build/{}",
        base_path.as_str(),
        if id == "index.ftd" {
            "index.html".to_string()
        } else {
            id.replace(".ftd", "/index.html")
        }
    );
    let mut f = std::fs::File::create(new_file_path.as_str()).expect("failed to create .html file");

    let doc = b.to_rt("main", id);

    f.write_all(
        ftd::html()
            .replace(
                "__ftd_data__",
                serde_json::to_string_pretty(&doc.data)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .replace(
                "__ftd_external_children__",
                serde_json::to_string_pretty(&doc.external_children)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .replace("__ftd__", b.html("main", id).as_str())
            .replace("__ftd_js__", ftd::js())
            .as_bytes(),
    )
    .expect("failed to write to .html file");
    println!(
        "Generated {} [{}]",
        new_file_path,
        format!("{}/{}", base_path, id)
    );
}
