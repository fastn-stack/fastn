pub async fn start_tracking(config: &fpm::Config, who: &str, whom: &str) -> fpm::Result<()> {
    tokio::fs::create_dir_all(config.track_dir()).await?;

    let snapshots = fpm::snapshot::get_latest_snapshots(&config.root).await?;
    check(config.root.as_str(), &snapshots, who, whom).await?;
    Ok(())
}

async fn check(
    base_path: &str,
    snapshots: &std::collections::BTreeMap<String, u128>,
    who: &str,
    whom: &str,
) -> fpm::Result<()> {
    if !snapshots.contains_key(whom) {
        return Err(fpm::Error::UsageError {
            message: format!(
                "{} is not synced yet. suggestion: Run `fpm sync {}` to sync the file",
                whom, whom
            ),
        });
    }

    let timestamp = if let Some(timestamp) = snapshots.get(who) {
        timestamp
    } else {
        return Err(fpm::Error::UsageError {
            message: format!(
                "{} is not synced yet. suggestion: Run `fpm sync {}` to sync the file",
                who, who
            ),
        });
    };

    if let Some((dir, _)) = who.rsplit_once('/') {
        tokio::fs::create_dir_all(
            camino::Utf8PathBuf::from(base_path)
                .join(".tracks")
                .join(dir),
        )
        .await?;
    }

    let new_file_path = fpm::utils::track_path(who, base_path);

    write(whom, *timestamp, &new_file_path).await?;
    println!("{} is now tracking {}", who, whom);

    Ok(())
}

async fn write(whom: &str, timestamp: u128, path: &camino::Utf8PathBuf) -> fpm::Result<()> {
    use tokio::io::AsyncWriteExt;
    let string = if path.exists() {
        let existing_doc = tokio::fs::read_to_string(path).await?;
        format!(
            "{}\n\n-- fpm.track: {}\nself-timestamp: {}",
            existing_doc, whom, timestamp
        )
    } else {
        format!(
            "-- import: fpm\n\n-- fpm.track: {}\nself-timestamp: {}",
            whom, timestamp
        )
    };

    let mut f = tokio::fs::File::create(path).await?;
    f.write_all(string.as_bytes()).await?;
    Ok(())
}
