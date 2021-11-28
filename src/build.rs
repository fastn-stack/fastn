pub fn build() {
    println!("Building...");
    std::fs::create_dir_all("./.build").expect("failed to create build folder");

    process_dir(
        std::env::current_dir()
            .expect("Panic1")
            .to_str()
            .expect("panic")
            .to_string(),
        0,
    );

}

pub fn process_dir(directory: String, depth: usize) -> u32 {
    let mut count: u32 = 0;
    for entry in std::fs::read_dir(&directory).expect("?//") {
        let e = entry.expect("Panic: Doc not found");
        let md = std::fs::metadata(e.path()).expect("Doc Metadata evaluation failed");
        let doc_path = e
            .path()
            .to_str()
            .expect("Directory path is expected")
            .to_string();
        if md.is_dir() {
            // Iterate the children
            count += process_dir(doc_path, depth + 1);
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
            );
            count += 1;
        }
    }
    count
}

fn write(id: &str, doc: String) {
    use std::io::Write;

    let lib = fpm::Library {};
    let b = match ftd::p2::Document::from(id, &*doc, &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse {}: {:?}", id, &e);
            return;
        }
    };

    std::fs::create_dir_all(format!("./.build/{}", id.replace(".ftd", "")))
        .expect("failed to create directory folder for doc");

    let mut f = std::fs::File::create(format!("./.build/{}", id.replace(".ftd", "/index.html")))
        .expect("failed to create .html file");

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
}
