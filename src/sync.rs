pub async fn sync() {
    let (_fpm_config, base_dir) = fpm::check().await;

    std::fs::create_dir_all(format!("{}/.history", base_dir.as_str()).as_str())
        .expect("failed to create build folder");

    let timestamp = fpm::get_timestamp_nanosecond();
    for doc in fpm::process_dir(base_dir.clone(), 0, base_dir) {
        write(&doc, timestamp);
    }
}

fn write(doc: &fpm::Document, timestamp: u128) {
    use std::io::Write;

    if doc.id.starts_with(".history") {
        return;
    }

    if doc.id.contains('/') {
        let (dir, _) = doc.id.rsplit_once('/').unwrap();
        std::fs::create_dir_all(format!("{}/.history/{}", doc.base_path.as_str(), dir))
            .expect("failed to create directory folder for doc");
    }

    let new_file_path = format!(
        "{}/.history/{}",
        doc.base_path.as_str(),
        doc.id.replace(".ftd", &format!(".{}.ftd", timestamp))
    );

    let mut f = std::fs::File::create(new_file_path.as_str()).expect("failed to create .html file");

    f.write_all(doc.document.as_bytes())
        .expect("failed to write to .html file");
    println!(
        "Generated history [{}]",
        format!("{}/{}", doc.base_path, doc.id)
    );
}
