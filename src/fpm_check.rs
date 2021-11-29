pub fn fpm_check() -> fpm::FPMConfig {
    let (is_okay, base_dir) = find_fpm_file(
        std::env::current_dir()
            .expect("Panic1")
            .to_str()
            .expect("panic")
            .to_string(),
    );
    fpm::FPMConfig::parse(base_dir)
}

fn find_fpm_file(dir: String) -> (bool, String) {
    if std::path::Path::new(format!("{}/.FPM.ftd", dir).as_str()).exists() {
        (true, dir)
    } else {
        if let Some((parent_dir, _)) = dir.rsplit_once("/") {
            return find_fpm_file(parent_dir.to_string());
        };
        (false, "".to_string())
    }
}
