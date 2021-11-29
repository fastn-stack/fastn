pub fn check() -> (fpm::Config, String) {
    let root_dir = std::env::current_dir()
        .expect("Panic1")
        .to_str()
        .expect("panic")
        .to_string();
    let (_, package_folder_name) = root_dir.as_str().rsplit_once("/").expect("");
    let (_is_okay, base_dir) = find_fpm_file(root_dir.clone());
    let config = fpm::Config::parse(base_dir.clone());
    if package_folder_name != config.package {
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
