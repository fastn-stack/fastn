pub async fn sync(files: Option<Vec<String>>) -> fpm::Result<()> {
    let config = fpm::Config::read().await?;

    let documents = if let Some(ref files) = files {
        let files = files.to_vec();
        let d = futures::future::join_all(
            files
                .into_iter()
                .map(|x| {
                    let base = config.root.clone();
                    tokio::spawn(async move {
                        fpm::process_file(
                            std::path::PathBuf::from(format!("{}/{}", base, x)),
                            base.as_str(),
                        )
                        .await
                    })
                })
                .collect::<Vec<tokio::task::JoinHandle<fpm::Result<fpm::FileFound>>>>(),
        )
        .await;
        let mut document = vec![];
        for doc in d.into_iter().flatten().flatten() {
            document.push(doc);
        }
        document
    } else {
        fpm::process_dir(config.root.as_str(), &config, fpm::ignore_history()).await?
    };

    tokio::fs::create_dir_all(format!("{}/.history", config.root.as_str()).as_str()).await?;

    let snapshots = fpm::snaphot::get_latest_snapshots(config.root.as_str())?;

    let timestamp = fpm::get_timestamp_nanosecond();
    let mut modified_files = vec![];
    let mut new_snapshots = vec![];
    for doc in documents {
        let (snapshot, is_modified) = write(&doc, timestamp, &snapshots).await?;
        if is_modified {
            modified_files.push(snapshot.file.to_string());
        }
        new_snapshots.push(snapshot);
    }

    if let Some(file) = files {
        let snapshot_id = new_snapshots
            .iter()
            .map(|v| v.file.to_string())
            .collect::<Vec<String>>();
        for (k, v) in snapshots {
            if !snapshot_id.contains(&k) && file.contains(&k) {
                continue;
            }
            if !snapshot_id.contains(&k) {
                new_snapshots.push(fpm::Snapshot {
                    file: k.to_string(),
                    timestamp: v.to_string(),
                })
            }
        }
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
    doc: &fpm::FileFound,
    timestamp: u128,
    snapshots: &std::collections::BTreeMap<String, String>,
) -> fpm::Result<(fpm::Snapshot, bool)> {
    if doc.get_id().contains('/') {
        let dir = doc.get_id().rsplit_once('/').unwrap().0.to_string();
        std::fs::create_dir_all(format!("{}/.history/{}", doc.get_base_path().as_str(), dir))?;
    }

    let file_extension = if let Some((_, b)) = doc.get_id().rsplit_once('.') {
        Some(b.to_string())
    } else {
        None
    };

    if let Some(timestamp) = snapshots.get(&doc.get_id()) {
        let path = format!("{}/.history/{}", doc.get_base_path().as_str(), {
            if let Some(ref ext) = file_extension {
                doc.get_id()
                    .replace(&format!(".{}", ext), &format!(".{}.{}", timestamp, ext))
            } else {
                format!(".{}", timestamp)
            }
        });

        if let Ok(current_doc) = tokio::fs::read_to_string(&doc.get_full_path()).await {
            let existing_doc = tokio::fs::read_to_string(&path).await?;
            if current_doc.eq(&existing_doc) {
                return Ok((
                    fpm::Snapshot {
                        file: doc.get_id(),
                        timestamp: timestamp.to_string(),
                    },
                    false,
                ));
            }
        }
    }

    let new_file_path = format!("{}/.history/{}", doc.get_base_path().as_str(), {
        if let Some(ext) = file_extension {
            doc.get_id()
                .replace(&format!(".{}", ext), &format!(".{}.{}", timestamp, ext))
        } else {
            format!(".{}", timestamp)
        }
    });

    tokio::fs::copy(doc.get_full_path(), new_file_path).await?;

    Ok((
        fpm::Snapshot {
            file: doc.get_id(),
            timestamp: timestamp.to_string(),
        },
        true,
    ))
}
