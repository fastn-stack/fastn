pub async fn check() -> (fpm::Package, String) {
    let root_dir = std::env::current_dir()
        .expect("Panic1")
        .to_str()
        .expect("panic")
        .to_string();
    let (_, package_folder_name) = root_dir.as_str().rsplit_once("/").expect("");
    let (_is_okay, base_dir) = find_fpm_file(root_dir.clone());

    let lib = fpm::Library {};
    let id = "fpm".to_string();
    let doc = std::fs::read_to_string(format!("{}/FPM.ftd", base_dir.as_str()))
        .unwrap_or_else(|_| panic!("cant read file. {}/FPM.ftd", base_dir.as_str()));
    let b = match ftd::p2::Document::from(id.as_str(), doc.as_str(), &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse {}: {:?}", id, &e);
            todo!();
        }
    };

    let config = fpm::Package::parse(&b);
    let _dep = fpm::Dependency::parse(&b)
        .into_iter()
        .map(|x| tokio::spawn(async move { x.process().await }))
        .collect::<Vec<tokio::task::JoinHandle<bool>>>();
    futures::future::join_all(_dep).await;

    if package_folder_name != config.name {
        todo!("package directory name mismatch")
    }
    (config, base_dir)
}

fn find_fpm_file(dir: String) -> (bool, String) {
    if std::path::Path::new(format!("{}/FPM.ftd", dir).as_str()).exists() {
        (true, dir)
    } else {
        if let Some((parent_dir, _)) = dir.rsplit_once("/") {
            return find_fpm_file(parent_dir.to_string());
        };
        (false, "".to_string())
    }
}
