pub async fn sync() {
    let (fpm_config, base_dir) = fpm::check().await;

    std::fs::create_dir_all(format!("{}/.history", base_dir.as_str()).as_str())
        .expect("failed to create build folder");

    let timestamp = fpm::get_timestamp_nanosecond();

    let mut modified_files = vec![];
    for doc in fpm::process_dir(base_dir.clone(), 0, base_dir) {
        if let Some(file) = write(&doc, timestamp) {
            modified_files.push(file);
        }
    }
    if modified_files.is_empty() {
        println!("Everything is upto date.");
    } else {
        println!(
            "Repo for {} is github, directly syncing with .history.",
            fpm_config.name
        );
        for file in modified_files {
            println!("{}", file);
        }
    }
}

fn write(doc: &fpm::Document, timestamp: u128) -> Option<String> {
    use std::io::Write;

    if doc.id.starts_with(".history") {
        return None;
    }

    let (path, doc_name) = if doc.id.contains('/') {
        let (dir, doc_name) = doc.id.rsplit_once('/').unwrap();
        std::fs::create_dir_all(format!("{}/.history/{}", doc.base_path.as_str(), dir))
            .expect("failed to create directory folder for doc");
        (
            format!("{}/.history/{}", doc.base_path.as_str(), dir),
            doc_name.to_string(),
        )
    } else {
        (
            format!("{}/.history", doc.base_path.as_str()),
            doc.id.to_string(),
        )
    };

    let files = std::fs::read_dir(&path).expect("Panic! Unable to process the directory");

    let mut max_timestamp: Option<(String, String)> = None;
    for n in files.flatten() {
        let p = format!("{}/{}.", path, doc_name.replace(".ftd", ""));
        let file = n.path().to_str().unwrap().to_string();
        if file.starts_with(&p) {
            let timestamp = file
                .replace(&format!("{}/{}.", path, doc_name.replace(".ftd", "")), "")
                .replace(".ftd", "");
            if let Some((t, _)) = &max_timestamp {
                if *t > timestamp {
                    continue;
                }
            }
            max_timestamp = Some((timestamp, file.to_string()));
        }
    }

    if let Some((_, path)) = max_timestamp {
        let existing_doc = std::fs::read_to_string(&path).expect("cant read file");
        if doc.document.eq(&existing_doc) {
            return None;
        }
    }

    let new_file_path = format!(
        "{}/.history/{}",
        doc.base_path.as_str(),
        doc.id.replace(".ftd", &format!(".{}.ftd", timestamp))
    );

    let mut f = std::fs::File::create(new_file_path.as_str()).expect("failed to create .html file");

    f.write_all(doc.document.as_bytes())
        .expect("failed to write to .html file");
    Some(doc.id.to_string())
}
