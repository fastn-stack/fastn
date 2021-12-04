pub async fn sync() -> fpm::Result<()> {
    let config = fpm::Config::read().await?;

    tokio::fs::create_dir_all(format!("{}/.history", config.root.as_str()).as_str()).await?;

    let snapshots = fpm::snaphot::get_latest_snapshots(config.root.as_str())?;

    let timestamp = fpm::get_timestamp_nanosecond();
    let mut modified_files = vec![];
    let mut new_snapshots = vec![];
    for doc in fpm::process_dir(config.root.as_str()).await? {
        if doc.id.starts_with(".history") {
            continue;
        }
        let (snapshot, is_modified) = write(&doc, timestamp, &snapshots).await?;
        if is_modified {
            modified_files.push(snapshot.file.to_string());
        }
        new_snapshots.push(snapshot);
    }

    if modified_files.is_empty() {
        println!("Everything is upto date.");
    } else {
        fpm::snaphot::create_latest_snapshots(config.root.as_str(), &new_snapshots).await?;
        println!(
            "Repo for {} is github, directly syncing with .history.",
            config.package.name
        );
        for file in modified_files {
            println!("{}", file);
        }
    }
    Ok(())
}

async fn write(
    doc: &fpm::Document,
    timestamp: u128,
    snapshots: &std::collections::BTreeMap<String, String>,
) -> fpm::Result<(fpm::Snapshot, bool)> {
    use tokio::io::AsyncWriteExt;

    if doc.id.contains('/') {
        let (dir, _) = doc.id.rsplit_once('/').unwrap();
        std::fs::create_dir_all(format!("{}/.history/{}", doc.base_path.as_str(), dir))?;
    }

    if let Some(timestamp) = snapshots.get(&doc.id) {
        let path = format!(
            "{}/.history/{}",
            doc.base_path.as_str(),
            doc.id.replace(".ftd", &format!(".{}.ftd", timestamp))
        );

        let existing_doc = tokio::fs::read_to_string(&path).await?;
        if doc.document.eq(&existing_doc) {
            return Ok((
                fpm::Snapshot {
                    file: doc.id.to_string(),
                    timestamp: timestamp.to_string(),
                },
                false,
            ));
        }
    }

    let new_file_path = format!(
        "{}/.history/{}",
        doc.base_path.as_str(),
        doc.id.replace(".ftd", &format!(".{}.ftd", timestamp))
    );

    let mut f = tokio::fs::File::create(new_file_path.as_str()).await?;
    f.write_all(doc.document.as_bytes()).await?;

    Ok((
        fpm::Snapshot {
            file: doc.id.to_string(),
            timestamp: timestamp.to_string(),
        },
        true,
    ))
}
