pub async fn build() {
    let (_fpm_config, base_dir) = fpm::check().await;

    std::fs::create_dir_all(format!("{}/.build", base_dir.as_str()).as_str())
        .expect("failed to create build folder");

    for doc in fpm::process_dir(base_dir.clone(), 0, base_dir) {
        write(&doc);
    }
}

fn write(doc: &fpm::Document) {
    use std::io::Write;

    let lib = fpm::Library {};
    let b = match ftd::p2::Document::from(&doc.id, &doc.document, &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse {}: {:?}", doc.id, &e);
            return;
        }
    };
    if !(doc.depth == 0 && doc.id.eq("index.ftd")) {
        std::fs::create_dir_all(format!(
            "{}/.build/{}",
            doc.base_path.as_str(),
            doc.id.replace(".ftd", "")
        ))
        .expect("failed to create directory folder for doc");
    }
    let new_file_path = format!(
        "{}/.build/{}",
        doc.base_path.as_str(),
        if doc.id.eq("index.ftd") {
            "index.html".to_string()
        } else {
            doc.id.replace(".ftd", "/index.html")
        }
    );
    let mut f = std::fs::File::create(new_file_path.as_str()).expect("failed to create .html file");

    let ftd_doc = b.to_rt("main", &doc.id);

    f.write_all(
        ftd::html()
            .replace(
                "__ftd_data__",
                serde_json::to_string_pretty(&ftd_doc.data)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .replace(
                "__ftd_external_children__",
                serde_json::to_string_pretty(&ftd_doc.external_children)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .replace("__ftd__", b.html("main", &doc.id).as_str())
            .replace("__ftd_js__", ftd::js())
            .as_bytes(),
    )
    .expect("failed to write to .html file");
    println!(
        "Generated {} [{}]",
        new_file_path,
        format!("{}/{}", doc.base_path, doc.id)
    );
}
