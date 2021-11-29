pub fn setup() {
    match home::home_dir() {
        Some(path) => {
            // println!("{}", path.display())
            let fpm_dir_path = format!("{}/.fpm/", path.display());
            let fpm_ftd_path = format!("{}/.fpm.ftd", fpm_dir_path.as_str());
            if !std::path::Path::new(fpm_dir_path.as_str()).exists() {
                std::fs::create_dir(fpm_dir_path.as_str())
                    .expect("Panic! Unable to create the .fpm directory.")
            }
            // Create the base .fpm.ftd file
            if !std::path::Path::new(fpm_ftd_path.as_str()).exists() {
                std::fs::File::create(fpm_ftd_path).expect("Unable to create the .fpm.ftd file");
            }
        }
        None => println!("Impossible to get your home dir!"),
    }
}
