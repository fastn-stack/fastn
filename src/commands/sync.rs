pub async fn sync(config: &fpm::Config, files: Option<Vec<String>>) -> fpm::Result<()> {
    let documents = if let Some(ref files) = files {
        let files = files
            .to_vec()
            .into_iter()
            .map(|x| config.root.join(x))
            .collect::<Vec<camino::Utf8PathBuf>>();
        fpm::paths_to_files(files, config.root.as_path()).await?
    } else {
        fpm::get_documents(config).await?
    };

    tokio::fs::create_dir_all(config.history_dir()).await?;

    let snapshots = fpm::snapshot::get_latest_snapshots(&config.root).await?;

    let timestamp = fpm::get_timestamp_nanosecond();
    let mut modified_files = vec![];
    let mut new_snapshots = vec![];
    for doc in documents {
        let (snapshot, is_modified) = write(&doc, timestamp, &snapshots).await?;
        if is_modified {
            modified_files.push(snapshot.filename.to_string());
        }
        new_snapshots.push(snapshot);
    }

    if let Some(file) = files {
        let snapshot_id = new_snapshots
            .iter()
            .map(|v| v.filename.to_string())
            .collect::<Vec<String>>();
        for (k, timestamp) in snapshots.iter() {
            if !snapshot_id.contains(k) && file.contains(k) {
                continue;
            }
            if !snapshot_id.contains(k) {
                new_snapshots.push(fpm::Snapshot {
                    filename: k.clone(),
                    timestamp: *timestamp,
                })
            }
        }
    }

    for key in snapshots.keys() {
        if new_snapshots.iter().filter(|v| v.filename.eq(key)).count() == 0 {
            modified_files.push(key.clone());
        }
    }

    if modified_files.is_empty() {
        println!("Everything is upto date.");
    } else {
        fpm::snapshot::create_latest_snapshots(config, &new_snapshots).await?;
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
    doc: &fpm::File,
    timestamp: u128,
    snapshots: &std::collections::BTreeMap<String, u128>,
) -> fpm::Result<(fpm::Snapshot, bool)> {
    if let Some((dir, _)) = doc.get_id().rsplit_once('/') {
        tokio::fs::create_dir_all(
            camino::Utf8PathBuf::from(doc.get_base_path())
                .join(".history")
                .join(dir),
        )
        .await?;
    }

    if let Some(timestamp) = snapshots.get(&doc.get_id()) {
        let path = fpm::utils::history_path(&doc.get_id(), &doc.get_base_path(), timestamp);
        if let Ok(current_doc) = tokio::fs::read_to_string(&doc.get_full_path()).await {
            let existing_doc = tokio::fs::read_to_string(&path).await?;
            if current_doc.eq(&existing_doc) {
                return Ok((
                    fpm::Snapshot {
                        filename: doc.get_id(),
                        timestamp: *timestamp,
                    },
                    false,
                ));
            }
        }
    }

    let new_file_path = fpm::utils::history_path(&doc.get_id(), &doc.get_base_path(), &timestamp);

    tokio::fs::copy(doc.get_full_path(), new_file_path).await?;

    Ok((
        fpm::Snapshot {
            filename: doc.get_id(),
            timestamp,
        },
        true,
    ))
}
