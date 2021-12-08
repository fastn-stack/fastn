pub async fn status() -> fpm::Result<()> {
    let config = fpm::Config::read().await?;
    let snapshots = fpm::snaphot::get_latest_snapshots(config.root.as_str())?;
    let mut filestatus = std::collections::BTreeMap::new();
    for doc in fpm::process_dir(config.root.as_str(), &config).await? {
        if doc.id.starts_with(".history") {
            continue;
        }
        let status = get_file_status(&doc, &snapshots).await?;
        filestatus.insert(doc.id, status);
    }

    print_file_status(&snapshots, &filestatus);
    Ok(())
}

async fn get_file_status(
    doc: &fpm::Document,
    snapshots: &std::collections::BTreeMap<String, String>,
) -> fpm::Result<FileStatus> {
    if let Some(timestamp) = snapshots.get(&doc.id) {
        let path = format!(
            "{}/.history/{}",
            doc.base_path.as_str(),
            doc.id.replace(".ftd", &format!(".{}.ftd", timestamp))
        );

        let existing_doc = tokio::fs::read_to_string(&path).await?;
        if doc.document.eq(&existing_doc) {
            return Ok(FileStatus::None);
        }
        return Ok(FileStatus::Modified);
    }
    Ok(FileStatus::Added)
}

fn print_file_status(
    snapshots: &std::collections::BTreeMap<String, String>,
    filestatus: &std::collections::BTreeMap<String, FileStatus>,
) {
    let mut any_file_removed = false;
    for id in snapshots.keys() {
        if let Some(status) = filestatus.get(id) {
            if status.eq(&FileStatus::None) {
                continue;
            }
            println!("{:?}: {}", status, id);
        } else {
            any_file_removed = true;
            println!("{:?}: {}", FileStatus::Removed, id);
        }
    }

    for (id, status) in filestatus
        .iter()
        .filter(|(_, f)| f.eq(&&FileStatus::Added))
        .collect::<Vec<(&String, &FileStatus)>>()
    {
        println!("{:?}: {}", status, id);
    }
    if !(filestatus.iter().any(|(_, f)| !f.eq(&FileStatus::None)) || any_file_removed) {
        println!("Nothing to sync, clean working tree");
    }
}

#[derive(Debug, PartialEq)]
enum FileStatus {
    Modified,
    Added,
    Removed,
    None,
}
