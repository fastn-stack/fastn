async fn template_contents(project_name: &str) -> (String, String) {
    let ftd = format!("-- import: fpm\n\n-- fpm.package: {}", project_name);
    let index = "-- ftd.text: Hello world".to_string();

    (ftd, index)
}

pub async fn start_project(name: &str, path: Option<&str>) -> fpm::Result<()> {
    let base_path = {
        match std::env::current_dir() {
            Ok(bp) => match bp.to_str() {
                Some(fbp) => fbp.to_string(),
                None => "None".to_string(),
            },
            Err(_) => panic!("Error cannot find the current working directory!!"),
        }
    };

    // Not using config for base path as it requires manifest or FPM.ftd file for building and will throw error
    // and since this command should work from anywhere within the system
    // so we dont need to rely on config for using it

    // name is a required field so it will always be some defined string (cant be None)
    // name can be any package url or standard project name
    // path is an optional field and if no path is provided then current directory is used

    let final_dir = {
        match path {
            Some(p) => camino::Utf8PathBuf::from(base_path).join(p).join(name),
            None => camino::Utf8PathBuf::from(base_path).join(name),
        }
    };

    // Create all directories if not present
    tokio::fs::create_dir_all(final_dir.as_str()).await?;

    let tmp_contents = template_contents(name).await;
    let tmp_fpm = tmp_contents.0;
    let tmp_index = tmp_contents.1;

    fpm::utils::update(&final_dir.join("FPM.ftd"), tmp_fpm.as_bytes()).await?;
    fpm::utils::update(&final_dir.join("index.ftd"), tmp_index.as_bytes()).await?;

    let sync_message = "Initial sync".to_string();
    let file_list: std::collections::BTreeMap<String, fpm::history::FileEditTemp> =
        IntoIterator::into_iter([
            (
                "FPM.ftd".to_string(),
                fpm::history::FileEditTemp {
                    message: Some(sync_message.to_string()),
                    author: None,
                    src_cr: None,
                    operation: fpm::history::FileOperation::Added,
                },
            ),
            (
                "index.ftd".to_string(),
                fpm::history::FileEditTemp {
                    message: Some(sync_message.to_string()),
                    author: None,
                    src_cr: None,
                    operation: fpm::history::FileOperation::Added,
                },
            ),
        ])
        .collect();

    fpm::history::insert_into_history(&final_dir, &file_list, &mut Default::default()).await?;

    println!(
        "Template FTD project created - {}\nPath -{}",
        name, final_dir
    );

    Ok(())
}
